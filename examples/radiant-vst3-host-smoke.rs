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
use radiant::widgets::WidgetKey;
#[cfg(all(target_os = "macos", feature = "radiant-vst3"))]
use toybox::vst3::gui::{RadiantVst3Editor, RadiantVst3HostedGui, Vst3HostedGui};

#[cfg(all(target_os = "macos", feature = "radiant-vst3"))]
/// Minimal declarative editor used to exercise the embedded renderer.
struct SmokeEditor {
    /// Gradient path presented by the hosted surface.
    plan: SurfacePaintPlan,
    /// Last logical size supplied by the host lifecycle.
    size: Option<(u32, u32)>,
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
        }
    }
}

#[cfg(all(target_os = "macos", feature = "radiant-vst3"))]
impl RadiantVst3Editor for SmokeEditor {
    fn resize(&mut self, width: u32, height: u32) {
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
        false
    }

    fn dispatch_key_press(&mut self, _key: WidgetKey) -> bool {
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

        let mut handle = toybox::raw_window_handle::AppKitWindowHandle::empty();
        handle.ns_view = parent.cast();
        let mut gui = RadiantVst3HostedGui::new(
            "ToyboxRadiantVst3EditorSmokeHost",
            SmokeEditor::new(),
            420,
            282,
        );
        gui.set_parent_raw(toybox::raw_window_handle::RawWindowHandle::AppKit(handle));
        assert!(gui.open(), "embedded Vello renderer should initialize");

        let subviews: *mut Object = msg_send![parent, subviews];
        let count: usize = msg_send![subviews, count];
        assert_eq!(count, 1, "hosted view should attach one child");
        let child: *mut Object = msg_send![subviews, objectAtIndex: 0_usize];
        let _: () = msg_send![child, display];

        gui.close();
        let subviews: *mut Object = msg_send![parent, subviews];
        let count: usize = msg_send![subviews, count];
        assert_eq!(count, 0, "hosted view should detach its child");
        let _: () = msg_send![parent, release];
    }
}

#[cfg(not(all(target_os = "macos", feature = "radiant-vst3")))]
fn main() {
    eprintln!("radiant-vst3-host-smoke requires macOS and --features radiant-vst3");
}
