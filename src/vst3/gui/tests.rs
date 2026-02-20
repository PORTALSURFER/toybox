use super::*;
use std::sync::Mutex;

struct MockHostedGui {
    last_size: Mutex<Option<(u32, u32)>>,
    resize_request: std::sync::Mutex<Option<(u32, u32)>>,
}

impl Vst3HostedGui for MockHostedGui {
    fn set_parent_raw(&mut self, _parent: RawWindowHandle) {}

    fn open(&mut self) -> bool {
        true
    }

    fn close(&mut self) {}

    fn last_size(&self) -> Option<(u32, u32)> {
        *self
            .last_size
            .lock()
            .expect("last_size mutex should not be poisoned")
    }

    fn request_resize(&self, width: u32, height: u32) {
        if let Ok(mut current) = self.last_size.lock() {
            *current = Some((width, height));
        }
        if let Ok(mut slot) = self.resize_request.lock() {
            *slot = Some((width, height));
        }
    }
}

#[test]
fn platform_type_matches_expected_constant() {
    assert!(unsafe { platform_type_matches(kPlatformTypeHWND, kPlatformTypeHWND) });
}

#[test]
fn parent_handle_conversion_rejects_null_parent() {
    let converted = unsafe { parent_to_raw_window_handle(std::ptr::null_mut(), kPlatformTypeHWND) };
    assert!(converted.is_none());
}

#[test]
fn parent_handle_conversion_rejects_unsupported_platform() {
    let bogus_platform = c"bogus".as_ptr();
    let parent = std::ptr::dangling_mut::<std::ffi::c_void>();
    let converted = unsafe { parent_to_raw_window_handle(parent, bogus_platform) };
    assert!(converted.is_none());
}

#[test]
fn vst3_key_translation_maps_navigation_virtual_keys() {
    use toybox_vst3_ffi::Steinberg::VirtualKeyCodes_::{
        KEY_DELETE, KEY_END, KEY_HOME, KEY_LEFT, KEY_RIGHT,
    };

    assert_eq!(
        vst3_key_down_to_input_char(0, KEY_LEFT as i16),
        Some('\u{1c}')
    );
    assert_eq!(
        vst3_key_down_to_input_char(0, KEY_RIGHT as i16),
        Some('\u{1d}')
    );
    assert_eq!(
        vst3_key_down_to_input_char(0, KEY_HOME as i16),
        Some('\u{1e}')
    );
    assert_eq!(
        vst3_key_down_to_input_char(0, KEY_END as i16),
        Some('\u{1f}')
    );
    assert_eq!(
        vst3_key_down_to_input_char(0, KEY_DELETE as i16),
        Some('\u{7f}')
    );
}

#[test]
fn vst3_key_translation_falls_back_to_unicode_key() {
    assert_eq!(vst3_key_down_to_input_char('A' as u16, 0), Some('A'));
    assert_eq!(vst3_key_down_to_input_char('ß' as u16, 0), Some('ß'));
}

#[test]
fn hosted_view_reports_default_size_before_attach() {
    let view = HostedVst3View::new(
        MockHostedGui {
            last_size: Mutex::new(None),
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
            last_size: Mutex::new(Some((777, 333))),
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
fn hosted_view_size_constraint_allows_sizes_below_minimum_when_disabled() {
    let view = HostedVst3View::new(
        MockHostedGui {
            last_size: Mutex::new(None),
            resize_request: std::sync::Mutex::new(None),
        },
        420,
        240,
    )
    .preserve_aspect_ratio(false)
    .enforce_minimum_size(false);
    let mut rect = view_rect(10, 10);
    let result = unsafe { view.checkSizeConstraint(&mut rect) };
    assert_eq!(result, kResultOk);
    assert_eq!(rect.right - rect.left, 10);
    assert_eq!(rect.bottom - rect.top, 10);

    let mut on_size = view_rect(10, 10);
    let result = unsafe { view.onSize(&mut on_size) };
    assert_eq!(result, kResultOk);
    assert_eq!(on_size.right - on_size.left, 10);
    assert_eq!(on_size.bottom - on_size.top, 10);
}

#[test]
fn hosted_view_size_constraint_keeps_requested_size_when_larger_than_minimum() {
    let view = HostedVst3View::new(
        MockHostedGui {
            last_size: Mutex::new(None),
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
            last_size: Mutex::new(None),
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
fn hosted_view_size_constraint_tracks_small_vertical_growth() {
    let view = HostedVst3View::new(
        MockHostedGui {
            last_size: Mutex::new(None),
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

    let mut next = view_rect(500, 314);
    let result = unsafe { view.onSize(&mut next) };
    assert_eq!(result, kResultOk);
    assert_eq!(next.right - next.left, 502);
    assert_eq!(next.bottom - next.top, 314);
}

#[test]
fn hosted_view_on_size_applies_resize_to_hosted_gui() {
    let view = HostedVst3View::new(
        MockHostedGui {
            last_size: Mutex::new(None),
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
            last_size: Mutex::new(None),
            resize_request: std::sync::Mutex::new(None),
        },
        320,
        240,
    );
    let result = unsafe { view.attached(std::ptr::null_mut(), kPlatformTypeHWND) };
    assert_eq!(result, kInvalidArgument);
}

#[test]
fn hosted_view_allows_direct_resize_when_aspect_ratio_disabled() {
    let view = HostedVst3View::new(
        MockHostedGui {
            last_size: Mutex::new(None),
            resize_request: std::sync::Mutex::new(None),
        },
        320,
        200,
    )
    .preserve_aspect_ratio(false);
    let mut rect = view_rect(500, 200);
    let result = unsafe { view.onSize(&mut rect) };
    assert_eq!(result, kResultOk);
    assert_eq!(rect.right - rect.left, 500);
    assert_eq!(rect.bottom - rect.top, 200);

    let gui = view.gui.lock().expect("gui mutex should not be poisoned");
    let resize = gui
        .resize_request
        .lock()
        .expect("resize mutex should not be poisoned");
    assert_eq!(*resize, Some((500, 200)));
}

#[test]
fn hosted_view_constraint_does_not_preserve_ratio_when_disabled() {
    let view = HostedVst3View::new(
        MockHostedGui {
            last_size: Mutex::new(None),
            resize_request: std::sync::Mutex::new(None),
        },
        320,
        200,
    )
    .preserve_aspect_ratio(false);
    let mut rect = view_rect(500, 200);
    let result = unsafe { view.checkSizeConstraint(&mut rect) };
    assert_eq!(result, kResultOk);
    assert_eq!(rect.right - rect.left, 500);
    assert_eq!(rect.bottom - rect.top, 200);
}

#[test]
fn hosted_view_host_resize_flow_simulates_vst3_growth_sequence() {
    let view = HostedVst3View::new(
        MockHostedGui {
            last_size: Mutex::new(None),
            resize_request: std::sync::Mutex::new(None),
        },
        2,
        2,
    )
    .preserve_aspect_ratio(false);

    let sizes = [(2, 2), (4, 4), (8, 8)];
    for (width, height) in sizes {
        let mut rect = view_rect(width, height);
        let constrained = unsafe { view.checkSizeConstraint(&mut rect) };
        assert_eq!(constrained, kResultOk);
        assert_eq!(rect.right - rect.left, width);
        assert_eq!(rect.bottom - rect.top, height);

        let on_size = unsafe { view.onSize(&mut rect) };
        assert_eq!(on_size, kResultOk);
        assert_eq!(rect.right - rect.left, width);
        assert_eq!(rect.bottom - rect.top, height);
    }

    let mut resolved = view_rect(0, 0);
    assert_eq!(unsafe { view.getSize(&mut resolved) }, kResultOk);
    assert_eq!(resolved.right - resolved.left, 8);
    assert_eq!(resolved.bottom - resolved.top, 8);

    let gui = view.gui.lock().expect("gui mutex should not be poisoned");
    let last_size = gui
        .last_size
        .lock()
        .expect("last_size mutex should not be poisoned");
    assert_eq!(*last_size, Some((8, 8)));
    let resize_request = gui
        .resize_request
        .lock()
        .expect("resize mutex should not be poisoned");
    assert_eq!(*resize_request, Some((8, 8)));
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
