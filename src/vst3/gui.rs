//! GUI helpers for VST3 plugin view implementations.

use std::ffi::{CStr, c_char};

use toybox_vst3_ffi::Steinberg::Vst::TChar;
#[cfg(target_os = "macos")]
use toybox_vst3_ffi::Steinberg::kPlatformTypeNSView;
#[cfg(all(unix, not(target_os = "macos")))]
use toybox_vst3_ffi::Steinberg::kPlatformTypeX11EmbedWindowID;
use toybox_vst3_ffi::Steinberg::{
    FIDString, ViewRect, kPlatformTypeHWND, kResultFalse, kResultTrue, tresult,
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

/// Convert a VST3 parent pointer and platform id into a raw window handle.
///
/// This helper is intended for plugins that host Patchbay GUI windows in a
/// VST3 `IPlugView::attached` callback.
///
/// On Windows, this accepts `kPlatformTypeHWND` and maps the parent pointer to
/// `RawWindowHandle::Win32`. On other platforms this currently returns `None`.
///
/// # Safety
///
/// `parent` and `platform` must come directly from the host-provided VST3
/// `IPlugView::attached` callback and remain valid for handle construction.
#[cfg(feature = "gui")]
pub unsafe fn parent_to_raw_window_handle(
    parent: *mut std::ffi::c_void,
    platform: FIDString,
) -> Option<raw_window_handle::RawWindowHandle> {
    if parent.is_null() {
        return None;
    }

    #[cfg(target_os = "windows")]
    {
        if !platform_type_matches(platform, kPlatformTypeHWND) {
            return None;
        }

        let mut handle = raw_window_handle::Win32WindowHandle::empty();
        handle.hwnd = parent;
        handle.hinstance = std::ptr::null_mut();
        Some(raw_window_handle::RawWindowHandle::Win32(handle))
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = platform;
        None
    }
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

#[cfg(all(test, feature = "gui"))]
mod tests {
    use super::*;

    #[test]
    fn platform_type_matches_expected_constant() {
        assert!(platform_type_matches(kPlatformTypeHWND, kPlatformTypeHWND));
    }

    #[test]
    fn parent_handle_conversion_rejects_null_parent() {
        let converted =
            unsafe { parent_to_raw_window_handle(std::ptr::null_mut(), kPlatformTypeHWND) };
        assert!(converted.is_none());
    }

    #[test]
    fn parent_handle_conversion_rejects_unsupported_platform() {
        let bogus_platform = c"bogus".as_ptr();
        let parent = 1usize as *mut std::ffi::c_void;
        let converted = unsafe { parent_to_raw_window_handle(parent, bogus_platform) };
        assert!(converted.is_none());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn parent_handle_conversion_maps_hwnd() {
        let parent = 0x1234usize as *mut std::ffi::c_void;
        let converted = unsafe { parent_to_raw_window_handle(parent, kPlatformTypeHWND) }
            .expect("expected handle");
        match converted {
            raw_window_handle::RawWindowHandle::Win32(handle) => {
                assert_eq!(handle.hwnd, parent);
            }
            _ => panic!("expected Win32 raw window handle"),
        }
    }
}
