//! Main-thread smoke host for Toybox's embedded GPUI AppKit child view.
//!
//! Run with `cargo run --example gpui-embedded-smoke --features gpui-gui` on
//! macOS. Set `TOYBOX_GPUI_SMOKE_SECONDS` for a visible interactive window.

#![allow(clippy::missing_docs_in_private_items)]

#[cfg(all(target_os = "macos", feature = "gpui-gui"))]
use std::cell::Cell;
#[cfg(all(target_os = "macos", feature = "gpui-gui"))]
use std::rc::Rc;
#[cfg(all(target_os = "macos", feature = "gpui-gui"))]
use std::time::{Duration, Instant};

#[cfg(all(target_os = "macos", feature = "gpui-gui"))]
use gpui::{
    App, AppContext, InteractiveElement, IntoElement, ParentElement, Render, Styled, Window, div,
    rgb,
};
#[cfg(all(target_os = "macos", feature = "gpui-gui"))]
use objc::runtime::{BOOL, NO, Object, YES};
#[cfg(all(target_os = "macos", feature = "gpui-gui"))]
use objc::{class, msg_send, sel, sel_impl};
#[cfg(all(target_os = "macos", feature = "gpui-gui"))]
use toybox::gpui_gui::GpuiHostedGui;

#[cfg(all(target_os = "macos", feature = "gpui-gui"))]
#[link(name = "AppKit", kind = "framework")]
unsafe extern "C" {}

#[cfg(all(target_os = "macos", feature = "gpui-gui"))]
#[repr(C)]
#[derive(Clone, Copy)]
struct NSPoint {
    x: f64,
    y: f64,
}

#[cfg(all(target_os = "macos", feature = "gpui-gui"))]
#[repr(C)]
#[derive(Clone, Copy)]
struct NSSize {
    width: f64,
    height: f64,
}

#[cfg(all(target_os = "macos", feature = "gpui-gui"))]
#[repr(C)]
#[derive(Clone, Copy)]
struct NSRect {
    origin: NSPoint,
    size: NSSize,
}

#[cfg(all(target_os = "macos", feature = "gpui-gui"))]
unsafe impl objc::Encode for NSPoint {
    fn encode() -> objc::Encoding {
        unsafe { objc::Encoding::from_str("{CGPoint=dd}") }
    }
}

#[cfg(all(target_os = "macos", feature = "gpui-gui"))]
unsafe impl objc::Encode for NSSize {
    fn encode() -> objc::Encoding {
        unsafe { objc::Encoding::from_str("{CGSize=dd}") }
    }
}

#[cfg(all(target_os = "macos", feature = "gpui-gui"))]
unsafe impl objc::Encode for NSRect {
    fn encode() -> objc::Encoding {
        unsafe { objc::Encoding::from_str("{CGRect={CGPoint=dd}{CGSize=dd}}") }
    }
}

#[cfg(all(target_os = "macos", feature = "gpui-gui"))]
struct SmokeView {
    key_count: Rc<Cell<u32>>,
}

#[cfg(all(target_os = "macos", feature = "gpui-gui"))]
impl Render for SmokeView {
    fn render(&mut self, _window: &mut Window, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let key_count = self.key_count.clone();
        div()
            .id("gpui-smoke")
            .size_full()
            .bg(rgb(0x224466))
            .on_key_down(move |_, _, _| key_count.set(key_count.get().saturating_add(1)))
            .child("Toybox GPUI")
    }
}

#[cfg(all(target_os = "macos", feature = "gpui-gui"))]
fn pump(app: *mut Object, seconds: f64, gui: &GpuiHostedGui) {
    let deadline = Instant::now() + Duration::from_secs_f64(seconds);
    while Instant::now() < deadline {
        unsafe {
            let date: *mut Object =
                msg_send![class!(NSDate), dateWithTimeIntervalSinceNow: 0.005_f64];
            let event: *mut Object = msg_send![
                app,
                nextEventMatchingMask: usize::MAX
                untilDate: date
                inMode: std::ptr::null_mut::<Object>()
                dequeue: YES
            ];
            if !event.is_null() {
                let _: () = msg_send![app, sendEvent: event];
            }
            let _: () = msg_send![app, updateWindows];
        }
        gui.pump();
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[cfg(all(target_os = "macos", feature = "gpui-gui"))]
fn main() {
    unsafe {
        let app: *mut Object = msg_send![class!(NSApplication), sharedApplication];
        assert!(!app.is_null(), "NSApplication should be available");
        let parent: *mut Object = msg_send![class!(NSView), new];
        let window: *mut Object = msg_send![class!(NSWindow), new];
        assert!(
            !parent.is_null() && !window.is_null(),
            "AppKit objects should allocate"
        );
        let frame = NSRect {
            origin: NSPoint { x: 0.0, y: 0.0 },
            size: NSSize {
                width: 208.0,
                height: 212.0,
            },
        };
        let _: () = msg_send![window, setFrame: frame display: YES];
        let _: () = msg_send![window, setContentView: parent];
        let _: () = msg_send![window, makeKeyAndOrderFront: std::ptr::null_mut::<Object>()];

        let keys = Rc::new(Cell::new(0));
        let factory_keys = keys.clone();
        let mut gui = GpuiHostedGui::new(
            "ToyboxGpuiEmbeddedSmoke",
            move |_window: &mut Window, cx: &mut App| {
                cx.new(|_| SmokeView {
                    key_count: factory_keys.clone(),
                })
                .into()
            },
            208,
            212,
        );
        let mut handle = toybox::raw_window_handle::AppKitWindowHandle::empty();
        handle.ns_view = parent.cast();
        gui.set_parent_raw(toybox::raw_window_handle::RawWindowHandle::AppKit(handle));
        assert!(gui.open(), "GPUI child should open in an AppKit parent");
        // Start with no editor focus. The click below must promote the child
        // through AppKit's real responder dispatch before the key event.
        let _: BOOL = msg_send![window, makeFirstResponder: std::ptr::null_mut::<Object>()];
        pump(app, 0.25, &gui);

        let subviews: *mut Object = msg_send![parent, subviews];
        let subview_count: usize = msg_send![subviews, count];
        assert_eq!(subview_count, 1, "child view should attach");
        let child: *mut Object = msg_send![subviews, objectAtIndex: 0_usize];
        let first_responder: *mut Object = msg_send![window, firstResponder];
        assert_ne!(
            first_responder, child,
            "smoke must begin without child focus"
        );
        let _: BOOL = msg_send![child, acceptsFirstResponder];

        // Exercise the real AppKit responder path. The event is sent through
        // NSApplication/window dispatch, never by calling the Rust callback.
        let window_number: isize = msg_send![window, windowNumber];
        let characters: *mut Object = msg_send![
            class!(NSString),
            stringWithUTF8String: c"a".as_ptr().cast::<i8>()
        ];
        let mouse_event: *mut Object = msg_send![
            class!(NSEvent),
            mouseEventWithType: 1_usize
            location: NSPoint { x: 20.0, y: 20.0 }
            modifierFlags: 0_u64
            timestamp: 0.0_f64
            windowNumber: window_number
            context: std::ptr::null_mut::<Object>()
            eventNumber: 0_isize
            clickCount: 1_isize
            pressure: 1.0_f64
        ];
        let _: () = msg_send![app, sendEvent: mouse_event];
        let first_responder: *mut Object = msg_send![window, firstResponder];
        assert_eq!(
            first_responder, child,
            "a native click must focus the GPUI child"
        );
        let key_event: *mut Object = msg_send![
            class!(NSEvent),
            keyEventWithType: 10_usize
            location: NSPoint { x: 20.0, y: 20.0 }
            modifierFlags: 0_u64
            timestamp: 0.0_f64
            windowNumber: window_number
            context: std::ptr::null_mut::<Object>()
            characters: characters
            charactersIgnoringModifiers: characters
            isARepeat: NO
            keyCode: 0_u16
        ];
        let _: () = msg_send![app, sendEvent: key_event];
        pump(app, 0.05, &gui);
        assert!(keys.get() >= 1, "AppKit key event should reach GPUI input");

        let captured = gui
            .capture_rgba()
            .expect("GPUI scene capture should complete");
        assert!(captured.0 > 0 && captured.1 > 0);
        assert_eq!(
            captured.2.len(),
            captured.0 as usize * captured.1 as usize * 4
        );

        let seconds = std::env::var("TOYBOX_GPUI_SMOKE_SECONDS")
            .ok()
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(0.0);
        if seconds > 0.0 {
            pump(app, seconds.min(600.0), &gui);
        }
        gui.close();
        let _: () = msg_send![window, orderOut: std::ptr::null_mut::<Object>()];
        let _: () = msg_send![parent, release];
        let _: () = msg_send![window, release];
    }
}

#[cfg(not(all(target_os = "macos", feature = "gpui-gui")))]
fn main() {
    eprintln!("gpui-embedded-smoke requires macOS and --features gpui-gui");
}
