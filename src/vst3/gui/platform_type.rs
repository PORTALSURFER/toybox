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
