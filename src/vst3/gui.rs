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

/// GUI contract for reusable host-parented VST3 views backed by Patchbay windows.
#[cfg(feature = "gui")]
pub trait Vst3HostedGui {
    /// Attach the host-provided raw parent window handle.
    fn set_parent_raw(&mut self, parent: RawWindowHandle);

    /// Open the GUI for the already configured host parent.
    fn open(&mut self) -> bool;

    /// Close the GUI if it is currently open.
    fn close(&mut self);

    /// Return the latest known GUI logical size.
    fn last_size(&self) -> Option<(u32, u32)>;

    /// Request a GUI resize from the host.
    fn request_resize(&self, width: u32, height: u32);
}

/// Reusable VST3 `IPlugView` implementation for host-parented Patchbay GUIs.
#[cfg(feature = "gui")]
pub struct HostedVst3View<G: Vst3HostedGui> {
    rect: Cell<ViewRect>,
    attached: Cell<bool>,
    default_size: (i32, i32),
    gui: Mutex<G>,
}

#[cfg(feature = "gui")]
#[derive(Clone, Copy)]
enum ResizeAxis {
    Width,
    Height,
}

#[cfg(feature = "gui")]
impl<G: Vst3HostedGui> HostedVst3View<G> {
    /// Create a new host-parented view with default logical dimensions.
    pub fn new(gui: G, default_width: u32, default_height: u32) -> Self {
        let width = default_width.max(1) as i32;
        let height = default_height.max(1) as i32;
        Self {
            rect: Cell::new(view_rect(width, height)),
            attached: Cell::new(false),
            default_size: (width, height),
            gui: Mutex::new(gui),
        }
    }

    fn sync_rect_from_gui(&self) {
        let Ok(gui) = self.gui.lock() else {
            return;
        };
        if let Some((width, height)) = gui.last_size() {
            self.rect.set(view_rect(width as i32, height as i32));
        }
    }

    fn minimum_size(&self) -> (i32, i32) {
        self.default_size
    }

    fn uniform_ratio(&self) -> f32 {
        self.default_size.0 as f32 / self.default_size.1.max(1) as f32
    }

    fn dominant_resize_axis(&self, requested_width: i32, requested_height: i32) -> ResizeAxis {
        let current = self.rect.get();
        let current_width = (current.right - current.left).max(1);
        let current_height = (current.bottom - current.top).max(1);
        let width_delta = (requested_width - current_width).abs();
        let height_delta = (requested_height - current_height).abs();
        if width_delta >= height_delta {
            ResizeAxis::Width
        } else {
            ResizeAxis::Height
        }
    }

    fn constrain_uniform_size(
        &self,
        requested_width: i32,
        requested_height: i32,
        axis: ResizeAxis,
    ) -> (i32, i32) {
        let (min_width, min_height) = self.minimum_size();
        let ratio = self.uniform_ratio();
        let clamped_width = requested_width.max(min_width).max(1);
        let clamped_height = requested_height.max(min_height).max(1);

        match axis {
            ResizeAxis::Width => {
                let mut width = clamped_width;
                let mut height = ((width as f32) / ratio).round() as i32;
                if height < min_height {
                    height = min_height;
                    width = ((height as f32) * ratio).round() as i32;
                }
                (width.max(min_width).max(1), height.max(min_height).max(1))
            }
            ResizeAxis::Height => {
                let mut height = clamped_height;
                let mut width = ((height as f32) * ratio).round() as i32;
                if width < min_width {
                    width = min_width;
                    height = ((width as f32) / ratio).round() as i32;
                }
                (width.max(min_width).max(1), height.max(min_height).max(1))
            }
        }
    }
}

#[cfg(feature = "gui")]
impl<G: Vst3HostedGui> Class for HostedVst3View<G> {
    type Interfaces = (IPlugView,);
}

#[cfg(feature = "gui")]
impl<G: Vst3HostedGui> IPlugViewTrait for HostedVst3View<G> {
    unsafe fn isPlatformTypeSupported(&self, r#type: FIDString) -> tresult {
        #[cfg(target_os = "windows")]
        {
            bool_to_tresult(platform_type_matches(r#type, kPlatformTypeHWND))
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = r#type;
            kResultFalse
        }
    }

    unsafe fn attached(&self, parent: *mut std::ffi::c_void, r#type: FIDString) -> tresult {
        if parent.is_null() {
            return kInvalidArgument;
        }

        let Some(parent_handle) = (unsafe { parent_to_raw_window_handle(parent, r#type) }) else {
            return kResultFalse;
        };

        let Ok(mut gui) = self.gui.lock() else {
            return kResultFalse;
        };
        gui.set_parent_raw(parent_handle);
        if !gui.open() {
            return kResultFalse;
        }
        let (min_width, min_height) = self.minimum_size();
        let (requested_width, requested_height) = if let Some((width, height)) = gui.last_size() {
            (width as i32, height as i32)
        } else {
            (min_width, min_height)
        };
        let axis = self.dominant_resize_axis(requested_width, requested_height);
        let (constrained_width, constrained_height) =
            self.constrain_uniform_size(requested_width, requested_height, axis);
        if constrained_width != requested_width || constrained_height != requested_height {
            gui.request_resize(constrained_width as u32, constrained_height as u32);
        }
        self.rect
            .set(view_rect(constrained_width, constrained_height));

        self.attached.set(true);
        kResultOk
    }

    unsafe fn removed(&self) -> tresult {
        if let Ok(mut gui) = self.gui.lock() {
            gui.close();
        }
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
        self.sync_rect_from_gui();
        unsafe { *size = self.rect.get() };
        kResultOk
    }

    unsafe fn onSize(&self, new_size: *mut ViewRect) -> tresult {
        if new_size.is_null() {
            return kInvalidArgument;
        }

        let requested = unsafe { *new_size };
        let requested_width = (requested.right - requested.left).max(1);
        let requested_height = (requested.bottom - requested.top).max(1);
        let axis = self.dominant_resize_axis(requested_width, requested_height);
        let (constrained_width, constrained_height) =
            self.constrain_uniform_size(requested_width, requested_height, axis);
        let constrained = view_rect(constrained_width, constrained_height);
        unsafe { *new_size = constrained };

        if let Ok(gui) = self.gui.lock() {
            gui.request_resize(constrained_width as u32, constrained_height as u32);
        }
        self.rect.set(constrained);
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
        let rect = unsafe { &mut *rect };
        let requested_width = (rect.right - rect.left).max(1);
        let requested_height = (rect.bottom - rect.top).max(1);
        let axis = self.dominant_resize_axis(requested_width, requested_height);
        let (constrained_width, constrained_height) =
            self.constrain_uniform_size(requested_width, requested_height, axis);
        rect.right = rect.left + constrained_width;
        rect.bottom = rect.top + constrained_height;
        kResultOk
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

    struct MockHostedGui {
        last_size: Option<(u32, u32)>,
        resize_request: std::sync::Mutex<Option<(u32, u32)>>,
    }

    impl Vst3HostedGui for MockHostedGui {
        fn set_parent_raw(&mut self, _parent: RawWindowHandle) {}

        fn open(&mut self) -> bool {
            true
        }

        fn close(&mut self) {}

        fn last_size(&self) -> Option<(u32, u32)> {
            self.last_size
        }

        fn request_resize(&self, width: u32, height: u32) {
            if let Ok(mut slot) = self.resize_request.lock() {
                *slot = Some((width, height));
            }
        }
    }

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

    #[test]
    fn hosted_view_reports_default_size_before_attach() {
        let view = HostedVst3View::new(
            MockHostedGui {
                last_size: None,
                resize_request: std::sync::Mutex::new(None),
            },
            420,
            240,
        );
        let mut size = view_rect(0, 0);
        let result = unsafe { view.getSize(&mut size) };
        assert_eq!(result, kResultOk);
        assert_eq!(size.right - size.left, 420);
        assert_eq!(size.bottom - size.top, 240);
    }

    #[test]
    fn hosted_view_size_constraint_applies_minimum_default_size() {
        let view = HostedVst3View::new(
            MockHostedGui {
                last_size: Some((777, 333)),
                resize_request: std::sync::Mutex::new(None),
            },
            420,
            240,
        );
        let mut rect = view_rect(100, 100);
        let result = unsafe { view.checkSizeConstraint(&mut rect) };
        assert_eq!(result, kResultOk);
        assert_eq!(rect.right - rect.left, 420);
        assert_eq!(rect.bottom - rect.top, 240);
    }

    #[test]
    fn hosted_view_size_constraint_keeps_requested_size_when_larger_than_minimum() {
        let view = HostedVst3View::new(
            MockHostedGui {
                last_size: None,
                resize_request: std::sync::Mutex::new(None),
            },
            320,
            200,
        );
        let mut rect = view_rect(640, 400);
        let result = unsafe { view.checkSizeConstraint(&mut rect) };
        assert_eq!(result, kResultOk);
        assert_eq!(rect.right - rect.left, 640);
        assert_eq!(rect.bottom - rect.top, 400);
    }

    #[test]
    fn hosted_view_size_constraint_blocks_non_uniform_resize() {
        let view = HostedVst3View::new(
            MockHostedGui {
                last_size: None,
                resize_request: std::sync::Mutex::new(None),
            },
            320,
            200,
        );
        let mut rect = view_rect(500, 200);
        let result = unsafe { view.checkSizeConstraint(&mut rect) };
        assert_eq!(result, kResultOk);
        assert_eq!(rect.right - rect.left, 500);
        assert_eq!(rect.bottom - rect.top, 313);
    }

    #[test]
    fn hosted_view_attach_rejects_null_parent() {
        let view = HostedVst3View::new(
            MockHostedGui {
                last_size: None,
                resize_request: std::sync::Mutex::new(None),
            },
            320,
            240,
        );
        let result = unsafe { view.attached(std::ptr::null_mut(), kPlatformTypeHWND) };
        assert_eq!(result, kInvalidArgument);
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
