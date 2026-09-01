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
    Arc, Mutex,
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
const NSEVENT_MODIFIER_FLAG_CONTROL: u64 = 1 << 18;
const NSEVENT_MODIFIER_FLAG_OPTION: u64 = 1 << 19;
const NSEVENT_MODIFIER_FLAG_COMMAND: u64 = 1 << 20;
const NS_ENTER_CHARACTER: char = '\u{3}';
const NS_TAB_CHARACTER: char = '\u{9}';
const NS_BACK_TAB_CHARACTER: char = '\u{19}';
const NS_UP_ARROW_FUNCTION_KEY: char = '\u{f700}';
const NS_DOWN_ARROW_FUNCTION_KEY: char = '\u{f701}';
const NS_LEFT_ARROW_FUNCTION_KEY: char = '\u{f702}';
const NS_RIGHT_ARROW_FUNCTION_KEY: char = '\u{f703}';
const NS_DELETE_FUNCTION_KEY: char = '\u{f728}';
const NS_HOME_FUNCTION_KEY: char = '\u{f729}';
const NS_END_FUNCTION_KEY: char = '\u{f72b}';
const NSTRACKING_MOUSE_ENTERED_AND_EXITED: usize = 0x01;
const NSTRACKING_MOUSE_MOVED: usize = 0x02;
const NSTRACKING_ACTIVE_ALWAYS: usize = 0x80;
const NSTRACKING_IN_VISIBLE_RECT: usize = 0x200;
const NSTRACKING_ENABLED_DURING_MOUSE_DRAG: usize = 0x400;
const PLAYHEAD_REDRAW_INTERVAL: Duration = Duration::from_millis(33);
const ACTIVE_POINTER_BUTTON_NONE: usize = 0;
const ACTIVE_POINTER_BUTTON_PRIMARY: usize = 1;
const ACTIVE_POINTER_BUTTON_SECONDARY: usize = 2;
const ACTIVE_POINTER_BUTTON_AUXILIARY: usize = 3;
static EDITOR_VIEW_CLASS_REGISTRATION: Mutex<()> = Mutex::new(());

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
    tick_pending: Arc<AtomicBool>,
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

    /// Dispatch one command-modified textual shortcut.
    ///
    /// Returning `true` consumes the shortcut. The default preserves the
    /// existing host responder-chain behavior for legacy editors.
    fn dispatch_shortcut(&mut self, _character: char, _modifiers: PointerModifiers) -> bool {
        false
    }

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
    callback_keyboard_only: bool,
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
            text_options: crate::radiant_gui::bundled_text_options(),
            callback_keyboard_only: false,
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
                self.callback_keyboard_only,
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
                cancel_native_interaction(root_view.as_ptr());
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

    /// Show the native child view without recreating the retained editor.
    pub fn show(&self) -> bool {
        let Some(root_view) = self.root_view else {
            return false;
        };
        unsafe {
            let _: () = msg_send![root_view.as_ptr(), setHidden: NO];
            let _: () = msg_send![root_view.as_ptr(), setNeedsDisplay: YES];
        }
        true
    }

    /// Hide the native child view while preserving its retained editor state.
    pub fn hide(&self) {
        if let Some(root_view) = self.root_view {
            unsafe {
                let _: () = msg_send![root_view.as_ptr(), setHidden: YES];
            }
        }
    }

    /// Refresh renderer state after a host DPI-scale change.
    pub fn set_scale(&self, _scale: f64) {
        if let Some(root_view) = self.root_view {
            unsafe {
                let bounds: NSRect = msg_send![root_view.as_ptr(), bounds];
                resize_renderer(
                    root_view.as_ptr(),
                    bounds.size.width.max(1.0) as u32,
                    bounds.size.height.max(1.0) as u32,
                );
                let _: () = msg_send![root_view.as_ptr(), setNeedsDisplay: YES];
            }
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

    fn show(&self) -> bool {
        RadiantVst3HostedGui::show(self)
    }

    fn set_callback_keyboard_mode(&mut self, callback_only: bool) {
        self.callback_keyboard_only = callback_only;
        if let Some(root_view) = self.root_view {
            unsafe {
                set_callback_keyboard_mode_for_view(root_view.as_ptr(), callback_only);
            }
        }
    }

    fn set_default_size(&mut self, width: u32, height: u32) {
        self.default_size = (width.max(1), height.max(1));
        if self.root_view.is_none() {
            self.size.set(None);
        }
    }

    fn host_size_from_logical(&self, width: u32, height: u32) -> (u32, u32) {
        (width.max(1), height.max(1))
    }

    fn logical_size_from_host(&self, width: u32, height: u32) -> (u32, u32) {
        (width.max(1), height.max(1))
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
            let _: () = msg_send![root_view.as_ptr(), setNeedsDisplay: YES];
            handled
        }
    }

    fn on_key_up(&self, _key: u16, _key_code: i16, modifiers: i16) -> bool {
        let Some(root_view) = self.root_view else {
            return false;
        };
        unsafe {
            let Some(runtime) = runtime_mut(root_view.as_ptr()) else {
                return false;
            };
            dispatch_vst3_key_up(runtime, modifiers);
            let _: () = msg_send![root_view.as_ptr(), setNeedsDisplay: YES];
        }
        false
    }

    fn on_focus(&self, focused: bool) -> bool {
        let Some(root_view) = self.root_view else {
            return false;
        };
        unsafe {
            if !focused {
                cancel_native_interaction(root_view.as_ptr());
            }
            set_first_responder(root_view.as_ptr(), focused)
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
    callback_keyboard_only: bool,
) -> Result<NonNull<Object>, Box<dyn RadiantVst3Editor>> {
    let Some(root_view) = new_radiant_view(class_name, width, height) else {
        return Err(editor);
    };
    set_callback_keyboard_mode_for_view(root_view.as_ptr(), callback_keyboard_only);
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
    start_redraw_driver(root_view.as_ptr());
    let _: () = msg_send![root_view.as_ptr(), updateTrackingAreas];
    Ok(root_view)
}

unsafe fn new_radiant_view(
    class_name: &'static str,
    width: u32,
    height: u32,
) -> Option<NonNull<Object>> {
    let view_class = editor_view_class(class_name)?;
    let view: *mut Object = msg_send![view_class, alloc];
    let view: *mut Object =
        msg_send![view, initWithFrame: ns_rect(0.0, 0.0, width as f64, height as f64)];
    let view = NonNull::new(view)?;
    (*view.as_ptr()).set_ivar("runtime", 0_usize);
    (*view.as_ptr()).set_ivar("renderer", 0_usize);
    (*view.as_ptr()).set_ivar("redraw_driver", 0_usize);
    (*view.as_ptr()).set_ivar("active_pointer_button", ACTIVE_POINTER_BUTTON_NONE);
    (*view.as_ptr()).set_ivar("callback_keyboard_only", 0_usize);
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

fn editor_view_class(class_name: &'static str) -> Option<&'static Class> {
    if let Some(existing) = Class::get(class_name) {
        return Some(existing);
    }

    let _registration = EDITOR_VIEW_CLASS_REGISTRATION
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(existing) = Class::get(class_name) {
        return Some(existing);
    }

    {
        let superclass = class!(NSView);
        let Some(mut decl) = ClassDecl::new(class_name, superclass) else {
            return Class::get(class_name);
        };
        decl.add_ivar::<usize>("runtime");
        decl.add_ivar::<usize>("renderer");
        decl.add_ivar::<usize>("tracking_area");
        decl.add_ivar::<usize>("redraw_driver");
        decl.add_ivar::<usize>("active_pointer_button");
        decl.add_ivar::<usize>("callback_keyboard_only");
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
                sel!(scrollWheel:),
                scroll_wheel as extern "C" fn(&Object, Sel, *mut Object),
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
        Some(decl.register())
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
        let button = primary_pointer_button_for_event(event);
        set_active_pointer_button(this, button);
        dispatch_mouse_event(this, event, button, MouseEventKind::Press);
    }
}

extern "C" fn mouse_dragged(this: &Object, _cmd: Sel, event: *mut Object) {
    let button = unsafe { active_pointer_button(this).unwrap_or(PointerButton::Primary) };
    dispatch_mouse_event(this, event, button, MouseEventKind::Move);
}

extern "C" fn mouse_up(this: &Object, _cmd: Sel, event: *mut Object) {
    let button = unsafe { take_active_pointer_button(this).unwrap_or(PointerButton::Primary) };
    dispatch_mouse_event(this, event, button, MouseEventKind::Release);
}

extern "C" fn right_mouse_down(this: &Object, _cmd: Sel, event: *mut Object) {
    unsafe {
        set_active_pointer_button(this, PointerButton::Secondary);
    }
    dispatch_mouse_event(this, event, PointerButton::Secondary, MouseEventKind::Press);
}

extern "C" fn right_mouse_dragged(this: &Object, _cmd: Sel, event: *mut Object) {
    dispatch_mouse_event(this, event, PointerButton::Secondary, MouseEventKind::Move);
}

extern "C" fn right_mouse_up(this: &Object, _cmd: Sel, event: *mut Object) {
    unsafe {
        clear_active_pointer_button(this);
    }
    dispatch_mouse_event(
        this,
        event,
        PointerButton::Secondary,
        MouseEventKind::Release,
    );
}

extern "C" fn flags_changed(this: &Object, _cmd: Sel, event: *mut Object) {
    unsafe {
        if native_keyboard_dispatch_suppressed(this) {
            return;
        }
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
        if native_keyboard_dispatch_suppressed(this) {
            return;
        }
        if event.is_null() {
            return;
        }
        let mut handled = false;
        if let Some(runtime) = runtime_mut(this) {
            let modifiers = event_modifiers(event);
            let text = event_characters(event);
            handled = dispatch_appkit_key_down(runtime, text.as_deref(), modifiers);
        }
        if handled {
            let _: () = msg_send![this, setNeedsDisplay: YES];
        } else {
            let _: () = msg_send![super(this, class!(NSView)), keyDown: event];
        }
    }
}

extern "C" fn scroll_wheel(this: &Object, _cmd: Sel, event: *mut Object) {
    unsafe {
        if event.is_null() {
            return;
        }
        let Some(runtime) = runtime_mut(this) else {
            return;
        };
        let position = event_position(this, event);
        let delta_x: f64 = msg_send![event, scrollingDeltaX];
        let delta_y: f64 = msg_send![event, scrollingDeltaY];
        let delta = Vector2::new(delta_x as f32, delta_y as f32);
        runtime.dispatch_event(Event::pointer_modifiers_changed(event_modifiers(event)));
        runtime.dispatch_event(Event::scroll(position, delta));
        let _: () = msg_send![this, setNeedsDisplay: YES];
    }
}

extern "C" fn playhead_redraw_tick(this: &Object, _cmd: Sel, _timer: *mut Object) {
    unsafe {
        complete_pending_redraw_tick(this);
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
        cancel_native_interaction(this);
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
                dispatch_pointer_move(runtime, position, modifiers);
            }
            MouseEventKind::Release => {
                runtime.dispatch_event(Event::pointer_modifiers_changed(modifiers));
                runtime.dispatch_event(Event::pointer_release(position, button, modifiers));
            }
        }
        let _: () = msg_send![this, setNeedsDisplay: YES];
    }
}

fn dispatch_pointer_move(
    runtime: &mut dyn RadiantVst3Editor,
    position: Point,
    modifiers: PointerModifiers,
) {
    runtime.dispatch_event(Event::pointer_modifiers_changed(modifiers));
    runtime.dispatch_event(Event::pointer_move(position));
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

unsafe fn event_modifier_flags(event: *mut Object) -> u64 {
    msg_send![event, modifierFlags]
}

fn primary_pointer_button(modifier_flags: u64) -> PointerButton {
    if modifier_flags & NSEVENT_MODIFIER_FLAG_CONTROL != 0 {
        PointerButton::Secondary
    } else {
        PointerButton::Primary
    }
}

unsafe fn primary_pointer_button_for_event(event: *mut Object) -> PointerButton {
    if event.is_null() {
        PointerButton::Primary
    } else {
        primary_pointer_button(event_modifier_flags(event))
    }
}

fn pointer_button_to_ivar_value(button: PointerButton) -> usize {
    match button {
        PointerButton::Primary => ACTIVE_POINTER_BUTTON_PRIMARY,
        PointerButton::Secondary => ACTIVE_POINTER_BUTTON_SECONDARY,
        PointerButton::Auxiliary => ACTIVE_POINTER_BUTTON_AUXILIARY,
    }
}

fn pointer_button_from_ivar_value(value: usize) -> Option<PointerButton> {
    match value {
        ACTIVE_POINTER_BUTTON_PRIMARY => Some(PointerButton::Primary),
        ACTIVE_POINTER_BUTTON_SECONDARY => Some(PointerButton::Secondary),
        ACTIVE_POINTER_BUTTON_AUXILIARY => Some(PointerButton::Auxiliary),
        _ => None,
    }
}

fn take_pointer_button_ivar_value(value: &mut usize) -> Option<PointerButton> {
    let button = pointer_button_from_ivar_value(*value);
    *value = ACTIVE_POINTER_BUTTON_NONE;
    button
}

unsafe fn set_active_pointer_button(view: *const Object, button: PointerButton) {
    let Some(view) = view.cast_mut().as_mut() else {
        return;
    };
    view.set_ivar(
        "active_pointer_button",
        pointer_button_to_ivar_value(button),
    );
}

unsafe fn active_pointer_button(view: *const Object) -> Option<PointerButton> {
    let view = view.as_ref()?;
    pointer_button_from_ivar_value(*view.get_ivar::<usize>("active_pointer_button"))
}

unsafe fn take_active_pointer_button(view: *const Object) -> Option<PointerButton> {
    let view = view.cast_mut().as_mut()?;
    let mut value = *view.get_ivar::<usize>("active_pointer_button");
    let button = take_pointer_button_ivar_value(&mut value);
    view.set_ivar("active_pointer_button", value);
    button
}

unsafe fn clear_active_pointer_button(view: *const Object) {
    let Some(view) = view.cast_mut().as_mut() else {
        return;
    };
    view.set_ivar("active_pointer_button", ACTIVE_POINTER_BUTTON_NONE);
}

/// Cancel any native pointer gesture before clearing Radiant focus.
///
/// The active-button ivar is consumed before dispatch so repeated focus or
/// teardown callbacks cannot deliver a second pointer-capture cancellation.
unsafe fn cancel_native_interaction(view: *const Object) {
    let had_active_button = take_active_pointer_button(view).is_some();
    let Some(runtime) = runtime_mut(view) else {
        return;
    };
    if had_active_button {
        runtime.dispatch_event(Event::pointer_capture_cancelled());
    }
    runtime.dispatch_event(Event::clear_focus());
}

unsafe fn event_modifiers(event: *mut Object) -> PointerModifiers {
    let flags = event_modifier_flags(event);
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
    let _ = set_first_responder(this as *const Object as *mut Object, true);
}

unsafe fn set_first_responder(view: *mut Object, focused: bool) -> bool {
    if view.is_null() {
        return false;
    }
    let window: *mut Object = msg_send![view, window];
    if window.is_null() {
        return false;
    }
    if focused {
        let result: BOOL = msg_send![window, makeFirstResponder: view];
        result == YES
    } else {
        let first_responder: *mut Object = msg_send![window, firstResponder];
        if first_responder != view {
            return true;
        }
        let result: BOOL = msg_send![view, resignFirstResponder];
        result == YES
    }
}

unsafe fn set_callback_keyboard_mode_for_view(view: *mut Object, callback_only: bool) {
    if let Some(view) = view.as_mut() {
        view.set_ivar(
            "callback_keyboard_only",
            if callback_only { 1_usize } else { 0_usize },
        );
    }
}

unsafe fn native_keyboard_dispatch_suppressed(view: *const Object) -> bool {
    view.as_ref()
        .is_some_and(|view| *view.get_ivar::<usize>("callback_keyboard_only") != 0)
}

fn dispatch_key_character(runtime: &mut dyn RadiantVst3Editor, ch: char) -> bool {
    match ch {
        '\u{1b}' => runtime.cancel_text_entry(),
        NS_ENTER_CHARACTER | '\r' | '\n' => runtime.dispatch_key_press(WidgetKey::Enter),
        NS_TAB_CHARACTER | NS_BACK_TAB_CHARACTER => runtime.dispatch_key_press(WidgetKey::Tab),
        '\u{8}' => runtime.dispatch_key_press(WidgetKey::Backspace),
        '\u{7f}' => runtime.dispatch_key_press(WidgetKey::Backspace),
        NS_UP_ARROW_FUNCTION_KEY => runtime.dispatch_key_press(WidgetKey::ArrowUp),
        NS_DOWN_ARROW_FUNCTION_KEY => runtime.dispatch_key_press(WidgetKey::ArrowDown),
        NS_LEFT_ARROW_FUNCTION_KEY => runtime.dispatch_key_press(WidgetKey::ArrowLeft),
        NS_RIGHT_ARROW_FUNCTION_KEY => runtime.dispatch_key_press(WidgetKey::ArrowRight),
        NS_DELETE_FUNCTION_KEY => runtime.dispatch_key_press(WidgetKey::Delete),
        NS_HOME_FUNCTION_KEY => runtime.dispatch_key_press(WidgetKey::Home),
        NS_END_FUNCTION_KEY => runtime.dispatch_key_press(WidgetKey::End),
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
        return text.chars().fold(false, |handled, ch| {
            handled | dispatch_command_shortcut(runtime, ch, modifiers)
        });
    }
    text.chars().fold(false, |handled, ch| {
        handled | dispatch_key_character(runtime, ch)
    })
}

fn dispatch_command_shortcut(
    runtime: &mut dyn RadiantVst3Editor,
    ch: char,
    modifiers: PointerModifiers,
) -> bool {
    if ch.is_control()
        || matches!(
            ch,
            NS_ENTER_CHARACTER
                | NS_TAB_CHARACTER
                | NS_BACK_TAB_CHARACTER
                | NS_UP_ARROW_FUNCTION_KEY
                | NS_DOWN_ARROW_FUNCTION_KEY
                | NS_LEFT_ARROW_FUNCTION_KEY
                | NS_RIGHT_ARROW_FUNCTION_KEY
                | NS_DELETE_FUNCTION_KEY
                | NS_HOME_FUNCTION_KEY
                | NS_END_FUNCTION_KEY
        )
    {
        return false;
    }
    runtime.dispatch_shortcut(ch, modifiers)
}

fn dispatch_appkit_key_down(
    runtime: &mut dyn RadiantVst3Editor,
    text: Option<&str>,
    modifiers: PointerModifiers,
) -> bool {
    runtime.dispatch_event(Event::pointer_modifiers_changed(modifiers));
    text.is_some_and(|text| dispatch_key_text(runtime, text, modifiers))
}

#[cfg(feature = "vst3")]
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
    runtime.dispatch_event(Event::pointer_modifiers_changed(modifiers));
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
        if modifiers.command {
            return false;
        }
        return runtime.dispatch_key_press(key);
    }
    if key_code == KEY_ESCAPE as i64 {
        if modifiers.command {
            return false;
        }
        return runtime.cancel_text_entry();
    }

    let Some(character) = vst3_key_down_to_input_char(key, key_code as i16) else {
        return false;
    };
    dispatch_key_text(runtime, &character.to_string(), modifiers)
}

#[cfg(feature = "vst3")]
fn dispatch_vst3_key_up(runtime: &mut dyn RadiantVst3Editor, modifiers: i16) {
    runtime.dispatch_event(Event::pointer_modifiers_changed(vst3_pointer_modifiers(
        modifiers,
    )));
}

#[cfg(feature = "vst3")]
fn vst3_pointer_modifiers(modifiers: i16) -> PointerModifiers {
    use toybox_vst3_ffi::Steinberg::KeyModifier_::{kAlternateKey, kCommandKey, kShiftKey};

    let modifiers = i64::from(modifiers);
    PointerModifiers {
        command: modifiers & kCommandKey as i64 != 0,
        shift: modifiers & kShiftKey as i64 != 0,
        alt: modifiers & kAlternateKey as i64 != 0,
    }
}

#[cfg(not(feature = "vst3"))]
fn dispatch_vst3_key_down(
    runtime: &mut dyn RadiantVst3Editor,
    key: u16,
    key_code: i16,
    modifiers: i16,
) -> bool {
    let modifiers = PointerModifiers {
        command: i64::from(modifiers) & (1 << 20) != 0,
        shift: i64::from(modifiers) & (1 << 17) != 0,
        alt: i64::from(modifiers) & (1 << 19) != 0,
    };
    runtime.dispatch_event(Event::pointer_modifiers_changed(modifiers));
    let Some(character) = vst3_key_down_to_input_char(key, key_code) else {
        return false;
    };
    dispatch_key_text(runtime, &character.to_string(), modifiers)
}

#[cfg(not(feature = "vst3"))]
fn dispatch_vst3_key_up(runtime: &mut dyn RadiantVst3Editor, modifiers: i16) {
    runtime.dispatch_event(Event::pointer_modifiers_changed(PointerModifiers {
        command: i64::from(modifiers) & (1 << 20) != 0,
        shift: i64::from(modifiers) & (1 << 17) != 0,
        alt: i64::from(modifiers) & (1 << 19) != 0,
    }));
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

unsafe fn start_redraw_driver(view: *mut Object) {
    if view.is_null() {
        return;
    }
    let retained_view: *mut Object = msg_send![view, retain];
    let view_addr = retained_view as usize;
    let stop = Arc::new(AtomicBool::new(false));
    let tick_pending = Arc::new(AtomicBool::new(false));
    let driver = Box::into_raw(Box::new(RedrawDriver {
        stop: Arc::clone(&stop),
        tick_pending: Arc::clone(&tick_pending),
        handle: None,
    }));
    (*view).set_ivar("redraw_driver", driver as usize);
    let stop_for_thread = Arc::clone(&stop);
    let tick_pending_for_thread = Arc::clone(&tick_pending);
    let handle = thread::spawn(move || {
        while !stop_for_thread.load(Ordering::Acquire) {
            if claim_redraw_tick(&tick_pending_for_thread) {
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
            }
            thread::sleep(PLAYHEAD_REDRAW_INTERVAL);
        }
        let view = view_addr as *mut Object;
        unsafe {
            let _: () = msg_send![view, release];
        }
    });
    (*driver).handle = Some(handle);
}

fn claim_redraw_tick(tick_pending: &AtomicBool) -> bool {
    tick_pending
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
}

fn complete_redraw_tick(tick_pending: &AtomicBool) {
    tick_pending.store(false, Ordering::Release);
}

unsafe fn complete_pending_redraw_tick(view: &Object) {
    let driver = *view.get_ivar::<usize>("redraw_driver") as *const RedrawDriver;
    if let Some(driver) = driver.as_ref() {
        complete_redraw_tick(&driver.tick_pending);
    }
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
        events: Vec<Event>,
        characters: Vec<char>,
        shortcuts: Vec<(char, PointerModifiers)>,
        keys: Vec<WidgetKey>,
        operations: Vec<&'static str>,
        canceled: bool,
        shortcut_result: bool,
        event_count: Option<Arc<Mutex<usize>>>,
        character_count: Option<Arc<Mutex<usize>>>,
        event_sink: Option<Arc<Mutex<Vec<Event>>>>,
    }

    impl MockEditor {
        fn new() -> Self {
            Self {
                plan: SurfacePaintPlan::empty(&ThemeTokens::default()),
                events: Vec::new(),
                characters: Vec::new(),
                shortcuts: Vec::new(),
                keys: Vec::new(),
                operations: Vec::new(),
                canceled: false,
                shortcut_result: false,
                event_count: None,
                character_count: None,
                event_sink: None,
            }
        }
    }

    impl RadiantVst3Editor for MockEditor {
        fn resize(&mut self, _width: u32, _height: u32) {}

        fn dispatch_event(&mut self, event: Event) {
            self.operations.push("event");
            self.events.push(event);
            if let Some(event_sink) = &self.event_sink {
                event_sink.lock().unwrap().push(event);
            }
            if let Some(event_count) = &self.event_count {
                *event_count.lock().unwrap() += 1;
            }
        }

        fn paint_plan(&mut self) -> &SurfacePaintPlan {
            &self.plan
        }

        fn needs_realtime_redraw(&self) -> bool {
            false
        }

        fn dispatch_key_press(&mut self, key: WidgetKey) -> bool {
            self.operations.push("key");
            self.keys.push(key);
            true
        }

        fn dispatch_character(&mut self, character: char) -> bool {
            self.operations.push("character");
            self.characters.push(character);
            if let Some(character_count) = &self.character_count {
                *character_count.lock().unwrap() += 1;
            }
            true
        }

        fn dispatch_shortcut(&mut self, character: char, modifiers: PointerModifiers) -> bool {
            self.operations.push("shortcut");
            self.shortcuts.push((character, modifiers));
            self.shortcut_result
        }

        fn cancel_text_entry(&mut self) -> bool {
            self.operations.push("cancel");
            self.canceled = true;
            true
        }
    }

    const TEST_EVENT_CLASS_NAME: &str = "ToyboxRadiantVst3KeyboardTestEvent";

    extern "C" fn test_event_modifier_flags(this: &Object, _cmd: Sel) -> u64 {
        unsafe { *this.get_ivar::<u64>("modifier_flags") }
    }

    extern "C" fn test_event_characters(this: &Object, _cmd: Sel) -> *mut Object {
        unsafe { *this.get_ivar::<usize>("characters") as *mut Object }
    }

    fn test_event_class() -> &'static Class {
        if let Some(class) = Class::get(TEST_EVENT_CLASS_NAME) {
            return class;
        }

        let _registration = EDITOR_VIEW_CLASS_REGISTRATION
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(class) = Class::get(TEST_EVENT_CLASS_NAME) {
            return class;
        }

        let mut decl = ClassDecl::new(TEST_EVENT_CLASS_NAME, class!(NSObject))
            .expect("test event class should be declared");
        decl.add_ivar::<u64>("modifier_flags");
        decl.add_ivar::<usize>("characters");
        unsafe {
            decl.add_method(
                sel!(modifierFlags),
                test_event_modifier_flags as extern "C" fn(&Object, Sel) -> u64,
            );
            decl.add_method(
                sel!(characters),
                test_event_characters as extern "C" fn(&Object, Sel) -> *mut Object,
            );
        }
        decl.register()
    }

    unsafe fn new_test_key_event(modifier_flags: u64) -> (NonNull<Object>, *mut Object) {
        let event: *mut Object = msg_send![test_event_class(), alloc];
        let event: *mut Object = msg_send![event, init];
        let event = NonNull::new(event).expect("test event should initialize");
        let characters: *mut Object = msg_send![class!(NSString), alloc];
        let characters: *mut Object = msg_send![characters, initWithUTF8String: c"a".as_ptr()];
        assert!(!characters.is_null(), "test event text should initialize");
        (*event.as_ptr()).set_ivar("modifier_flags", modifier_flags);
        (*event.as_ptr()).set_ivar("characters", characters as usize);
        (event, characters)
    }

    unsafe fn install_test_runtime(view: NonNull<Object>, editor: Box<dyn RadiantVst3Editor>) {
        (*view.as_ptr()).set_ivar("runtime", Box::into_raw(Box::new(editor)) as usize);
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
    fn ordinary_primary_pointer_press_stays_primary() {
        let button = primary_pointer_button(0);

        assert_eq!(button, PointerButton::Primary);
        assert_eq!(
            pointer_button_from_ivar_value(pointer_button_to_ivar_value(button)),
            Some(PointerButton::Primary)
        );
    }

    #[test]
    fn right_pointer_gesture_stays_secondary() {
        let button = PointerButton::Secondary;

        assert_eq!(
            pointer_button_from_ivar_value(pointer_button_to_ivar_value(button)),
            Some(PointerButton::Secondary)
        );
    }

    #[test]
    fn control_primary_pointer_press_normalizes_to_secondary() {
        assert_eq!(
            primary_pointer_button(NSEVENT_MODIFIER_FLAG_CONTROL),
            PointerButton::Secondary
        );
    }

    #[test]
    fn release_uses_stored_effective_button_and_clears_state() {
        let mut stored =
            pointer_button_to_ivar_value(primary_pointer_button(NSEVENT_MODIFIER_FLAG_CONTROL));

        assert_eq!(
            take_pointer_button_ivar_value(&mut stored),
            Some(PointerButton::Secondary)
        );
        assert_eq!(stored, ACTIVE_POINTER_BUTTON_NONE);
        assert_eq!(take_pointer_button_ivar_value(&mut stored), None);
    }

    #[test]
    fn pointer_move_dispatches_modifiers_before_position() {
        let mut editor = MockEditor::new();
        let position = Point::new(24.0, 48.0);
        let modifiers = PointerModifiers {
            shift: true,
            alt: true,
            command: true,
        };

        dispatch_pointer_move(&mut editor, position, modifiers);

        assert_eq!(
            editor.events,
            vec![
                Event::pointer_modifiers_changed(modifiers),
                Event::pointer_move(position),
            ]
        );
    }

    #[test]
    fn command_modified_text_is_left_for_the_host_responder_chain() {
        let mut editor = MockEditor::new();
        let modifiers = PointerModifiers {
            command: true,
            ..PointerModifiers::default()
        };

        assert!(!dispatch_appkit_key_down(&mut editor, Some("z"), modifiers));
        assert!(editor.characters.is_empty());
        assert_eq!(editor.shortcuts, vec![('z', modifiers)]);
        assert_eq!(editor.operations, vec!["event", "shortcut"]);
    }

    #[test]
    fn appkit_claimed_command_shortcut_is_consumed_without_text_input() {
        let mut editor = MockEditor::new();
        editor.shortcut_result = true;
        let modifiers = PointerModifiers {
            command: true,
            shift: true,
            ..PointerModifiers::default()
        };

        assert!(dispatch_appkit_key_down(&mut editor, Some("Z"), modifiers));
        assert_eq!(editor.shortcuts, vec![('Z', modifiers)]);
        assert!(editor.characters.is_empty());
        assert_eq!(editor.operations, vec!["event", "shortcut"]);
    }

    #[test]
    fn appkit_key_down_dispatches_modifiers_before_semantic_key() {
        let mut editor = MockEditor::new();
        let modifiers = PointerModifiers {
            shift: true,
            ..PointerModifiers::default()
        };

        assert!(dispatch_appkit_key_down(
            &mut editor,
            Some(&NS_LEFT_ARROW_FUNCTION_KEY.to_string()),
            modifiers,
        ));

        assert_eq!(editor.operations, vec!["event", "key"]);
        assert_eq!(
            editor.events,
            vec![Event::pointer_modifiers_changed(modifiers)]
        );
        assert_eq!(editor.keys, vec![WidgetKey::ArrowLeft]);
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
    fn appkit_function_keys_dispatch_semantic_widget_keys() {
        let mut editor = MockEditor::new();

        for character in [
            NS_UP_ARROW_FUNCTION_KEY,
            NS_DOWN_ARROW_FUNCTION_KEY,
            NS_LEFT_ARROW_FUNCTION_KEY,
            NS_RIGHT_ARROW_FUNCTION_KEY,
            NS_DELETE_FUNCTION_KEY,
            NS_HOME_FUNCTION_KEY,
            NS_END_FUNCTION_KEY,
        ] {
            assert!(dispatch_key_character(&mut editor, character));
        }

        assert_eq!(
            editor.keys,
            vec![
                WidgetKey::ArrowUp,
                WidgetKey::ArrowDown,
                WidgetKey::ArrowLeft,
                WidgetKey::ArrowRight,
                WidgetKey::Delete,
                WidgetKey::Home,
                WidgetKey::End,
            ]
        );
        assert!(editor.characters.is_empty());
    }

    #[test]
    fn appkit_delete_character_and_forward_delete_dispatch_distinct_keys() {
        let mut editor = MockEditor::new();

        assert!(dispatch_key_character(&mut editor, '\u{7f}'));
        assert!(dispatch_key_character(&mut editor, NS_DELETE_FUNCTION_KEY));

        assert_eq!(editor.keys, vec![WidgetKey::Backspace, WidgetKey::Delete]);
        assert!(editor.characters.is_empty());
    }

    #[test]
    fn appkit_tab_backtab_and_keypad_enter_dispatch_semantic_keys() {
        let mut editor = MockEditor::new();

        for character in [NS_TAB_CHARACTER, NS_BACK_TAB_CHARACTER, NS_ENTER_CHARACTER] {
            assert!(dispatch_key_character(&mut editor, character));
        }

        assert_eq!(
            editor.keys,
            vec![WidgetKey::Tab, WidgetKey::Tab, WidgetKey::Enter]
        );
        assert!(editor.characters.is_empty());
    }

    #[test]
    #[cfg(feature = "vst3")]
    fn vst3_key_callback_dispatches_text_and_semantic_keys() {
        use toybox_vst3_ffi::Steinberg::VirtualKeyCodes_::KEY_LEFT;

        let mut editor = MockEditor::new();

        assert!(dispatch_vst3_key_down(&mut editor, 'A' as u16, 0, 0));
        assert!(dispatch_vst3_key_down(&mut editor, 0, KEY_LEFT as i16, 0));
        assert_eq!(editor.characters, vec!['A']);
        assert_eq!(editor.keys, vec![WidgetKey::ArrowLeft]);
    }

    #[test]
    #[cfg(feature = "vst3")]
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
        assert_eq!(
            editor.shortcuts,
            vec![(
                'z',
                PointerModifiers {
                    command: true,
                    ..PointerModifiers::default()
                }
            )]
        );
    }

    #[test]
    #[cfg(feature = "vst3")]
    fn vst3_claimed_command_shortcut_preserves_shift_without_text_input() {
        use toybox_vst3_ffi::Steinberg::KeyModifier_::{kCommandKey, kShiftKey};

        let mut editor = MockEditor::new();
        editor.shortcut_result = true;
        let modifiers = (kCommandKey | kShiftKey) as i16;

        assert!(dispatch_vst3_key_down(
            &mut editor,
            'Z' as u16,
            0,
            modifiers
        ));
        assert_eq!(
            editor.shortcuts,
            vec![(
                'Z',
                PointerModifiers {
                    command: true,
                    shift: true,
                    alt: false,
                }
            ),]
        );
        assert!(editor.characters.is_empty());
    }

    #[test]
    #[cfg(feature = "vst3")]
    fn vst3_key_callbacks_preserve_held_modifiers_on_key_up() {
        use toybox_vst3_ffi::Steinberg::KeyModifier_::{kAlternateKey, kShiftKey};
        use toybox_vst3_ffi::Steinberg::VirtualKeyCodes_::KEY_LEFT;

        let mut editor = MockEditor::new();
        let modifiers = (kShiftKey | kAlternateKey) as i16;

        assert!(dispatch_vst3_key_down(
            &mut editor,
            0,
            KEY_LEFT as i16,
            modifiers
        ));
        dispatch_vst3_key_up(&mut editor, kShiftKey as i16);
        dispatch_vst3_key_up(&mut editor, 0);

        assert_eq!(editor.keys, vec![WidgetKey::ArrowLeft]);
        assert_eq!(
            editor.events,
            vec![
                Event::pointer_modifiers_changed(PointerModifiers {
                    shift: true,
                    alt: true,
                    command: false,
                }),
                Event::pointer_modifiers_changed(PointerModifiers {
                    shift: true,
                    alt: false,
                    command: false,
                }),
                Event::pointer_modifiers_changed(PointerModifiers::default()),
            ]
        );
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
        assert_ne!(gui.text_options, crate::radiant_gui::bundled_text_options());
    }

    #[test]
    fn hosted_gui_defaults_to_bundled_text_options() {
        let gui = RadiantVst3HostedGui::new(
            "ToyboxRadiantVst3EditorBundledTextOptionsTest",
            MockEditor::new(),
            420,
            282,
        );

        assert_eq!(gui.text_options, crate::radiant_gui::bundled_text_options());
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
    fn host_focus_loss_cancels_pointer_before_clearing_focus_once() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let mut gui = RadiantVst3HostedGui::new(
            "ToyboxRadiantVst3EditorFocusCancellationTest",
            MockEditor::new(),
            420,
            282,
        );

        unsafe {
            let view =
                new_radiant_view("ToyboxRadiantVst3EditorFocusCancellationViewTest", 420, 282)
                    .expect("Radiant editor view should be created");
            gui.root_view = Some(view);
            let mut editor = MockEditor::new();
            editor.event_sink = Some(Arc::clone(&events));
            install_test_runtime(view, Box::new(editor));
            set_active_pointer_button(view.as_ptr(), PointerButton::Primary);

            assert!(!Vst3HostedGui::on_focus(&gui, false));
            assert!(!Vst3HostedGui::on_focus(&gui, false));

            let events = events.lock().unwrap().clone();
            assert_eq!(events[0], Event::pointer_capture_cancelled());
            assert_eq!(events[1], Event::clear_focus());
            assert_eq!(
                events
                    .iter()
                    .filter(|event| **event == Event::pointer_capture_cancelled())
                    .count(),
                1
            );
            assert_eq!(
                *view.as_ref().get_ivar::<usize>("active_pointer_button"),
                ACTIVE_POINTER_BUTTON_NONE
            );

            drop_runtime(view.as_ptr());
            gui.root_view = None;
            let _: () = msg_send![view.as_ptr(), release];
        }
    }

    #[test]
    fn close_reopen_cancels_pointer_before_retaining_editor_once() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let mut gui = RadiantVst3HostedGui::new(
            "ToyboxRadiantVst3EditorCloseReopenCancellationTest",
            MockEditor::new(),
            420,
            282,
        );

        unsafe {
            let view =
                new_radiant_view("ToyboxRadiantVst3EditorCloseCancellationViewTest", 420, 282)
                    .expect("Radiant editor view should be created");
            gui.root_view = Some(view);
            let mut editor = MockEditor::new();
            editor.event_sink = Some(Arc::clone(&events));
            install_test_runtime(view, Box::new(editor));
            set_active_pointer_button(view.as_ptr(), PointerButton::Primary);

            gui.close_view();

            assert!(gui.root_view.is_none());
            assert!(gui.editor.is_some());
            let events_after_close = events.lock().unwrap().clone();
            assert_eq!(
                events_after_close,
                vec![Event::pointer_capture_cancelled(), Event::clear_focus()]
            );

            let reopened = new_radiant_view(
                "ToyboxRadiantVst3EditorCloseReopenCancellationViewTest",
                420,
                282,
            )
            .expect("reopened Radiant editor view should be created");
            let editor = gui.editor.take().expect("editor should survive close");
            install_test_runtime(reopened, editor);
            gui.root_view = Some(reopened);
            assert_eq!(
                *reopened.as_ref().get_ivar::<usize>("active_pointer_button"),
                ACTIVE_POINTER_BUTTON_NONE
            );

            gui.close_view();
            let events_after_reopen_close = events.lock().unwrap().clone();
            assert_eq!(
                events_after_reopen_close
                    .iter()
                    .filter(|event| **event == Event::pointer_capture_cancelled())
                    .count(),
                1
            );
        }
    }

    #[test]
    fn callback_only_mode_controls_appkit_paths_without_affecting_vst3_callbacks() {
        let event_count = Arc::new(Mutex::new(0));
        let character_count = Arc::new(Mutex::new(0));
        let mut gui = RadiantVst3HostedGui::new(
            "ToyboxRadiantVst3EditorKeyboardModeTest",
            MockEditor::new(),
            420,
            282,
        );
        Vst3HostedGui::set_callback_keyboard_mode(&mut gui, true);
        assert!(gui.callback_keyboard_only);

        unsafe {
            let view = new_radiant_view("ToyboxRadiantVst3EditorKeyboardModeViewTest", 420, 282)
                .expect("Radiant editor view should be created");
            gui.root_view = Some(view);
            let mut editor = MockEditor::new();
            editor.event_count = Some(Arc::clone(&event_count));
            editor.character_count = Some(Arc::clone(&character_count));
            let editor: Box<dyn RadiantVst3Editor> = Box::new(editor);
            (*view.as_ptr()).set_ivar("runtime", Box::into_raw(Box::new(editor)) as usize);
            let (event, characters) = new_test_key_event(NSEVENT_MODIFIER_FLAG_SHIFT);

            Vst3HostedGui::set_callback_keyboard_mode(&mut gui, true);
            flags_changed(view.as_ref(), sel!(flagsChanged:), event.as_ptr());
            key_down(view.as_ref(), sel!(keyDown:), event.as_ptr());
            assert_eq!(*event_count.lock().unwrap(), 0);
            assert_eq!(*character_count.lock().unwrap(), 0);

            Vst3HostedGui::set_callback_keyboard_mode(&mut gui, false);
            assert!(!native_keyboard_dispatch_suppressed(view.as_ptr()));
            flags_changed(view.as_ref(), sel!(flagsChanged:), event.as_ptr());
            key_down(view.as_ref(), sel!(keyDown:), event.as_ptr());
            let native_event_count = *event_count.lock().unwrap();
            let native_character_count = *character_count.lock().unwrap();
            assert_eq!(native_event_count, 2);
            assert_eq!(native_character_count, 1);

            Vst3HostedGui::set_callback_keyboard_mode(&mut gui, true);
            assert!(Vst3HostedGui::on_key_down(&gui, 'b' as u16, 0, 0));
            assert!(!Vst3HostedGui::on_key_up(&gui, 0, 0, 0));
            assert_eq!(*event_count.lock().unwrap(), native_event_count + 2);
            assert_eq!(*character_count.lock().unwrap(), native_character_count + 1);

            drop_runtime(view.as_ptr());
            gui.root_view = None;
            let _: () = msg_send![characters, release];
            let _: () = msg_send![event.as_ptr(), release];
            let _: () = msg_send![view.as_ptr(), release];
        }
    }

    #[test]
    fn redraw_tick_claims_are_coalesced_until_main_thread_completion() {
        let tick_pending = AtomicBool::new(false);

        assert!(claim_redraw_tick(&tick_pending));
        assert!(!claim_redraw_tick(&tick_pending));

        complete_redraw_tick(&tick_pending);
        assert!(claim_redraw_tick(&tick_pending));
    }

    #[test]
    fn radiant_editor_view_class_registration_is_concurrency_safe() {
        const THREAD_COUNT: usize = 8;
        let barrier = Arc::new(std::sync::Barrier::new(THREAD_COUNT));
        let handles = (0..THREAD_COUNT)
            .map(|_| {
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    editor_view_class("ToyboxRadiantVst3EditorConcurrentRegistrationTest")
                        .map(|class| class as *const Class as usize)
                })
            })
            .collect::<Vec<_>>();
        let classes = handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .expect("class registration thread should not panic")
            })
            .collect::<Vec<_>>();

        assert!(classes[0].is_some());
        assert!(classes.iter().all(|class| *class == classes[0]));
    }

    #[test]
    fn radiant_editor_view_registers_input_and_redraw_selectors() {
        unsafe {
            let view = new_radiant_view("ToyboxRadiantVst3EditorSelectorTest", 420, 282)
                .expect("Radiant editor view should be created");
            assert_eq!(
                *view.as_ref().get_ivar::<usize>("active_pointer_button"),
                ACTIVE_POINTER_BUTTON_NONE
            );

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
