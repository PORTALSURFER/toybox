//! GUI helpers for VST3 plugin view implementations.

use std::ffi::{CStr, c_char};

#[cfg(feature = "gui")]
use raw_window_handle::RawWindowHandle;
#[cfg(feature = "gui")]
use std::cell::Cell;
#[cfg(feature = "gui")]
use std::sync::Mutex;
#[cfg(feature = "gui")]
use toybox_vst3_ffi::Class;
use toybox_vst3_ffi::Steinberg::Vst::TChar;
#[cfg(target_os = "macos")]
use toybox_vst3_ffi::Steinberg::kPlatformTypeNSView;
#[cfg(all(unix, not(target_os = "macos")))]
use toybox_vst3_ffi::Steinberg::kPlatformTypeX11EmbedWindowID;
use toybox_vst3_ffi::Steinberg::{
    FIDString, ViewRect, kPlatformTypeHWND, kResultFalse, kResultTrue, tresult,
};
#[cfg(feature = "gui")]
use toybox_vst3_ffi::Steinberg::{
    IPlugFrame, IPlugView, IPlugViewTrait, TBool, char16, int16, kInvalidArgument, kResultOk,
};

include!("string_conversion.rs");
include!("key_input.rs");
include!("platform_type.rs");
include!("hosted_view_types.rs");
include!("plug_view_impl.rs");
include!("view_rect_utils.rs");

#[cfg(all(test, feature = "gui"))]
mod tests;
