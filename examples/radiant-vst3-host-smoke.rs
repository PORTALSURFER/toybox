//! Main-thread smoke host for Toybox's Radiant-backed macOS VST3 view.

#[cfg(all(target_os = "macos", feature = "radiant-vst3"))]
use objc::runtime::Object;
#[cfg(all(target_os = "macos", feature = "radiant-vst3"))]
use objc::{class, msg_send, sel, sel_impl};
#[cfg(all(target_os = "macos", feature = "radiant-vst3"))]
use radiant::gui::types::{Point, Rect, Rgba8};
#[cfg(all(target_os = "macos", feature = "radiant-vst3"))]
use radiant::runtime::{
    Event, PaintBrush, PaintFillPath, PaintLinearGradient, PaintPath, PaintPathCommand,
    PaintPrimitive, SurfacePaintPlan,
};
#[cfg(all(target_os = "macos", feature = "radiant-vst3"))]
use radiant::theme::ThemeTokens;
#[cfg(all(target_os = "macos", feature = "radiant-vst3"))]
use radiant::widgets::{KeyboardModifiers, WidgetKey};
#[cfg(all(target_os = "macos", feature = "radiant-vst3"))]
use toybox::vst3::gui::{RadiantVst3Editor, RadiantVst3HostedGui, Vst3HostedGui};

#[cfg(all(target_os = "macos", feature = "radiant-vst3"))]
/// Minimal declarative editor used to exercise the embedded renderer.
struct SmokeEditor {
    /// Gradient path presented by the hosted surface.
    plan: SurfacePaintPlan,
    /// Last logical size supplied by the host lifecycle.
    size: Option<(u32, u32)>,
    /// Latest effective visibility observation from the native host.
    visible: std::rc::Rc<std::cell::Cell<Option<bool>>>,
    /// Optional failure injected by the subprocess regression.
    failure: Option<String>,
}

#[cfg(all(target_os = "macos", feature = "radiant-vst3"))]
impl SmokeEditor {
    /// Build a paint plan containing the path primitive Pump depends on.
    fn new() -> Self {
        let bounds = Rect::from_min_max(Point::new(0.0, 0.0), Point::new(420.0, 282.0));
        let fill = PaintFillPath::new(
            1,
            PaintPath::from([
                PaintPathCommand::MoveTo(Point::new(20.0, 20.0)),
                PaintPathCommand::LineTo(Point::new(400.0, 20.0)),
                PaintPathCommand::LineTo(Point::new(400.0, 262.0)),
                PaintPathCommand::LineTo(Point::new(20.0, 262.0)),
                PaintPathCommand::Close,
            ]),
            PaintBrush::linear_gradient(PaintLinearGradient::vertical(
                bounds,
                Rgba8::new(255, 96, 64, 160),
                Rgba8::new(255, 96, 64, 0),
            )),
        );
        Self {
            plan: SurfacePaintPlan {
                clear_color: ThemeTokens::default().bg_primary,
                primitives: vec![PaintPrimitive::FillPath(fill)],
            },
            size: None,
            visible: Default::default(),
            failure: None,
        }
    }
}

#[cfg(all(target_os = "macos", feature = "radiant-vst3"))]
impl RadiantVst3Editor for SmokeEditor {
    fn set_visible(&mut self, visible: bool) {
        self.visible.set(Some(visible));
    }

    fn resize(&mut self, width: u32, height: u32) {
        assert_ne!(
            self.failure.as_deref(),
            Some("resize"),
            "injected initialization panic"
        );
        self.size = Some((width, height));
    }

    fn dispatch_event(&mut self, _event: Event) {}

    fn paint_plan(&mut self) -> &SurfacePaintPlan {
        assert_eq!(
            self.size,
            Some((420, 282)),
            "editor must be sized before its first paint"
        );
        &self.plan
    }

    fn needs_realtime_redraw(&self) -> bool {
        assert_ne!(
            self.failure.as_deref(),
            Some("redraw"),
            "injected redraw panic"
        );
        false
    }

    fn dispatch_key_press(&mut self, _key: WidgetKey, _modifiers: KeyboardModifiers) -> bool {
        false
    }

    fn dispatch_character(&mut self, _character: char) -> bool {
        false
    }

    fn cancel_text_entry(&mut self) -> bool {
        false
    }
}

#[cfg(all(target_os = "macos", feature = "radiant-vst3"))]
fn main() {
    unsafe {
        let _: *mut Object = msg_send![class!(NSApplication), sharedApplication];
        let parent: *mut Object = msg_send![class!(NSView), new];
        assert!(!parent.is_null(), "NSView allocation should succeed");

        let window: *mut Object = msg_send![class!(NSWindow), new];
        let _: () = msg_send![window, setContentView: parent];
        let _: () = msg_send![window, orderFront: std::ptr::null_mut::<Object>()];
        let mut handle = toybox::raw_window_handle::AppKitWindowHandle::empty();
        handle.ns_view = parent.cast();
        let failure = std::env::args().nth(1);
        assert!(objc::runtime::Class::get("RawWindowMetalLayer").is_none());
        for iteration in 0..3 {
            let mut editor = SmokeEditor::new();
            if iteration == 0 {
                editor.failure = failure.clone();
            }
            let visible = std::rc::Rc::clone(&editor.visible);
            let mut gui =
                RadiantVst3HostedGui::new("ToyboxRadiantVst3EditorSmokeHost", editor, 420, 282);
            gui.set_parent_raw(toybox::raw_window_handle::RawWindowHandle::AppKit(handle));
            let fail_open = iteration == 0 && failure.as_deref() == Some("resize");
            assert_eq!(gui.open(), !fail_open, "initialization result");
            if !fail_open {
                let subviews: *mut Object = msg_send![parent, subviews];
                let count: usize = msg_send![subviews, count];
                assert_eq!(count, 1, "hosted view attaches one child");
                let child: *mut Object = msg_send![subviews, objectAtIndex: 0_usize];
                let layer: *mut Object = msg_send![child, layer];
                let sublayers: *mut Object = msg_send![layer, sublayers];
                let count: usize = msg_send![sublayers, count];
                assert_eq!(count, 1, "one owned Metal sublayer");
                let metal: *mut Object = msg_send![sublayers, objectAtIndex: 0_usize];
                let is_metal: objc::runtime::BOOL =
                    msg_send![metal, isKindOfClass: class!(CAMetalLayer)];
                assert_eq!(is_metal, objc::runtime::YES);
                let _: () = msg_send![child, display];
                assert!(gui.show());
                let _: () = msg_send![child, playheadRedrawTick: std::ptr::null_mut::<Object>()];
                if iteration == 0 && failure.is_none() {
                    assert_eq!(visible.get(), Some(true));
                    let _ = gui.on_focus(false);
                    assert_eq!(visible.get(), Some(true), "focus must not hide an editor");
                    let _: () = msg_send![parent, setHidden: objc::runtime::YES];
                    let _: () =
                        msg_send![child, playheadRedrawTick: std::ptr::null_mut::<Object>()];
                    assert_eq!(
                        visible.get(),
                        Some(false),
                        "ancestor hiding must be observed"
                    );
                    let _: () = msg_send![parent, setHidden: objc::runtime::NO];
                    let _: () =
                        msg_send![child, playheadRedrawTick: std::ptr::null_mut::<Object>()];
                    assert_eq!(visible.get(), Some(true));
                    let _: () = msg_send![window, orderOut: std::ptr::null_mut::<Object>()];
                    let _: () =
                        msg_send![child, playheadRedrawTick: std::ptr::null_mut::<Object>()];
                    assert_eq!(visible.get(), Some(false), "window hiding must be observed");
                    let _: () = msg_send![window, orderFront: std::ptr::null_mut::<Object>()];
                }
                let _: () = msg_send![child, playheadRedrawTick: std::ptr::null_mut::<Object>()];
                gui.request_resize(600, 400);
                gui.set_scale(2.0);
            }
            gui.close();
            if iteration == 0 && failure.is_none() {
                assert_eq!(visible.get(), Some(false));
            }
            let subviews: *mut Object = msg_send![parent, subviews];
            let count: usize = msg_send![subviews, count];
            assert_eq!(count, 0, "close rolls back all child views");
            if iteration == 0 && failure.is_some() {
                assert!(!gui.open(), "failed editor is quarantined");
            }
            assert!(objc::runtime::Class::get("RawWindowMetalLayer").is_none());
        }
        let _: () = msg_send![parent, release];
        let _: () = msg_send![window, release];
    }
}

#[cfg(not(all(target_os = "macos", feature = "radiant-vst3")))]
fn main() {
    eprintln!("radiant-vst3-host-smoke requires macOS and --features radiant-vst3");
}
