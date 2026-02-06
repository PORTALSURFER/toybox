//! GUI helpers for VST3 plugin view implementations.

use std::ffi::{CStr, c_char};

use toybox_vst3_ffi::Steinberg::Vst::TChar;
#[cfg(target_os = "macos")]
use toybox_vst3_ffi::Steinberg::kPlatformTypeNSView;
use toybox_vst3_ffi::Steinberg::{
    FIDString, ViewRect, kPlatformTypeHWND, kPlatformTypeX11EmbedWindowID, kResultFalse,
    kResultTrue, tresult,
};

/// Copy a UTF-8 Rust string into a fixed UTF-16 `TChar` destination buffer.
///
/// The destination is always null-terminated when non-empty.
pub fn copy_wstring(source: &str, destination: &mut [TChar]) {
    let mut written = 0usize;
    for (src, dst) in source.encode_utf16().zip(destination.iter_mut()) {
        *dst = src as TChar;
        written += 1;
    }

    if written < destination.len() {
        destination[written] = 0;
    } else if let Some(last) = destination.last_mut() {
        *last = 0;
    }
}

/// Compute the element length of a zero-terminated UTF-16 `TChar` string.
///
/// # Safety
///
/// `string` must point to a readable, zero-terminated `TChar` sequence.
pub unsafe fn tchar_len(string: *const TChar) -> usize {
    let mut len = 0;
    while unsafe { *string.add(len) } != 0 {
        len += 1;
    }
    len
}

/// Resolve the default VST3 view platform type for the current target OS.
pub const fn default_platform_type() -> FIDString {
    #[cfg(target_os = "windows")]
    {
        return kPlatformTypeHWND;
    }

    #[cfg(target_os = "macos")]
    {
        return kPlatformTypeNSView;
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        return kPlatformTypeX11EmbedWindowID;
    }

    #[allow(unreachable_code)]
    kPlatformTypeHWND
}

/// Compare a requested host platform string with an expected `FIDString`.
pub fn platform_type_matches(requested: *const c_char, expected: FIDString) -> bool {
    if requested.is_null() || expected.is_null() {
        return false;
    }

    // SAFETY: pointers are expected to be VST3-provided null-terminated strings.
    let requested = unsafe { CStr::from_ptr(requested) };
    // SAFETY: VST3 constants are static null-terminated strings.
    let expected = unsafe { CStr::from_ptr(expected) };

    requested.to_bytes() == expected.to_bytes()
}

/// Convert a boolean into a VST3 `tresult` success/failure code.
pub const fn bool_to_tresult(value: bool) -> tresult {
    if value { kResultTrue } else { kResultFalse }
}

/// Build a `ViewRect` for plugin views.
pub const fn view_rect(width: i32, height: i32) -> ViewRect {
    ViewRect {
        left: 0,
        top: 0,
        right: width,
        bottom: height,
    }
}
