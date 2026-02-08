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
    let converted = unsafe { parent_to_raw_window_handle(std::ptr::null_mut(), kPlatformTypeHWND) };
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
fn hosted_view_on_size_applies_resize_to_hosted_gui() {
    let view = HostedVst3View::new(
        MockHostedGui {
            last_size: None,
            resize_request: std::sync::Mutex::new(None),
        },
        320,
        200,
    );
    let mut rect = view_rect(500, 200);
    let result = unsafe { view.onSize(&mut rect) };
    assert_eq!(result, kResultOk);
    assert_eq!(rect.right - rect.left, 500);
    assert_eq!(rect.bottom - rect.top, 313);

    let gui = view.gui.lock().expect("gui mutex should not be poisoned");
    let resize = gui
        .resize_request
        .lock()
        .expect("resize mutex should not be poisoned");
    assert_eq!(*resize, Some((500, 313)));
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
    let converted =
        unsafe { parent_to_raw_window_handle(parent, kPlatformTypeHWND) }.expect("expected handle");
    match converted {
        raw_window_handle::RawWindowHandle::Win32(handle) => {
            assert_eq!(handle.hwnd, parent);
        }
        _ => panic!("expected Win32 raw window handle"),
    }
}
