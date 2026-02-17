//! Controller component implementation for the minimal VST3 example.

use std::cell::Cell;
use std::ffi::CStr;
use std::ptr;

use toybox::vst3::prelude::Steinberg::*;
use toybox::vst3::prelude::*;

use crate::constants::PARAM_GAIN_ID;
use crate::params::{gain_to_normalized, normalized_to_gain};
use crate::state_io::{load_normalized_gain, store_normalized_gain};
use crate::view::GainView;

/// Minimal controller implementation handling parameter and GUI state.
pub(crate) struct GainController {
    /// Current normalized gain value.
    gain_normalized: Cell<f64>,
}

impl GainController {
    /// Unique class identifier for the edit controller.
    pub(crate) const CID: TUID = uid(0xA50AF46B, 0xCC7B43FF, 0x8CF8A3E0, 0xA2C0EE33);

    /// Create a controller initialized to unity gain.
    pub(crate) fn new() -> Self {
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
        info.flags = ParameterInfo_::ParameterFlags_::kCanAutomate;

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
