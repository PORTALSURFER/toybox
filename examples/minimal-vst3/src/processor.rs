//! Processor component implementation for the minimal VST3 example.

use std::sync::atomic::{AtomicU64, Ordering};

use toybox::vst3::prelude::Steinberg::*;
use toybox::vst3::prelude::*;

use crate::constants::PARAM_GAIN_ID;
use crate::controller::GainController;
use crate::params::{gain_to_normalized, normalized_to_gain};
use crate::state_io::{load_normalized_gain, store_normalized_gain};

/// Minimal gain processor implementation.
pub(crate) struct GainProcessor {
    /// Normalized gain value represented as f64 bits for atomic access.
    gain_normalized_bits: AtomicU64,
}

impl GainProcessor {
    /// Unique class identifier for the processor component.
    pub(crate) const CID: TUID = uid(0xEE96D1A4, 0x53B140DB, 0x8F52EC7B, 0x09C55BA8);

    /// Create a processor with default gain of 1.0x.
    pub(crate) fn new() -> Self {
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
        bus.flags = BusInfo_::BusFlags_::kDefaultActive;

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
