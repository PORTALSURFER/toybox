//! Main-thread smoke host for Toybox's embedded GPUI AppKit child view.
//!
//! Run with `cargo run --example gpui-embedded-smoke --features gpui-gui` on
//! macOS. Set `TOYBOX_GPUI_SMOKE_SECONDS` for a visible interactive window.

#![allow(clippy::missing_docs_in_private_items)]

#[cfg(all(target_os = "macos", feature = "gpui-gui"))]
use std::cell::Cell;
#[cfg(all(target_os = "macos", feature = "gpui-gui"))]
use std::ffi::{CStr, CString};
#[cfg(all(target_os = "macos", feature = "gpui-gui"))]
use std::rc::Rc;
#[cfg(all(target_os = "macos", feature = "gpui-gui"))]
use std::time::{Duration, Instant};

#[cfg(all(target_os = "macos", feature = "gpui-gui"))]
use gpui::{
    App, AppContext, ClipboardItem, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, div, rgb,
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
#[link(name = "Foundation", kind = "framework")]
unsafe extern "C" {
    static NSDefaultRunLoopMode: *mut Object;
}

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
struct ClipboardItemBackup {
    entries: Vec<(String, Vec<u8>)>,
}

#[cfg(all(target_os = "macos", feature = "gpui-gui"))]
struct ClipboardRestore {
    pasteboard: *mut Object,
    items: Vec<ClipboardItemBackup>,
}

#[cfg(all(target_os = "macos", feature = "gpui-gui"))]
impl ClipboardRestore {
    unsafe fn capture() -> Option<Self> {
        let pasteboard: *mut Object = msg_send![class!(NSPasteboard), generalPasteboard];
        if pasteboard.is_null() {
            return None;
        }
        let native_items: *mut Object = msg_send![pasteboard, pasteboardItems];
        let count: usize = if native_items.is_null() {
            0
        } else {
            msg_send![native_items, count]
        };
        let mut items = Vec::with_capacity(count);
        for index in 0..count {
            let item: *mut Object = msg_send![native_items, objectAtIndex: index];
            let types: *mut Object = msg_send![item, types];
            let type_count: usize = msg_send![types, count];
            let mut entries = Vec::with_capacity(type_count);
            for type_index in 0..type_count {
                let kind: *mut Object = msg_send![types, objectAtIndex: type_index];
                let kind_ptr: *const i8 = msg_send![kind, UTF8String];
                if kind_ptr.is_null() {
                    continue;
                }
                let data: *mut Object = msg_send![item, dataForType: kind];
                let Ok(kind) = unsafe { CStr::from_ptr(kind_ptr) }.to_str() else {
                    continue;
                };
                let length: usize = if data.is_null() {
                    0
                } else {
                    msg_send![data, length]
                };
                let bytes: *const u8 = if data.is_null() || length == 0 {
                    std::ptr::null()
                } else {
                    msg_send![data, bytes]
                };
                let value = if bytes.is_null() {
                    Vec::new()
                } else {
                    unsafe { std::slice::from_raw_parts(bytes, length) }.to_vec()
                };
                entries.push((kind.to_owned(), value));
            }
            items.push(ClipboardItemBackup { entries });
        }
        Some(Self { pasteboard, items })
    }
}

#[cfg(all(target_os = "macos", feature = "gpui-gui"))]
impl Drop for ClipboardRestore {
    fn drop(&mut self) {
        unsafe {
            let _: isize = msg_send![self.pasteboard, clearContents];
            let mut native_items = Vec::with_capacity(self.items.len());
            for item_backup in &self.items {
                let item: *mut Object = msg_send![class!(NSPasteboardItem), new];
                if item.is_null() {
                    continue;
                }
                for (kind, bytes) in &item_backup.entries {
                    let Ok(kind) = CString::new(kind.as_bytes()) else {
                        continue;
                    };
                    let kind: *mut Object = msg_send![
                        class!(NSString),
                        stringWithUTF8String: kind.as_ptr()
                    ];
                    let data: *mut Object = msg_send![
                        class!(NSData),
                        dataWithBytes: bytes.as_ptr()
                        length: bytes.len()
                    ];
                    let _: BOOL = msg_send![item, setData: data forType: kind];
                }
                native_items.push(item);
            }
            if !native_items.is_empty() {
                let array: *mut Object = msg_send![
                    class!(NSArray),
                    arrayWithObjects: native_items.as_ptr()
                    count: native_items.len()
                ];
                let _: BOOL = msg_send![self.pasteboard, writeObjects: array];
            }
            for item in native_items {
                let _: () = msg_send![item, release];
            }
        }
    }
}

#[cfg(all(target_os = "macos", feature = "gpui-gui"))]
impl Render for SmokeView {
    fn render(&mut self, _window: &mut Window, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let key_count = self.key_count.clone();
        div()
            .id("gpui-smoke")
            .size_full()
            .focusable()
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
                inMode: NSDefaultRunLoopMode
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
        eprintln!("gpui smoke: app ready");
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
        eprintln!("gpui smoke: window ready");

        // The guard keeps every existing pasteboard item/type alive while the
        // GPUI platform path performs a text roundtrip, then restores the
        // user's pasteboard after the smoke process exits.
        let _clipboard_restore =
            ClipboardRestore::capture().expect("AppKit general pasteboard should be available");
        let clipboard_roundtrip = Rc::new(Cell::new(false));
        let factory_clipboard = clipboard_roundtrip.clone();

        let keys = Rc::new(Cell::new(0));
        let factory_keys = keys.clone();
        let mut gui = GpuiHostedGui::new(
            "ToyboxGpuiEmbeddedSmoke",
            move |_window: &mut Window, cx: &mut App| {
                let fixture = "toybox-gpui-clipboard-roundtrip".to_string();
                cx.write_to_clipboard(ClipboardItem::new_string(fixture.clone()));
                assert_eq!(
                    cx.read_from_clipboard().and_then(|item| item.text()),
                    Some(fixture)
                );
                factory_clipboard.set(true);
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
        eprintln!("gpui smoke: child opened");
        assert!(
            clipboard_roundtrip.get(),
            "GPUI platform clipboard should roundtrip text"
        );
        eprintln!("gpui smoke: clipboard roundtrip complete");
        // Start with no editor focus. The click below must promote the child
        // through AppKit's real responder dispatch before the key event.
        let _: BOOL = msg_send![window, makeFirstResponder: std::ptr::null_mut::<Object>()];
        pump(app, 0.25, &gui);
        eprintln!("gpui smoke: initial pump complete");

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
        eprintln!("gpui smoke: click dispatched");
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
        eprintln!("gpui smoke: key dispatched count={}", keys.get());
        assert!(keys.get() >= 1, "AppKit key event should reach GPUI input");

        let captured = gui
            .capture_rgba()
            .expect("GPUI scene capture should complete");
        eprintln!("gpui smoke: capture complete {}x{}", captured.0, captured.1);
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
        eprintln!("gpui smoke: child closed");
        let _: () = msg_send![window, orderOut: std::ptr::null_mut::<Object>()];
        let _: () = msg_send![parent, release];
        let _: () = msg_send![window, release];
    }
}

#[cfg(not(all(target_os = "macos", feature = "gpui-gui")))]
fn main() {
    eprintln!("gpui-embedded-smoke requires macOS and --features gpui-gui");
}
