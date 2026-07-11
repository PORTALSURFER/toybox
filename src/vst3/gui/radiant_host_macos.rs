//! Radiant-backed AppKit VST3 editor for macOS hosts.
#![allow(
    clippy::missing_docs_in_private_items,
    unexpected_cfgs,
    unsafe_op_in_unsafe_fn
)]

use std::cell::Cell;
use std::ffi::{CStr, c_void};
use std::ptr::{self, NonNull};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use objc::declare::ClassDecl;
use objc::runtime::{BOOL, Class, NO, Object, Sel, YES};
use objc::{Encode, Encoding, class, msg_send, sel, sel_impl};
use radiant::gui::types::{Point, Vector2};
use radiant::runtime::{
    EmbeddedVelloRenderer, EmbeddedVelloSurfaceHandle, Event, NativeTextOptions, Renderer,
    SurfacePaintPlan,
};
use radiant::theme::DpiScale;
use radiant::widgets::{PointerButton, PointerModifiers, WidgetKey};
use raw_window_handle_06::{
    AppKitDisplayHandle, AppKitWindowHandle, RawDisplayHandle as RawDisplayHandle06,
    RawWindowHandle as RawWindowHandle06,
};

use super::{Vst3HostedGui, vst3_key_down_to_input_char};

const NSEVENT_MODIFIER_FLAG_SHIFT: u64 = 1 << 17;
const NSEVENT_MODIFIER_FLAG_OPTION: u64 = 1 << 19;
const NSEVENT_MODIFIER_FLAG_COMMAND: u64 = 1 << 20;
const NSTRACKING_MOUSE_ENTERED_AND_EXITED: usize = 0x01;
const NSTRACKING_MOUSE_MOVED: usize = 0x02;
const NSTRACKING_ACTIVE_ALWAYS: usize = 0x80;
const NSTRACKING_IN_VISIBLE_RECT: usize = 0x200;
const NSTRACKING_ENABLED_DURING_MOUSE_DRAG: usize = 0x400;
const PLAYHEAD_REDRAW_INTERVAL: Duration = Duration::from_millis(33);

type CFRunLoopRef = *mut c_void;

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFRunLoopGetMain() -> CFRunLoopRef;
    fn CFRunLoopWakeUp(rl: CFRunLoopRef);
}

#[repr(C)]
#[derive(Clone, Copy)]
struct NSPoint {
    x: f64,
    y: f64,
}

unsafe impl Encode for NSPoint {
    fn encode() -> Encoding {
        unsafe { Encoding::from_str("{CGPoint=dd}") }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct NSSize {
    width: f64,
    height: f64,
}

unsafe impl Encode for NSSize {
    fn encode() -> Encoding {
        unsafe { Encoding::from_str("{CGSize=dd}") }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct NSRect {
    origin: NSPoint,
    size: NSSize,
}

unsafe impl Encode for NSRect {
    fn encode() -> Encoding {
        unsafe { Encoding::from_str("{CGRect={CGPoint=dd}{CGSize=dd}}") }
    }
}

struct RedrawDriver {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

/// Radiant editor contract consumed by Toybox's hosted VST3 view.
pub trait RadiantVst3Editor: 'static {
    /// Resize the declarative editor to a new logical host size.
    fn resize(&mut self, width: u32, height: u32);

    /// Dispatch one backend-neutral Radiant input event.
    fn dispatch_event(&mut self, event: Event);

    /// Build or borrow the latest Radiant paint plan.
    fn paint_plan(&mut self) -> &SurfacePaintPlan;

    /// Return whether transport or animation state needs periodic redraws.
    fn needs_realtime_redraw(&self) -> bool;

    /// Dispatch a semantic key press to the active Radiant editor.
    fn dispatch_key_press(&mut self, key: WidgetKey) -> bool;

    /// Dispatch one text character to the active Radiant editor.
    fn dispatch_character(&mut self, character: char) -> bool;

    /// Cancel the active Radiant text or numeric entry, if any.
    fn cancel_text_entry(&mut self) -> bool;
}

/// Toybox-owned macOS VST3 host view rendered by Radiant's embedded Vello backend.
pub struct RadiantVst3HostedGui {
    parent: Option<NonNull<c_void>>,
    root_view: Option<NonNull<Object>>,
    size: Cell<Option<(u32, u32)>>,
    default_size: (u32, u32),
    class_name: &'static str,
    editor: Option<Box<dyn RadiantVst3Editor>>,
    text_options: NativeTextOptions,
}

impl RadiantVst3HostedGui {
    /// Create a reusable hosted view for one plugin's Radiant editor.
    pub fn new(
        class_name: &'static str,
        editor: impl RadiantVst3Editor,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            parent: None,
            root_view: None,
            size: Cell::new(None),
            default_size: (width.max(1), height.max(1)),
            class_name,
            editor: Some(Box::new(editor)),
            text_options: NativeTextOptions::default(),
        }
    }

    /// Configure portable embedded fonts or host-approved font paths for Radiant text rendering.
    pub fn with_text_options(mut self, text_options: NativeTextOptions) -> Self {
        self.text_options = text_options;
        self
    }

    fn set_parent(&mut self, parent: raw_window_handle::RawWindowHandle) {
        if let raw_window_handle::RawWindowHandle::AppKit(handle) = parent {
            self.parent = NonNull::new(handle.ns_view);
        }
    }

    fn open_view(&mut self) -> bool {
        if self.root_view.is_some() {
            return true;
        }
        let Some(parent) = self.parent else {
            return false;
        };
        let Some(editor) = self.editor.take() else {
            return false;
        };
        let (width, height) = self.initial_open_size();
        let root_view = match unsafe {
            create_editor_view(
                parent,
                self.class_name,
                editor,
                width,
                height,
                &self.text_options,
            )
        } {
            Ok(root_view) => root_view,
            Err(editor) => {
                self.editor = Some(editor);
                return false;
            }
        };
        self.root_view = Some(root_view);
        self.size.set(Some((width, height)));
        true
    }

    fn close_view(&mut self) {
        unsafe {
            if let Some(root_view) = self.root_view.take() {
                stop_redraw_driver(root_view.as_ptr());
                drop_renderer(root_view.as_ptr());
                self.editor = take_runtime(root_view.as_ptr());
                let view = root_view.as_ptr();
                let _: () = msg_send![view, removeFromSuperview];
                let _: () = msg_send![view, release];
            }
        }
    }

    fn hosted_size(&self) -> Option<(u32, u32)> {
        self.size.get().or(Some(self.default_size))
    }

    fn initial_open_size(&self) -> (u32, u32) {
        self.hosted_size().unwrap_or(self.default_size)
    }

    /// Apply a host-driven resize to the hosted child view.
    fn resize_view(&self, width: u32, height: u32) {
        let width = width.max(1);
        let height = height.max(1);
        self.size.set(Some((width, height)));
        let Some(root_view) = self.root_view else {
            return;
        };
        unsafe {
            set_frame(root_view.as_ptr(), 0.0, 0.0, width as f64, height as f64);
            if let Some(runtime) = runtime_mut(root_view.as_ptr()) {
                runtime.resize(width, height);
            }
            resize_renderer(root_view.as_ptr(), width, height);
            let _: () = msg_send![root_view.as_ptr(), setNeedsDisplay: YES];
        }
    }
}

impl Drop for RadiantVst3HostedGui {
    fn drop(&mut self) {
        self.close_view();
    }
}

impl Vst3HostedGui for RadiantVst3HostedGui {
    fn set_parent_raw(&mut self, parent: raw_window_handle::RawWindowHandle) {
        self.set_parent(parent);
    }

    fn open(&mut self) -> bool {
        self.open_view()
    }

    fn close(&mut self) {
        self.close_view();
    }

    fn last_size(&self) -> Option<(u32, u32)> {
        self.hosted_size()
    }

    fn request_resize(&self, width: u32, height: u32) {
        self.resize_view(width, height);
    }

    fn on_key_down(&self, key: u16, key_code: i16, modifiers: i16) -> bool {
        let Some(root_view) = self.root_view else {
            return false;
        };
        unsafe {
            let Some(runtime) = runtime_mut(root_view.as_ptr()) else {
                return false;
            };
            let handled = dispatch_vst3_key_down(runtime, key, key_code, modifiers);
            if handled {
                let _: () = msg_send![root_view.as_ptr(), setNeedsDisplay: YES];
            }
            handled
        }
    }
}

unsafe fn create_editor_view(
    parent: NonNull<c_void>,
    class_name: &'static str,
    mut editor: Box<dyn RadiantVst3Editor>,
    width: u32,
    height: u32,
    text_options: &NativeTextOptions,
) -> Result<NonNull<Object>, Box<dyn RadiantVst3Editor>> {
    let Some(root_view) = new_radiant_view(class_name, width, height) else {
        return Err(editor);
    };
    let parent = parent.as_ptr().cast::<Object>();
    let _: () = msg_send![parent, addSubview: root_view.as_ptr()];
    let _: () = msg_send![root_view.as_ptr(), setWantsLayer: YES];
    let Some(renderer) = embedded_renderer_for_view(root_view, width, height, text_options) else {
        let _: () = msg_send![root_view.as_ptr(), removeFromSuperview];
        let _: () = msg_send![root_view.as_ptr(), release];
        return Err(editor);
    };
    editor.resize(width, height);
    (*root_view.as_ptr()).set_ivar("runtime", Box::into_raw(Box::new(editor)) as usize);
    (*root_view.as_ptr()).set_ivar("renderer", Box::into_raw(Box::new(renderer)) as usize);
    (*root_view.as_ptr()).set_ivar(
        "redraw_driver",
        start_redraw_driver(root_view.as_ptr()) as usize,
    );
    let _: () = msg_send![root_view.as_ptr(), updateTrackingAreas];
    Ok(root_view)
}

unsafe fn new_radiant_view(
    class_name: &'static str,
    width: u32,
    height: u32,
) -> Option<NonNull<Object>> {
    let view: *mut Object = msg_send![editor_view_class(class_name), alloc];
    let view: *mut Object =
        msg_send![view, initWithFrame: ns_rect(0.0, 0.0, width as f64, height as f64)];
    let view = NonNull::new(view)?;
    (*view.as_ptr()).set_ivar("runtime", 0_usize);
    (*view.as_ptr()).set_ivar("renderer", 0_usize);
    (*view.as_ptr()).set_ivar("redraw_driver", 0_usize);
    Some(view)
}

unsafe fn embedded_renderer_for_view(
    view: NonNull<Object>,
    width: u32,
    height: u32,
    text_options: &NativeTextOptions,
) -> Option<EmbeddedVelloRenderer> {
    let window_handle = AppKitWindowHandle::new(view.cast());
    let display_handle = AppKitDisplayHandle::new();
    let handle = EmbeddedVelloSurfaceHandle::from_raw(
        RawDisplayHandle06::AppKit(display_handle),
        RawWindowHandle06::AppKit(window_handle),
    );
    EmbeddedVelloRenderer::new_with_text_options(
        handle,
        Vector2::new(width.max(1) as f32, height.max(1) as f32),
        view_dpi_scale(view.as_ptr()),
        text_options,
    )
    .ok()
}

unsafe fn view_dpi_scale(view: *mut Object) -> DpiScale {
    let window: *mut Object = msg_send![view, window];
    if window.is_null() {
        return DpiScale::ONE;
    }
    let factor: f64 = msg_send![window, backingScaleFactor];
    DpiScale::new(factor)
}

unsafe fn render_paint_plan(
    renderer: &mut EmbeddedVelloRenderer,
    plan: &SurfacePaintPlan,
    view: *mut Object,
    bounds: NSRect,
) {
    renderer.resize(
        Vector2::new(
            bounds.size.width.max(1.0) as f32,
            bounds.size.height.max(1.0) as f32,
        ),
        view_dpi_scale(view),
    );
    let _ = renderer.render(plan);
}

unsafe fn resize_renderer(view: *mut Object, width: u32, height: u32) {
    let scale = view_dpi_scale(view);
    if let Some(renderer) = renderer_mut(view) {
        renderer.resize(
            Vector2::new(width.max(1) as f32, height.max(1) as f32),
            scale,
        );
    }
}

unsafe fn set_frame(view: *mut Object, x: f64, y: f64, width: f64, height: f64) {
    let _: () = msg_send![view, setFrame: ns_rect(x, y, width.max(1.0), height.max(1.0))];
}

fn ns_rect(x: f64, y: f64, width: f64, height: f64) -> NSRect {
    NSRect {
        origin: NSPoint { x, y },
        size: NSSize { width, height },
    }
}

fn editor_view_class(class_name: &'static str) -> *const Class {
    if let Some(existing) = Class::get(class_name) {
        return existing;
    }
    {
        let superclass = class!(NSView);
        let mut decl = ClassDecl::new(class_name, superclass).expect("unique class name");
        decl.add_ivar::<usize>("runtime");
        decl.add_ivar::<usize>("renderer");
        decl.add_ivar::<usize>("tracking_area");
        decl.add_ivar::<usize>("redraw_driver");
        unsafe {
            decl.add_method(
                sel!(drawRect:),
                draw_rect as extern "C" fn(&Object, Sel, NSRect),
            );
            decl.add_method(
                sel!(updateTrackingAreas),
                update_tracking_areas as extern "C" fn(&Object, Sel),
            );
            decl.add_method(
                sel!(mouseMoved:),
                mouse_moved as extern "C" fn(&Object, Sel, *mut Object),
            );
            decl.add_method(
                sel!(mouseExited:),
                mouse_exited as extern "C" fn(&Object, Sel, *mut Object),
            );
            decl.add_method(
                sel!(mouseDown:),
                mouse_down as extern "C" fn(&Object, Sel, *mut Object),
            );
            decl.add_method(
                sel!(mouseDragged:),
                mouse_dragged as extern "C" fn(&Object, Sel, *mut Object),
            );
            decl.add_method(
                sel!(mouseUp:),
                mouse_up as extern "C" fn(&Object, Sel, *mut Object),
            );
            decl.add_method(
                sel!(rightMouseDown:),
                right_mouse_down as extern "C" fn(&Object, Sel, *mut Object),
            );
            decl.add_method(
                sel!(rightMouseDragged:),
                right_mouse_dragged as extern "C" fn(&Object, Sel, *mut Object),
            );
            decl.add_method(
                sel!(rightMouseUp:),
                right_mouse_up as extern "C" fn(&Object, Sel, *mut Object),
            );
            decl.add_method(
                sel!(flagsChanged:),
                flags_changed as extern "C" fn(&Object, Sel, *mut Object),
            );
            decl.add_method(
                sel!(keyDown:),
                key_down as extern "C" fn(&Object, Sel, *mut Object),
            );
            decl.add_method(
                sel!(playheadRedrawTick:),
                playhead_redraw_tick as extern "C" fn(&Object, Sel, *mut Object),
            );
            decl.add_method(
                sel!(isFlipped),
                is_flipped as extern "C" fn(&Object, Sel) -> BOOL,
            );
            decl.add_method(
                sel!(acceptsFirstResponder),
                accepts_first_responder as extern "C" fn(&Object, Sel) -> BOOL,
            );
            decl.add_method(
                sel!(acceptsFirstMouse:),
                accepts_first_mouse as extern "C" fn(&Object, Sel, *mut Object) -> BOOL,
            );
            decl.add_method(sel!(dealloc), dealloc as extern "C" fn(&Object, Sel));
        }
        decl.register()
    }
}

extern "C" fn update_tracking_areas(this: &Object, _cmd: Sel) {
    unsafe {
        let superclass = class!(NSView);
        let _: () = msg_send![super(this, superclass), updateTrackingAreas];
        remove_tracking_area(this);

        let options = NSTRACKING_MOUSE_ENTERED_AND_EXITED
            | NSTRACKING_MOUSE_MOVED
            | NSTRACKING_ACTIVE_ALWAYS
            | NSTRACKING_IN_VISIBLE_RECT
            | NSTRACKING_ENABLED_DURING_MOUSE_DRAG;
        let area: *mut Object = msg_send![class!(NSTrackingArea), alloc];
        let area: *mut Object = msg_send![
            area,
            initWithRect: ns_rect(0.0, 0.0, 0.0, 0.0)
            options: options
            owner: this
            userInfo: ptr::null_mut::<Object>()
        ];
        if !area.is_null() {
            let _: () = msg_send![this, addTrackingArea: area];
            let Some(view) = (this as *const Object as *mut Object).as_mut() else {
                return;
            };
            view.set_ivar("tracking_area", area as usize);
        }
    }
}

extern "C" fn is_flipped(_this: &Object, _cmd: Sel) -> BOOL {
    YES
}

extern "C" fn accepts_first_responder(_this: &Object, _cmd: Sel) -> BOOL {
    YES
}

extern "C" fn accepts_first_mouse(_this: &Object, _cmd: Sel, _event: *mut Object) -> BOOL {
    YES
}

extern "C" fn draw_rect(this: &Object, _cmd: Sel, _dirty: NSRect) {
    unsafe {
        let bounds: NSRect = msg_send![this, bounds];
        let view = this as *const Object as *mut Object;
        if let (Some(runtime), Some(renderer)) = (runtime_mut(this), renderer_mut(this)) {
            render_paint_plan(renderer, runtime.paint_plan(), view, bounds);
        }
    }
}

extern "C" fn mouse_moved(this: &Object, _cmd: Sel, event: *mut Object) {
    dispatch_mouse_event(this, event, PointerButton::Primary, MouseEventKind::Move);
}

extern "C" fn mouse_exited(this: &Object, _cmd: Sel, event: *mut Object) {
    unsafe {
        let Some(runtime) = runtime_mut(this) else {
            return;
        };
        if !event.is_null() {
            runtime.dispatch_event(Event::pointer_modifiers_changed(event_modifiers(event)));
        }
        runtime.dispatch_event(Event::pointer_move(Point::new(-1.0, -1.0)));
        let _: () = msg_send![this, setNeedsDisplay: YES];
    }
}

extern "C" fn mouse_down(this: &Object, _cmd: Sel, event: *mut Object) {
    unsafe {
        make_first_responder(this);
    }
    dispatch_mouse_event(this, event, PointerButton::Primary, MouseEventKind::Press);
}

extern "C" fn mouse_dragged(this: &Object, _cmd: Sel, event: *mut Object) {
    dispatch_mouse_event(this, event, PointerButton::Primary, MouseEventKind::Move);
}

extern "C" fn mouse_up(this: &Object, _cmd: Sel, event: *mut Object) {
    dispatch_mouse_event(this, event, PointerButton::Primary, MouseEventKind::Release);
}

extern "C" fn right_mouse_down(this: &Object, _cmd: Sel, event: *mut Object) {
    dispatch_mouse_event(this, event, PointerButton::Secondary, MouseEventKind::Press);
}

extern "C" fn right_mouse_dragged(this: &Object, _cmd: Sel, event: *mut Object) {
    dispatch_mouse_event(this, event, PointerButton::Secondary, MouseEventKind::Move);
}

extern "C" fn right_mouse_up(this: &Object, _cmd: Sel, event: *mut Object) {
    dispatch_mouse_event(
        this,
        event,
        PointerButton::Secondary,
        MouseEventKind::Release,
    );
}

extern "C" fn flags_changed(this: &Object, _cmd: Sel, event: *mut Object) {
    unsafe {
        if event.is_null() {
            return;
        }
        let Some(runtime) = runtime_mut(this) else {
            return;
        };
        runtime.dispatch_event(Event::pointer_modifiers_changed(event_modifiers(event)));
        let _: () = msg_send![this, setNeedsDisplay: YES];
    }
}

extern "C" fn key_down(this: &Object, _cmd: Sel, event: *mut Object) {
    unsafe {
        if event.is_null() {
            return;
        }
        let mut handled = false;
        if let Some(runtime) = runtime_mut(this)
            && let Some(text) = event_characters(event)
        {
            handled = dispatch_key_text(runtime, &text, event_modifiers(event));
        }
        if handled {
            let _: () = msg_send![this, setNeedsDisplay: YES];
        } else {
            let _: () = msg_send![super(this, class!(NSView)), keyDown: event];
        }
    }
}

extern "C" fn playhead_redraw_tick(this: &Object, _cmd: Sel, _timer: *mut Object) {
    unsafe {
        if runtime_mut(this)
            .map(|runtime| runtime.needs_realtime_redraw())
            .unwrap_or(false)
        {
            let _: () = msg_send![this, setNeedsDisplay: YES];
            let _: () = msg_send![this, displayIfNeeded];
        }
    }
}

extern "C" fn dealloc(this: &Object, _cmd: Sel) {
    unsafe {
        stop_redraw_driver(this);
        remove_tracking_area(this);
        drop_runtime(this);
        drop_renderer(this);
        let superclass = class!(NSView);
        let _: () = msg_send![super(this, superclass), dealloc];
    }
}

#[derive(Clone, Copy)]
enum MouseEventKind {
    Press,
    Move,
    Release,
}

fn dispatch_mouse_event(
    this: &Object,
    event: *mut Object,
    button: PointerButton,
    kind: MouseEventKind,
) {
    unsafe {
        if event.is_null() {
            return;
        }
        let Some(runtime) = runtime_mut(this) else {
            return;
        };
        let position = event_position(this, event);
        let modifiers = event_modifiers(event);
        match kind {
            MouseEventKind::Press => {
                runtime.dispatch_event(Event::pointer_modifiers_changed(modifiers));
                runtime.dispatch_event(pointer_press_event_for_click_count(
                    position,
                    button,
                    modifiers,
                    event_click_count(event),
                ));
            }
            MouseEventKind::Move => {
                runtime.dispatch_event(Event::pointer_move(position));
                runtime.dispatch_event(Event::pointer_modifiers_changed(modifiers));
            }
            MouseEventKind::Release => {
                runtime.dispatch_event(Event::pointer_modifiers_changed(modifiers));
                runtime.dispatch_event(Event::pointer_release(position, button, modifiers));
            }
        }
        let _: () = msg_send![this, setNeedsDisplay: YES];
    }
}

fn pointer_press_event_for_click_count(
    position: Point,
    button: PointerButton,
    modifiers: PointerModifiers,
    click_count: usize,
) -> Event {
    if click_count >= 2 {
        Event::pointer_double_click(position, button, modifiers)
    } else {
        Event::pointer_press(position, button, modifiers)
    }
}

unsafe fn event_modifiers(event: *mut Object) -> PointerModifiers {
    let flags: u64 = msg_send![event, modifierFlags];
    PointerModifiers {
        command: flags & NSEVENT_MODIFIER_FLAG_COMMAND != 0,
        shift: flags & NSEVENT_MODIFIER_FLAG_SHIFT != 0,
        alt: flags & NSEVENT_MODIFIER_FLAG_OPTION != 0,
    }
}

unsafe fn event_characters(event: *mut Object) -> Option<String> {
    let characters: *mut Object = msg_send![event, characters];
    ns_string_to_string(characters)
}

unsafe fn ns_string_to_string(string: *mut Object) -> Option<String> {
    if string.is_null() {
        return None;
    }
    let bytes: *const i8 = msg_send![string, UTF8String];
    if bytes.is_null() {
        return None;
    }
    CStr::from_ptr(bytes).to_str().ok().map(str::to_owned)
}

unsafe fn make_first_responder(this: &Object) {
    let window: *mut Object = msg_send![this, window];
    if !window.is_null() {
        let _: BOOL = msg_send![window, makeFirstResponder: this];
    }
}

fn dispatch_key_character(runtime: &mut dyn RadiantVst3Editor, ch: char) -> bool {
    match ch {
        '\u{1b}' => runtime.cancel_text_entry(),
        '\r' | '\n' => runtime.dispatch_key_press(WidgetKey::Enter),
        '\u{8}' => runtime.dispatch_key_press(WidgetKey::Backspace),
        '\u{7f}' => runtime.dispatch_key_press(WidgetKey::Delete),
        _ if !ch.is_control() => runtime.dispatch_character(ch),
        _ => false,
    }
}

fn dispatch_key_text(
    runtime: &mut dyn RadiantVst3Editor,
    text: &str,
    modifiers: PointerModifiers,
) -> bool {
    if modifiers.command {
        return false;
    }
    text.chars().fold(false, |handled, ch| {
        handled | dispatch_key_character(runtime, ch)
    })
}

fn dispatch_vst3_key_down(
    runtime: &mut dyn RadiantVst3Editor,
    key: u16,
    key_code: i16,
    modifiers: i16,
) -> bool {
    use toybox_vst3_ffi::Steinberg::VirtualKeyCodes_::{
        KEY_BACK, KEY_DELETE, KEY_DOWN, KEY_END, KEY_ENTER, KEY_ESCAPE, KEY_HOME, KEY_LEFT,
        KEY_RETURN, KEY_RIGHT, KEY_TAB, KEY_UP,
    };

    let modifiers = vst3_pointer_modifiers(modifiers);
    if modifiers.command {
        return false;
    }

    let key_code = i64::from(key_code);
    let semantic_key = if key_code == KEY_ENTER as i64 || key_code == KEY_RETURN as i64 {
        Some(WidgetKey::Enter)
    } else if key_code == KEY_TAB as i64 {
        Some(WidgetKey::Tab)
    } else if key_code == KEY_BACK as i64 {
        Some(WidgetKey::Backspace)
    } else if key_code == KEY_DELETE as i64 {
        Some(WidgetKey::Delete)
    } else if key_code == KEY_LEFT as i64 {
        Some(WidgetKey::ArrowLeft)
    } else if key_code == KEY_RIGHT as i64 {
        Some(WidgetKey::ArrowRight)
    } else if key_code == KEY_UP as i64 {
        Some(WidgetKey::ArrowUp)
    } else if key_code == KEY_DOWN as i64 {
        Some(WidgetKey::ArrowDown)
    } else if key_code == KEY_HOME as i64 {
        Some(WidgetKey::Home)
    } else if key_code == KEY_END as i64 {
        Some(WidgetKey::End)
    } else {
        None
    };
    if let Some(key) = semantic_key {
        return runtime.dispatch_key_press(key);
    }
    if key_code == KEY_ESCAPE as i64 {
        return runtime.cancel_text_entry();
    }

    let Some(character) = vst3_key_down_to_input_char(key, key_code as i16) else {
        return false;
    };
    dispatch_key_text(runtime, &character.to_string(), modifiers)
}

fn vst3_pointer_modifiers(modifiers: i16) -> PointerModifiers {
    use toybox_vst3_ffi::Steinberg::KeyModifier_::{kAlternateKey, kCommandKey, kShiftKey};

    let modifiers = i64::from(modifiers);
    PointerModifiers {
        command: modifiers & kCommandKey as i64 != 0,
        shift: modifiers & kShiftKey as i64 != 0,
        alt: modifiers & kAlternateKey as i64 != 0,
    }
}

unsafe fn event_click_count(event: *mut Object) -> usize {
    msg_send![event, clickCount]
}

unsafe fn event_position(this: &Object, event: *mut Object) -> Point {
    let window_point: NSPoint = msg_send![event, locationInWindow];
    let local_point: NSPoint =
        msg_send![this, convertPoint: window_point fromView: ptr::null_mut::<Object>()];
    Point::new(local_point.x as f32, local_point.y as f32)
}

unsafe fn start_redraw_driver(view: *mut Object) -> *mut RedrawDriver {
    if view.is_null() {
        return ptr::null_mut();
    }
    let retained_view: *mut Object = msg_send![view, retain];
    let view_addr = retained_view as usize;
    let stop = Arc::new(AtomicBool::new(false));
    let stop_for_thread = Arc::clone(&stop);
    let handle = thread::spawn(move || {
        while !stop_for_thread.load(Ordering::Acquire) {
            let view = view_addr as *mut Object;
            unsafe {
                let _: () = msg_send![
                    view,
                    performSelectorOnMainThread: sel!(playheadRedrawTick:)
                    withObject: ptr::null_mut::<Object>()
                    waitUntilDone: NO
                ];
                wake_main_run_loop();
            }
            thread::sleep(PLAYHEAD_REDRAW_INTERVAL);
        }
        let view = view_addr as *mut Object;
        unsafe {
            let _: () = msg_send![view, release];
        }
    });
    Box::into_raw(Box::new(RedrawDriver {
        stop,
        handle: Some(handle),
    }))
}

unsafe fn wake_main_run_loop() {
    let main_run_loop = CFRunLoopGetMain();
    if !main_run_loop.is_null() {
        CFRunLoopWakeUp(main_run_loop);
    }
}

unsafe fn stop_redraw_driver(view: *const Object) {
    let Some(view) = view.cast_mut().as_mut() else {
        return;
    };
    let driver = *view.get_ivar::<usize>("redraw_driver") as *mut RedrawDriver;
    if driver.is_null() {
        return;
    }
    view.set_ivar("redraw_driver", 0_usize);
    let mut driver = Box::from_raw(driver);
    driver.stop.store(true, Ordering::Release);
    if let Some(handle) = driver.handle.take() {
        let _ = handle.join();
    }
}

unsafe fn remove_tracking_area(view: *const Object) {
    let Some(view_ref) = view.as_ref() else {
        return;
    };
    let area = *view_ref.get_ivar::<usize>("tracking_area") as *mut Object;
    if area.is_null() {
        return;
    }
    let _: () = msg_send![view_ref, removeTrackingArea: area];
    let _: () = msg_send![area, release];
    if let Some(view_mut) = view.cast_mut().as_mut() {
        view_mut.set_ivar("tracking_area", 0_usize);
    }
}

unsafe fn runtime_mut(view: *const Object) -> Option<&'static mut dyn RadiantVst3Editor> {
    let runtime = *(view.as_ref()?.get_ivar::<usize>("runtime")) as *mut Box<dyn RadiantVst3Editor>;
    runtime.as_mut().map(Box::as_mut)
}

unsafe fn drop_runtime(view: *const Object) {
    drop(take_runtime(view));
}

unsafe fn take_runtime(view: *const Object) -> Option<Box<dyn RadiantVst3Editor>> {
    let view = view.cast_mut().as_mut()?;
    let runtime = *view.get_ivar::<usize>("runtime") as *mut Box<dyn RadiantVst3Editor>;
    if runtime.is_null() {
        return None;
    }
    (*view).set_ivar("runtime", 0_usize);
    Some(*Box::from_raw(runtime))
}

unsafe fn renderer_mut(view: *const Object) -> Option<&'static mut EmbeddedVelloRenderer> {
    let renderer = *(view.as_ref()?.get_ivar::<usize>("renderer")) as *mut EmbeddedVelloRenderer;
    renderer.as_mut()
}

unsafe fn drop_renderer(view: *const Object) {
    let Some(view) = view.cast_mut().as_mut() else {
        return;
    };
    let renderer = *view.get_ivar::<usize>("renderer") as *mut EmbeddedVelloRenderer;
    if renderer.is_null() {
        return;
    }
    (*view).set_ivar("renderer", 0_usize);
    drop(Box::from_raw(renderer));
}

#[cfg(test)]
mod tests {
    use super::*;
    use radiant::runtime::EmbeddedFont;
    use radiant::theme::ThemeTokens;

    struct MockEditor {
        plan: SurfacePaintPlan,
        characters: Vec<char>,
        keys: Vec<WidgetKey>,
        canceled: bool,
    }

    impl MockEditor {
        fn new() -> Self {
            Self {
                plan: SurfacePaintPlan::empty(&ThemeTokens::default()),
                characters: Vec::new(),
                keys: Vec::new(),
                canceled: false,
            }
        }
    }

    impl RadiantVst3Editor for MockEditor {
        fn resize(&mut self, _width: u32, _height: u32) {}

        fn dispatch_event(&mut self, _event: Event) {}

        fn paint_plan(&mut self) -> &SurfacePaintPlan {
            &self.plan
        }

        fn needs_realtime_redraw(&self) -> bool {
            false
        }

        fn dispatch_key_press(&mut self, key: WidgetKey) -> bool {
            self.keys.push(key);
            true
        }

        fn dispatch_character(&mut self, character: char) -> bool {
            self.characters.push(character);
            true
        }

        fn cancel_text_entry(&mut self) -> bool {
            self.canceled = true;
            true
        }
    }

    #[test]
    fn pointer_press_event_uses_double_click_for_repeated_appkit_press() {
        let position = Point::new(24.0, 48.0);
        let modifiers = PointerModifiers {
            alt: true,
            ..PointerModifiers::default()
        };

        assert!(matches!(
            pointer_press_event_for_click_count(position, PointerButton::Primary, modifiers, 1),
            Event::PointerPress { position: pressed, .. } if pressed == position
        ));
        assert!(matches!(
            pointer_press_event_for_click_count(position, PointerButton::Primary, modifiers, 2),
            Event::PointerDoubleClick { position: clicked, .. } if clicked == position
        ));
    }

    #[test]
    fn command_modified_text_is_left_for_the_host_responder_chain() {
        let mut editor = MockEditor::new();
        let modifiers = PointerModifiers {
            command: true,
            ..PointerModifiers::default()
        };

        assert!(!dispatch_key_text(&mut editor, "z", modifiers));
        assert!(editor.characters.is_empty());
    }

    #[test]
    fn option_generated_text_preserves_the_character_appkit_produced() {
        let mut editor = MockEditor::new();
        let modifiers = PointerModifiers {
            alt: true,
            ..PointerModifiers::default()
        };

        assert!(dispatch_key_text(&mut editor, "å", modifiers));
        assert_eq!(editor.characters, vec!['å']);
    }

    #[test]
    fn vst3_key_callback_dispatches_text_and_semantic_keys() {
        use toybox_vst3_ffi::Steinberg::VirtualKeyCodes_::KEY_LEFT;

        let mut editor = MockEditor::new();

        assert!(dispatch_vst3_key_down(&mut editor, 'A' as u16, 0, 0));
        assert!(dispatch_vst3_key_down(&mut editor, 0, KEY_LEFT as i16, 0));
        assert_eq!(editor.characters, vec!['A']);
        assert_eq!(editor.keys, vec![WidgetKey::ArrowLeft]);
    }

    #[test]
    fn vst3_command_modified_text_is_left_for_the_host() {
        use toybox_vst3_ffi::Steinberg::KeyModifier_::kCommandKey;

        let mut editor = MockEditor::new();

        assert!(!dispatch_vst3_key_down(
            &mut editor,
            'z' as u16,
            0,
            kCommandKey as i16
        ));
        assert!(editor.characters.is_empty());
        assert!(editor.keys.is_empty());
    }

    #[test]
    fn hosted_gui_reports_declared_default_size_before_open() {
        let gui = RadiantVst3HostedGui::new(
            "ToyboxRadiantVst3EditorContractTest",
            MockEditor::new(),
            420,
            282,
        );

        assert_eq!(gui.last_size(), Some((420, 282)));
    }

    #[test]
    fn hosted_gui_preserves_explicit_text_options() {
        let gui = RadiantVst3HostedGui::new(
            "ToyboxRadiantVst3EditorTextOptionsTest",
            MockEditor::new(),
            420,
            282,
        )
        .with_text_options(
            NativeTextOptions::default().embedded_font(EmbeddedFont::from_static(b"font bytes")),
        );

        assert_eq!(gui.text_options.embedded_fonts.len(), 1);
        assert_eq!(gui.text_options.embedded_fonts[0].bytes(), b"font bytes");
    }

    #[test]
    fn hosted_gui_preserves_last_host_size_after_close() {
        let mut gui = RadiantVst3HostedGui::new(
            "ToyboxRadiantVst3EditorPreservedSizeTest",
            MockEditor::new(),
            420,
            282,
        );

        gui.request_resize(640, 480);
        gui.close();

        assert_eq!(gui.last_size(), Some((640, 480)));
        assert_eq!(gui.initial_open_size(), (640, 480));
    }

    #[test]
    fn radiant_editor_view_registers_input_and_redraw_selectors() {
        unsafe {
            let view = new_radiant_view("ToyboxRadiantVst3EditorSelectorTest", 420, 282)
                .expect("Radiant editor view should be created");

            let responds_mouse_moved: BOOL =
                msg_send![view.as_ptr(), respondsToSelector: sel!(mouseMoved:)];
            let responds_right_mouse_dragged: BOOL =
                msg_send![view.as_ptr(), respondsToSelector: sel!(rightMouseDragged:)];
            let responds_flags_changed: BOOL =
                msg_send![view.as_ptr(), respondsToSelector: sel!(flagsChanged:)];
            let responds_key_down: BOOL =
                msg_send![view.as_ptr(), respondsToSelector: sel!(keyDown:)];
            let responds_redraw_tick: BOOL =
                msg_send![view.as_ptr(), respondsToSelector: sel!(playheadRedrawTick:)];
            assert_eq!(responds_mouse_moved, YES);
            assert_eq!(responds_right_mouse_dragged, YES);
            assert_eq!(responds_flags_changed, YES);
            assert_eq!(responds_key_down, YES);
            assert_eq!(responds_redraw_tick, YES);

            let _: () = msg_send![view.as_ptr(), release];
        }
    }
}
