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

struct RecordingHostedGui {
    events: Mutex<Vec<&'static str>>,
}

impl RecordingHostedGui {
    fn record(&self, event: &'static str) {
        self.events
            .lock()
            .expect("event mutex should not be poisoned")
            .push(event);
    }
}

impl Vst3HostedGui for RecordingHostedGui {
    fn set_parent_raw(&mut self, _parent: RawWindowHandle) {
        self.record("set-parent");
    }

    fn open(&mut self) -> bool {
        self.record("open");
        true
    }

    fn close(&mut self) {
        self.record("close");
    }

    fn last_size(&self) -> Option<(u32, u32)> {
        None
    }

    fn show(&self) -> bool {
        self.record("show");
        true
    }

    fn set_callback_keyboard_mode(&mut self, callback_only: bool) {
        self.record(if callback_only {
            "callback-only"
        } else {
            "native"
        });
    }

    fn request_resize(&self, _width: u32, _height: u32) {
        self.record("resize");
    }
}

struct ScaledHostedGui {
    resize_request: Mutex<Option<(u32, u32)>>,
}

impl Vst3HostedGui for ScaledHostedGui {
    fn set_parent_raw(&mut self, _parent: RawWindowHandle) {}

    fn open(&mut self) -> bool {
        true
    }

    fn close(&mut self) {}

    fn last_size(&self) -> Option<(u32, u32)> {
        None
    }

    fn host_size_from_logical(&self, width: u32, height: u32) -> (u32, u32) {
        (width.saturating_mul(2), height.saturating_mul(2))
    }

    fn logical_size_from_host(&self, width: u32, height: u32) -> (u32, u32) {
        ((width / 2).max(1), (height / 2).max(1))
    }

    fn request_resize(&self, width: u32, height: u32) {
        *self
            .resize_request
            .lock()
            .expect("resize mutex should not be poisoned") = Some((width, height));
    }
}

#[test]
fn platform_type_matches_expected_constant() {
    assert!(unsafe { platform_type_matches(kPlatformTypeHWND, kPlatformTypeHWND) });
}

#[cfg(target_os = "macos")]
#[test]
fn hosted_view_supports_nsview_platform_on_macos() {
    let view = HostedVst3View::new(
        MockHostedGui {
            last_size: Mutex::new(None),
            resize_request: std::sync::Mutex::new(None),
        },
        420,
        240,
    );

    assert_eq!(
        unsafe { view.isPlatformTypeSupported(kPlatformTypeNSView) },
        kResultTrue
    );
    assert_eq!(
        unsafe { view.isPlatformTypeSupported(kPlatformTypeHWND) },
        kResultFalse
    );
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

#[cfg(any(target_os = "macos", target_os = "windows"))]
#[test]
fn hosted_view_enables_callback_keyboard_before_open_and_shows_child() {
    let view = HostedVst3View::new(
        RecordingHostedGui {
            events: Mutex::new(Vec::new()),
        },
        420,
        240,
    );
    let parent = std::ptr::dangling_mut::<std::ffi::c_void>();
    #[cfg(target_os = "macos")]
    let platform = kPlatformTypeNSView;
    #[cfg(target_os = "windows")]
    let platform = kPlatformTypeHWND;

    assert_eq!(unsafe { view.attached(parent, platform) }, kResultOk);
    assert_eq!(
        *view
            .gui
            .lock()
            .expect("gui mutex should not be poisoned")
            .events
            .lock()
            .expect("event mutex should not be poisoned"),
        vec!["callback-only", "set-parent", "open", "show"]
    );

    assert_eq!(unsafe { view.removed() }, kResultOk);
    assert_eq!(
        *view
            .gui
            .lock()
            .expect("gui mutex should not be poisoned")
            .events
            .lock()
            .expect("event mutex should not be poisoned"),
        vec![
            "callback-only",
            "set-parent",
            "open",
            "show",
            "close",
            "native"
        ]
    );
}

#[test]
fn hosted_view_keeps_view_rect_in_host_pixels_when_gui_scales_logical_sizes() {
    let view = HostedVst3View::new(
        ScaledHostedGui {
            resize_request: Mutex::new(None),
        },
        420,
        240,
    );

    let mut size = view_rect(0, 0);
    assert_eq!(unsafe { view.getSize(&mut size) }, kResultOk);
    assert_eq!((size.right - size.left, size.bottom - size.top), (840, 480));

    let mut requested = view_rect(1_000, 600);
    assert_eq!(unsafe { view.onSize(&mut requested) }, kResultOk);
    assert_eq!(
        (
            requested.right - requested.left,
            requested.bottom - requested.top
        ),
        (1_000, 572)
    );
    let gui = view.gui.lock().expect("gui mutex should not be poisoned");
    assert_eq!(
        *gui.resize_request
            .lock()
            .expect("resize mutex should not be poisoned"),
        Some((1_000, 572))
    );
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
fn hosted_view_on_size_rejects_unrepresentable_origin_without_mutation() {
    let view = HostedVst3View::new(
        MockHostedGui {
            last_size: Mutex::new(None),
            resize_request: std::sync::Mutex::new(None),
        },
        320,
        200,
    );
    let mut rect = ViewRect {
        left: i32::MAX - 1,
        top: i32::MAX - 1,
        right: i32::MAX,
        bottom: i32::MAX,
    };
    let original = rect;
    let result = unsafe { view.onSize(&mut rect) };
    assert_eq!(result, kResultFalse);
    assert_eq!(rect.left, original.left);
    assert_eq!(rect.top, original.top);
    assert_eq!(rect.right, original.right);
    assert_eq!(rect.bottom, original.bottom);
    assert_eq!(rect.right, i32::MAX);
    assert_eq!(rect.bottom, i32::MAX);

    let gui = view.gui.lock().expect("gui mutex should not be poisoned");
    let resize = gui
        .resize_request
        .lock()
        .expect("resize mutex should not be poisoned");
    assert_eq!(*resize, None);
}

#[test]
fn hosted_view_constraint_rejects_unrepresentable_origin_without_mutation() {
    let view = HostedVst3View::new(
        MockHostedGui {
            last_size: Mutex::new(None),
            resize_request: std::sync::Mutex::new(None),
        },
        320,
        200,
    );
    let mut rect = ViewRect {
        left: i32::MAX - 1,
        top: i32::MAX - 1,
        right: i32::MAX,
        bottom: i32::MAX,
    };
    let original = rect;
    let result = unsafe { view.checkSizeConstraint(&mut rect) };
    assert_eq!(result, kResultFalse);
    assert_eq!(rect.left, original.left);
    assert_eq!(rect.top, original.top);
    assert_eq!(rect.right, original.right);
    assert_eq!(rect.bottom, original.bottom);
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

#[test]
fn hosted_view_explicit_bounds_allow_a_minimum_below_default() {
    let view = HostedVst3View::new(
        MockHostedGui {
            last_size: Mutex::new(None),
            resize_request: std::sync::Mutex::new(None),
        },
        420,
        240,
    )
    .with_size_bounds(300, 180, 960, 540);

    let mut rect = view_rect(250, 100);
    assert_eq!(unsafe { view.checkSizeConstraint(&mut rect) }, kResultOk);
    // The ratio-preserving pair is raised to the minimum height while staying
    // within the declared minimum width.
    assert_eq!(rect.right - rect.left, 315);
    assert_eq!(rect.bottom - rect.top, 180);
}

#[test]
fn hosted_view_explicit_bounds_keep_default_size_reporting() {
    let view = HostedVst3View::new(
        MockHostedGui {
            last_size: Mutex::new(None),
            resize_request: std::sync::Mutex::new(None),
        },
        420,
        240,
    )
    .with_size_bounds(300, 180, 960, 540);

    let mut size = view_rect(0, 0);
    assert_eq!(unsafe { view.getSize(&mut size) }, kResultOk);
    assert_eq!(size.right - size.left, 420);
    assert_eq!(size.bottom - size.top, 240);
}

#[test]
fn hosted_view_get_size_bounds_gui_reported_dimensions() {
    let view = HostedVst3View::new(
        MockHostedGui {
            last_size: Mutex::new(Some((2_000, 100))),
            resize_request: std::sync::Mutex::new(None),
        },
        420,
        240,
    )
    .with_size_bounds(300, 180, 960, 540);

    let mut size = view_rect(0, 0);
    assert_eq!(unsafe { view.getSize(&mut size) }, kResultOk);
    // The GUI-reported size is both above the maximum and off the default
    // aspect ratio, so getSize exposes the same bounded pair as host resize.
    assert_eq!(size.right - size.left, 945);
    assert_eq!(size.bottom - size.top, 540);
}

#[test]
fn hosted_view_explicit_bounds_clamp_maximum_size() {
    let view = HostedVst3View::new(
        MockHostedGui {
            last_size: Mutex::new(None),
            resize_request: std::sync::Mutex::new(None),
        },
        420,
        240,
    )
    .with_size_bounds(300, 180, 960, 540);

    let mut rect = view_rect(2_000, 1_600);
    assert_eq!(unsafe { view.onSize(&mut rect) }, kResultOk);
    assert_eq!(rect.right - rect.left, 945);
    assert_eq!(rect.bottom - rect.top, 540);
}

#[test]
fn hosted_view_explicit_bounds_normalize_off_aspect_requests() {
    let view = HostedVst3View::new(
        MockHostedGui {
            last_size: Mutex::new(None),
            resize_request: std::sync::Mutex::new(None),
        },
        420,
        240,
    )
    .with_size_bounds(300, 180, 960, 540);

    let mut rect = ViewRect {
        left: 10,
        top: 20,
        right: 610,
        bottom: 120,
    };
    assert_eq!(unsafe { view.checkSizeConstraint(&mut rect) }, kResultOk);
    assert_eq!(rect.right - rect.left, 600);
    assert_eq!(rect.bottom - rect.top, 343);
    assert_eq!((rect.left, rect.top), (10, 20));
    assert_eq!((rect.right, rect.bottom), (610, 363));
}

#[test]
fn hosted_view_legacy_constructor_keeps_unbounded_resize_behavior() {
    let view = HostedVst3View::new(
        MockHostedGui {
            last_size: Mutex::new(None),
            resize_request: std::sync::Mutex::new(None),
        },
        420,
        240,
    );

    let mut rect = view_rect(2_000, 100);
    assert_eq!(unsafe { view.checkSizeConstraint(&mut rect) }, kResultOk);
    assert_eq!(rect.right - rect.left, 2_000);
    assert_eq!(rect.bottom - rect.top, 1_143);
}

#[test]
fn hosted_view_explicit_bounds_normalize_invalid_zero_and_reversed_values() {
    let view = HostedVst3View::new(
        MockHostedGui {
            last_size: Mutex::new(None),
            resize_request: std::sync::Mutex::new(None),
        },
        420,
        240,
    )
    .with_size_bounds(1_000, 900, 0, 0);

    let mut rect = view_rect(2_000, 2_000);
    assert_eq!(unsafe { view.checkSizeConstraint(&mut rect) }, kResultOk);
    assert_eq!(rect.right - rect.left, 420);
    assert_eq!(rect.bottom - rect.top, 240);
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

#[cfg(target_os = "macos")]
#[test]
fn parent_handle_conversion_maps_ns_view() {
    let parent = std::ptr::dangling_mut::<std::ffi::c_void>();
    let converted = unsafe { parent_to_raw_window_handle(parent, kPlatformTypeNSView) }
        .expect("expected handle");
    match converted {
        raw_window_handle::RawWindowHandle::AppKit(handle) => {
            assert_eq!(handle.ns_view, parent);
        }
        _ => panic!("expected AppKit raw window handle"),
    }
}
