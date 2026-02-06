//! Minimal VST3 gain plugin showcasing toybox VST3 helpers.

#![deny(clippy::missing_docs_in_private_items, missing_docs, warnings)]

use std::cell::Cell;
use std::ffi::{CStr, c_void};
use std::ptr;
use std::slice;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};

use toybox::vst3::prelude::Steinberg::*;
use toybox::vst3::prelude::*;

/// Human-readable plugin name shown in hosts.
const PLUGIN_NAME: &str = "Toybox Minimal Gain";
/// VST3 parameter id for gain.
const PARAM_GAIN_ID: ParamID = 0;
/// State payload magic (`TVST`).
const STATE_MAGIC: u32 = u32::from_le_bytes(*b"TVST");
/// State payload version.
const STATE_VERSION: u32 = 1;

/// Convert gain in linear domain to normalized VST3 value.
fn gain_to_normalized(gain: f64) -> f64 {
    ParamRange::new(0.0, 2.0).plain_to_normalized(gain)
}

/// Convert normalized VST3 value to linear gain.
fn normalized_to_gain(value: f64) -> f64 {
    ParamRange::new(0.0, 2.0).normalized_to_plain(value)
}

/// Parse a UTF-16 VST3 string into `f64`.
unsafe fn parse_tchar_f64(text: *mut TChar) -> Option<f64> {
    if text.is_null() {
        return None;
    }

    // SAFETY: caller guarantees a valid null-terminated TChar pointer.
    let length = unsafe { tchar_len(text as *const TChar) };
    // SAFETY: caller guarantees this pointer references `length` UTF-16 units.
    let utf16 = unsafe { slice::from_raw_parts(text.cast::<u16>(), length) };
    let parsed = String::from_utf16(utf16).ok()?;
    let trimmed = parsed.trim().trim_end_matches('x');
    f64::from_str(trimmed).ok()
}

/// Read normalized gain value from a serialized plugin state stream.
unsafe fn load_normalized_gain(state: *mut IBStream) -> Result<f64, StreamError> {
    // SAFETY: caller provides a host-owned VST3 state stream pointer.
    let payload = unsafe { read_versioned_payload(state, STATE_MAGIC, &[STATE_VERSION]) }?;
    if payload.payload.len() != 8 {
        return Err(StreamError::InvalidHeader);
    }

    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&payload.payload);
    Ok(f64::from_le_bytes(bytes).clamp(0.0, 1.0))
}

/// Write normalized gain value to a serialized plugin state stream.
unsafe fn store_normalized_gain(state: *mut IBStream, normalized_gain: f64) -> tresult {
    let payload = normalized_gain.to_le_bytes();
    // SAFETY: caller provides a host-owned VST3 state stream pointer.
    match unsafe { write_versioned_payload(state, STATE_MAGIC, STATE_VERSION, &payload) } {
        Ok(()) => kResultOk,
        Err(_) => kResultFalse,
    }
}

/// Minimal gain processor implementation.
struct GainProcessor {
    /// Normalized gain value represented as f64 bits for atomic access.
    gain_normalized_bits: AtomicU64,
}

impl GainProcessor {
    /// Unique class identifier for the processor component.
    const CID: TUID = uid(0xEE96D1A4, 0x53B140DB, 0x8F52EC7B, 0x09C55BA8);

    /// Create a processor with default gain of 1.0x.
    fn new() -> Self {
        Self {
            gain_normalized_bits: AtomicU64::new(gain_to_normalized(1.0).to_bits()),
        }
    }

    /// Read the current normalized gain value.
    fn normalized_gain(&self) -> f64 {
        f64::from_bits(self.gain_normalized_bits.load(Ordering::Relaxed)).clamp(0.0, 1.0)
    }

    /// Store a normalized gain value.
    fn set_normalized_gain(&self, value: f64) {
        self.gain_normalized_bits
            .store(value.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    /// Read gain in linear units used for sample scaling.
    fn linear_gain(&self) -> f32 {
        normalized_to_gain(self.normalized_gain()) as f32
    }
}

impl Class for GainProcessor {
    type Interfaces = (IComponent, IAudioProcessor, IProcessContextRequirements);
}

impl IPluginBaseTrait for GainProcessor {
    unsafe fn initialize(&self, _context: *mut FUnknown) -> tresult {
        kResultOk
    }

    unsafe fn terminate(&self) -> tresult {
        kResultOk
    }
}

impl IComponentTrait for GainProcessor {
    unsafe fn getControllerClassId(&self, class_id: *mut TUID) -> tresult {
        if class_id.is_null() {
            return kInvalidArgument;
        }

        // SAFETY: `class_id` is validated non-null above.
        unsafe { *class_id = GainController::CID };
        kResultOk
    }

    unsafe fn setIoMode(&self, _mode: IoMode) -> tresult {
        kResultOk
    }

    unsafe fn getBusCount(&self, media_type: MediaType, dir: BusDirection) -> i32 {
        match media_type as MediaTypes {
            MediaTypes_::kAudio => match dir as BusDirections {
                BusDirections_::kInput | BusDirections_::kOutput => 1,
                _ => 0,
            },
            _ => 0,
        }
    }

    unsafe fn getBusInfo(
        &self,
        media_type: MediaType,
        dir: BusDirection,
        index: i32,
        bus: *mut BusInfo,
    ) -> tresult {
        if bus.is_null() || index != 0 {
            return kInvalidArgument;
        }

        if media_type as MediaTypes != MediaTypes_::kAudio {
            return kInvalidArgument;
        }

        let label = match dir as BusDirections {
            BusDirections_::kInput => "Input",
            BusDirections_::kOutput => "Output",
            _ => return kInvalidArgument,
        };

        // SAFETY: pointer is checked non-null above.
        let bus = unsafe { &mut *bus };
        bus.mediaType = MediaTypes_::kAudio as MediaType;
        bus.direction = dir;
        bus.channelCount = 2;
        copy_wstring(label, &mut bus.name);
        bus.busType = BusTypes_::kMain as BusType;
        bus.flags = BusInfo_::BusFlags_::kDefaultActive as u32;

        kResultOk
    }

    unsafe fn getRoutingInfo(
        &self,
        _in_info: *mut RoutingInfo,
        _out_info: *mut RoutingInfo,
    ) -> tresult {
        kNotImplemented
    }

    unsafe fn activateBus(
        &self,
        _media_type: MediaType,
        _dir: BusDirection,
        _index: i32,
        _state: TBool,
    ) -> tresult {
        kResultOk
    }

    unsafe fn setActive(&self, _state: TBool) -> tresult {
        kResultOk
    }

    unsafe fn setState(&self, state: *mut IBStream) -> tresult {
        // SAFETY: host provides state stream pointer per VST3 contract.
        let Ok(normalized_gain) = (unsafe { load_normalized_gain(state) }) else {
            return kInvalidArgument;
        };
        self.set_normalized_gain(normalized_gain);
        kResultOk
    }

    unsafe fn getState(&self, state: *mut IBStream) -> tresult {
        // SAFETY: host provides state stream pointer per VST3 contract.
        unsafe { store_normalized_gain(state, self.normalized_gain()) }
    }
}

impl IAudioProcessorTrait for GainProcessor {
    unsafe fn setBusArrangements(
        &self,
        inputs: *mut SpeakerArrangement,
        num_ins: i32,
        outputs: *mut SpeakerArrangement,
        num_outs: i32,
    ) -> tresult {
        if num_ins != 1 || num_outs != 1 {
            return kResultFalse;
        }

        if inputs.is_null() || outputs.is_null() {
            return kInvalidArgument;
        }

        // SAFETY: non-null pointers validated above.
        if unsafe { *inputs } != SpeakerArr::kStereo || unsafe { *outputs } != SpeakerArr::kStereo {
            return kResultFalse;
        }

        kResultTrue
    }

    unsafe fn getBusArrangement(
        &self,
        dir: BusDirection,
        index: i32,
        arr: *mut SpeakerArrangement,
    ) -> tresult {
        if arr.is_null() || index != 0 {
            return kInvalidArgument;
        }

        match dir as BusDirections {
            BusDirections_::kInput | BusDirections_::kOutput => {
                // SAFETY: `arr` was validated as non-null.
                unsafe { *arr = SpeakerArr::kStereo };
                kResultOk
            }
            _ => kInvalidArgument,
        }
    }

    unsafe fn canProcessSampleSize(&self, symbolic_sample_size: i32) -> tresult {
        match symbolic_sample_size as SymbolicSampleSizes {
            SymbolicSampleSizes_::kSample32 => kResultOk,
            SymbolicSampleSizes_::kSample64 => kNotImplemented,
            _ => kInvalidArgument,
        }
    }

    unsafe fn getLatencySamples(&self) -> u32 {
        0
    }

    unsafe fn setupProcessing(&self, _setup: *mut ProcessSetup) -> tresult {
        kResultOk
    }

    unsafe fn setProcessing(&self, _state: TBool) -> tresult {
        kResultOk
    }

    unsafe fn process(&self, data: *mut ProcessData) -> tresult {
        if data.is_null() {
            return kInvalidArgument;
        }

        // SAFETY: pointer validated non-null above.
        let process_data = unsafe { &*data };

        // SAFETY: VST3 host owns parameter event data during the process callback.
        if let Some((_, value)) =
            unsafe { latest_param_point(process_data.inputParameterChanges, PARAM_GAIN_ID) }
        {
            self.set_normalized_gain(value);
        }

        let gain = self.linear_gain();
        // SAFETY: `process_data` references host-owned buffer arrays valid for this callback.
        let Some(buffers) = (unsafe { stereo_f32_buffers(process_data) }) else {
            return process_ok();
        };

        for sample_index in 0..buffers.num_samples {
            buffers.output_left[sample_index] = buffers.input_left[sample_index] * gain;
            buffers.output_right[sample_index] = buffers.input_right[sample_index] * gain;
        }

        process_ok()
    }

    unsafe fn getTailSamples(&self) -> u32 {
        0
    }
}

impl IProcessContextRequirementsTrait for GainProcessor {
    unsafe fn getProcessContextRequirements(&self) -> u32 {
        0
    }
}

/// Minimal controller implementation handling parameter and GUI state.
struct GainController {
    /// Current normalized gain value.
    gain_normalized: Cell<f64>,
}

impl GainController {
    /// Unique class identifier for the edit controller.
    const CID: TUID = uid(0xA50AF46B, 0xCC7B43FF, 0x8CF8A3E0, 0xA2C0EE33);

    /// Create a controller initialized to unity gain.
    fn new() -> Self {
        Self {
            gain_normalized: Cell::new(gain_to_normalized(1.0)),
        }
    }
}

impl Class for GainController {
    type Interfaces = (IEditController,);
}

impl IPluginBaseTrait for GainController {
    unsafe fn initialize(&self, _context: *mut FUnknown) -> tresult {
        kResultOk
    }

    unsafe fn terminate(&self) -> tresult {
        kResultOk
    }
}

impl IEditControllerTrait for GainController {
    unsafe fn setComponentState(&self, state: *mut IBStream) -> tresult {
        // SAFETY: host provides state stream pointer per VST3 contract.
        let Ok(normalized_gain) = (unsafe { load_normalized_gain(state) }) else {
            return kInvalidArgument;
        };
        self.gain_normalized.set(normalized_gain);
        kResultOk
    }

    unsafe fn setState(&self, state: *mut IBStream) -> tresult {
        // SAFETY: forwarded to shared state parser.
        unsafe { self.setComponentState(state) }
    }

    unsafe fn getState(&self, state: *mut IBStream) -> tresult {
        // SAFETY: host provides state stream pointer per VST3 contract.
        unsafe { store_normalized_gain(state, self.gain_normalized.get()) }
    }

    unsafe fn getParameterCount(&self) -> int32 {
        1
    }

    unsafe fn getParameterInfo(&self, param_index: int32, info: *mut ParameterInfo) -> tresult {
        if info.is_null() || param_index != 0 {
            return kInvalidArgument;
        }

        // SAFETY: pointer validated non-null above.
        let info = unsafe { &mut *info };
        info.id = PARAM_GAIN_ID;
        copy_wstring("Gain", &mut info.title);
        copy_wstring("Gain", &mut info.shortTitle);
        copy_wstring("x", &mut info.units);
        info.stepCount = 0;
        info.defaultNormalizedValue = gain_to_normalized(1.0);
        info.unitId = 0;
        info.flags = ParameterInfo_::ParameterFlags_::kCanAutomate as i32;

        kResultOk
    }

    unsafe fn getParamStringByValue(
        &self,
        id: ParamID,
        value_normalized: ParamValue,
        string: *mut String128,
    ) -> tresult {
        if id != PARAM_GAIN_ID || string.is_null() {
            return kInvalidArgument;
        }

        let display = format!("{:.2}x", normalized_to_gain(value_normalized));
        // SAFETY: pointer validated non-null above.
        let string = unsafe { &mut *string };
        copy_wstring(&display, string);

        kResultOk
    }

    unsafe fn getParamValueByString(
        &self,
        id: ParamID,
        string: *mut TChar,
        value_normalized: *mut ParamValue,
    ) -> tresult {
        if id != PARAM_GAIN_ID || value_normalized.is_null() {
            return kInvalidArgument;
        }

        // SAFETY: parser requires a valid host-provided TChar pointer.
        let Some(parsed_gain) = (unsafe { parse_tchar_f64(string) }) else {
            return kInvalidArgument;
        };

        // SAFETY: pointer validated non-null above.
        unsafe { *value_normalized = gain_to_normalized(parsed_gain) };
        kResultOk
    }

    unsafe fn normalizedParamToPlain(
        &self,
        id: ParamID,
        value_normalized: ParamValue,
    ) -> ParamValue {
        if id == PARAM_GAIN_ID {
            normalized_to_gain(value_normalized)
        } else {
            0.0
        }
    }

    unsafe fn plainParamToNormalized(&self, id: ParamID, plain_value: ParamValue) -> ParamValue {
        if id == PARAM_GAIN_ID {
            gain_to_normalized(plain_value)
        } else {
            0.0
        }
    }

    unsafe fn getParamNormalized(&self, id: ParamID) -> ParamValue {
        if id == PARAM_GAIN_ID {
            self.gain_normalized.get()
        } else {
            0.0
        }
    }

    unsafe fn setParamNormalized(&self, id: ParamID, value: ParamValue) -> tresult {
        if id != PARAM_GAIN_ID {
            return kInvalidArgument;
        }

        self.gain_normalized.set(value.clamp(0.0, 1.0));
        kResultOk
    }

    unsafe fn setComponentHandler(&self, _handler: *mut IComponentHandler) -> tresult {
        kResultOk
    }

    unsafe fn createView(&self, name: FIDString) -> *mut IPlugView {
        if name.is_null() {
            return ptr::null_mut();
        }

        // SAFETY: VST3 host passes a null-terminated view type string.
        let requested = unsafe { CStr::from_ptr(name) };
        // SAFETY: VST3 SDK constant points to a null-terminated static string.
        let editor = unsafe { CStr::from_ptr(ViewType::kEditor) };
        if requested.to_bytes() != editor.to_bytes() {
            return ptr::null_mut();
        }

        let Some(view) = ComWrapper::new(GainView::new()).to_com_ptr::<IPlugView>() else {
            return ptr::null_mut();
        };
        ComPtr::into_raw(view)
    }
}

/// Minimal host-parented view implementation.
struct GainView {
    /// Current view rectangle.
    rect: Cell<ViewRect>,
    /// Whether the view is currently attached to a host parent.
    attached: Cell<bool>,
}

impl GainView {
    /// Create a default view rectangle.
    fn new() -> Self {
        Self {
            rect: Cell::new(view_rect(420, 200)),
            attached: Cell::new(false),
        }
    }
}

impl Class for GainView {
    type Interfaces = (IPlugView,);
}

impl IPlugViewTrait for GainView {
    unsafe fn isPlatformTypeSupported(&self, r#type: FIDString) -> tresult {
        bool_to_tresult(platform_type_matches(r#type, default_platform_type()))
    }

    unsafe fn attached(&self, parent: *mut c_void, r#type: FIDString) -> tresult {
        if parent.is_null() {
            return kInvalidArgument;
        }
        if !platform_type_matches(r#type, default_platform_type()) {
            return kResultFalse;
        }

        self.attached.set(true);
        kResultOk
    }

    unsafe fn removed(&self) -> tresult {
        self.attached.set(false);
        kResultOk
    }

    unsafe fn onWheel(&self, _distance: f32) -> tresult {
        kResultFalse
    }

    unsafe fn onKeyDown(&self, _key: char16, _key_code: int16, _modifiers: int16) -> tresult {
        kResultFalse
    }

    unsafe fn onKeyUp(&self, _key: char16, _key_code: int16, _modifiers: int16) -> tresult {
        kResultFalse
    }

    unsafe fn getSize(&self, size: *mut ViewRect) -> tresult {
        if size.is_null() {
            return kInvalidArgument;
        }

        // SAFETY: pointer validated non-null above.
        unsafe { *size = self.rect.get() };
        kResultOk
    }

    unsafe fn onSize(&self, new_size: *mut ViewRect) -> tresult {
        if new_size.is_null() {
            return kInvalidArgument;
        }

        // SAFETY: pointer validated non-null above.
        self.rect.set(unsafe { *new_size });
        kResultOk
    }

    unsafe fn onFocus(&self, _state: TBool) -> tresult {
        kResultOk
    }

    unsafe fn setFrame(&self, _frame: *mut IPlugFrame) -> tresult {
        kResultOk
    }

    unsafe fn canResize(&self) -> tresult {
        kResultTrue
    }

    unsafe fn checkSizeConstraint(&self, rect: *mut ViewRect) -> tresult {
        if rect.is_null() {
            return kInvalidArgument;
        }

        // SAFETY: pointer validated non-null above.
        let rect = unsafe { &mut *rect };
        if rect.right - rect.left < 320 {
            rect.right = rect.left + 320;
        }
        if rect.bottom - rect.top < 160 {
            rect.bottom = rect.top + 160;
        }

        kResultOk
    }
}

/// Plugin factory exposed to the host.
#[derive(Default)]
struct Factory;

impl Class for Factory {
    type Interfaces = (IPluginFactory,);
}

impl IPluginFactoryTrait for Factory {
    unsafe fn getFactoryInfo(&self, info: *mut PFactoryInfo) -> tresult {
        if info.is_null() {
            return kInvalidArgument;
        }

        // SAFETY: pointer validated non-null above.
        let info = unsafe { &mut *info };
        copy_cstring("Toybox", &mut info.vendor);
        copy_cstring("https://github.com/PORTALSURFER/toybox", &mut info.url);
        copy_cstring("support@example.com", &mut info.email);
        info.flags = PFactoryInfo_::FactoryFlags_::kUnicode as int32;

        kResultOk
    }

    unsafe fn countClasses(&self) -> i32 {
        2
    }

    unsafe fn getClassInfo(&self, index: i32, info: *mut PClassInfo) -> tresult {
        if info.is_null() {
            return kInvalidArgument;
        }

        // SAFETY: pointer validated non-null above.
        let info = unsafe { &mut *info };
        match index {
            0 => {
                write_class_info_many(
                    info,
                    GainProcessor::CID,
                    CATEGORY_AUDIO_MODULE_CLASS,
                    PLUGIN_NAME,
                );
                kResultOk
            }
            1 => {
                write_class_info_many(
                    info,
                    GainController::CID,
                    CATEGORY_COMPONENT_CONTROLLER_CLASS,
                    PLUGIN_NAME,
                );
                kResultOk
            }
            _ => kInvalidArgument,
        }
    }

    unsafe fn createInstance(
        &self,
        cid: FIDString,
        iid: FIDString,
        obj: *mut *mut c_void,
    ) -> tresult {
        if cid.is_null() || iid.is_null() || obj.is_null() {
            return kInvalidArgument;
        }

        // SAFETY: caller provides valid pointers per VST3 ABI.
        let class_id = unsafe { *(cid as *const TUID) };
        let instance = match class_id {
            GainProcessor::CID => ComWrapper::new(GainProcessor::new()).to_com_ptr::<FUnknown>(),
            GainController::CID => ComWrapper::new(GainController::new()).to_com_ptr::<FUnknown>(),
            _ => None,
        };

        let Some(instance) = instance else {
            return kInvalidArgument;
        };

        let ptr = instance.as_ptr();
        // SAFETY: object pointer and requested interface are provided by the host.
        unsafe { ((*(*ptr).vtbl).queryInterface)(ptr, iid as *mut TUID, obj) }
    }
}

toybox::vst3_plugin_entry!(Factory);
