//! AppKit child view and explicit CAMetalLayer bridge for embedded GPUI.

#![allow(
    clippy::missing_docs_in_private_items,
    unexpected_cfgs,
    unsafe_op_in_unsafe_fn
)]

use std::ffi::c_void;
use std::ptr::{self, NonNull};
use std::rc::{Rc, Weak};
use std::sync::Mutex;

use gpui::{
    ClipboardItem, DevicePixels, GpuSpecs, KeyDownEvent, KeyUpEvent, Modifiers,
    ModifiersChangedEvent, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels,
    PlatformAtlas, PlatformInput, Point, Scene, ScrollDelta, ScrollWheelEvent, Size, TouchPhase,
    px,
};
use gpui_wgpu::{WgpuRenderer, WgpuSurfaceConfig};
use objc::declare::ClassDecl;
use objc::runtime::{BOOL, Class, NO, Object, Sel, YES};
use objc::{Encode, Encoding, class, msg_send, sel, sel_impl};
use raw_window_handle_06::{
    AppKitWindowHandle, DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle,
    RawWindowHandle, WindowHandle,
};

use super::super::{NativeEventIdentity, WindowState};

pub(crate) fn parent_scale_factor(_parent: raw_window_handle::RawWindowHandle) -> f32 {
    1.0
}

/// Return the identity of the AppKit key event currently being dispatched.
///
/// VST3 hosts can deliver their keyboard callback from inside AppKit's event
/// dispatch. Reading the current event lets the two paths share a token when
/// they are genuinely the same native event, while returning `None` for
/// unrelated asynchronous callbacks preserves both deliveries.
pub(crate) fn current_event_identity() -> Option<NativeEventIdentity> {
    unsafe {
        let application: *mut Object = msg_send![class!(NSApplication), sharedApplication];
        if application.is_null() {
            return None;
        }
        let event: *mut Object = msg_send![application, currentEvent];
        event_identity(event)
    }
}

const NSPASTEBOARD_STRING_TYPE: &str = "public.utf8-plain-text";

pub(crate) fn read_clipboard() -> Option<ClipboardItem> {
    unsafe {
        let pasteboard: *mut Object = msg_send![class!(NSPasteboard), generalPasteboard];
        if pasteboard.is_null() {
            return None;
        }
        let kind = cocoa_string(NSPASTEBOARD_STRING_TYPE)?;
        let value: *mut Object = msg_send![pasteboard, stringForType: kind.as_ptr()];
        let text = ns_string(value);
        let _: () = msg_send![kind.as_ptr(), release];
        text.map(ClipboardItem::new_string)
    }
}

pub(crate) fn write_clipboard(item: ClipboardItem) {
    let Some(text) = item.text() else {
        return;
    };
    unsafe {
        let pasteboard: *mut Object = msg_send![class!(NSPasteboard), generalPasteboard];
        if pasteboard.is_null() {
            return;
        }
        let Some(kind) = cocoa_string(NSPASTEBOARD_STRING_TYPE) else {
            return;
        };
        let Some(value) = cocoa_string(&text) else {
            let _: () = msg_send![kind.as_ptr(), release];
            return;
        };
        let _: isize = msg_send![pasteboard, clearContents];
        let _: BOOL = msg_send![pasteboard, setString: value.as_ptr() forType: kind.as_ptr()];
        let _: () = msg_send![value.as_ptr(), release];
        let _: () = msg_send![kind.as_ptr(), release];
    }
}

const SHIFT: u64 = 1 << 17;
const CONTROL: u64 = 1 << 18;
const OPTION: u64 = 1 << 19;
const COMMAND: u64 = 1 << 20;
const FUNCTION: u64 = 1 << 23;
const NS_NOT_FOUND: usize = usize::MAX;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
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
#[derive(Clone, Copy, Debug)]
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
#[derive(Clone, Copy, Debug)]
struct NSRect {
    origin: NSPoint,
    size: NSSize,
}

unsafe impl Encode for NSRect {
    fn encode() -> Encoding {
        unsafe { Encoding::from_str("{CGRect={CGPoint=dd}{CGSize=dd}}") }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct NSRange {
    location: usize,
    length: usize,
}

unsafe impl Encode for NSRange {
    fn encode() -> Encoding {
        unsafe { Encoding::from_str("{_NSRange=QQ}") }
    }
}

#[derive(Clone, Debug)]
struct ViewHandle(usize);

unsafe impl Send for ViewHandle {}
unsafe impl Sync for ViewHandle {}

impl HasWindowHandle for ViewHandle {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let Some(view) = NonNull::new(self.0 as *mut c_void) else {
            return Err(HandleError::NotSupported);
        };
        let handle = AppKitWindowHandle::new(view);
        Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::AppKit(handle)) })
    }
}

impl HasDisplayHandle for ViewHandle {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Ok(DisplayHandle::appkit())
    }
}

/// One independently owned native GPUI child view.
pub(crate) struct NativeChild {
    view: NonNull<Object>,
    layer: NonNull<Object>,
    renderer: Option<WgpuRenderer>,
    timer: Option<NonNull<Object>>,
    owner_token: Option<Box<Weak<WindowState>>>,
    capture_requested: bool,
    capture_result: Option<Vec<u8>>,
    failed: bool,
}

impl NativeChild {
    pub(crate) fn new(
        parent: raw_window_handle::RawWindowHandle,
        class_name: &'static str,
        owner: Weak<WindowState>,
        gpu_context: gpui_wgpu::GpuContext,
        size: Size<Pixels>,
        _callback_keyboard_only: bool,
    ) -> anyhow::Result<Self> {
        let raw_parent = match parent {
            raw_window_handle::RawWindowHandle::AppKit(handle) => handle.ns_view,
            _ => ptr::null_mut(),
        };
        let parent = NonNull::new(raw_parent)
            .map(|parent| parent.cast::<Object>())
            .ok_or_else(|| anyhow::anyhow!("missing AppKit parent NSView"))?;
        let view = unsafe { new_view(class_name, size) }
            .ok_or_else(|| anyhow::anyhow!("failed to allocate GPUI AppKit child view"))?;
        let child = PendingChild::new(view);
        unsafe {
            let owner_token = Box::new(owner);
            (&mut *view.as_ptr()).set_ivar(
                "owner",
                (&*owner_token as *const Weak<WindowState>) as usize,
            );
            let _: () = msg_send![parent.as_ptr(), addSubview: view.as_ptr()];
            let _: () = msg_send![view.as_ptr(), setWantsLayer: YES];
            // GPUI lays out in logical points, while CAMetalLayer and WGPU
            // consume device pixels. The child is attached before querying
            // the window so this also handles a host whose parent is already
            // on a Retina display.
            let scale_factor = view_scale_factor(view.as_ptr());
            let layer = create_metal_layer(view.as_ptr())
                .ok_or_else(|| anyhow::anyhow!("failed to create GPUI CAMetalLayer"))?;
            let handle = ViewHandle(view.as_ptr() as usize);
            let config = WgpuSurfaceConfig {
                size: gpui::size(
                    DevicePixels((f32::from(size.width) * scale_factor).max(1.0) as i32),
                    DevicePixels((f32::from(size.height) * scale_factor).max(1.0) as i32),
                ),
                transparent: false,
                preferred_present_mode: None,
            };
            let renderer = WgpuRenderer::new_with_surface_target(
                gpu_context,
                &handle,
                gpui_wgpu::wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(layer.as_ptr().cast()),
                config,
                None,
            )
            .map_err(|error| anyhow::anyhow!("failed to initialize GPUI WGPU renderer: {error}"))?;
            sync_metal_layer(view.as_ptr());
            let mut child = child.commit(layer, renderer, owner_token)?;
            child.start_timer();
            Ok(child)
        }
    }

    pub(crate) fn draw(&mut self, scene: &Scene) {
        if self.failed {
            return;
        }
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };
        unsafe { sync_metal_layer(self.view.as_ptr()) };
        if self.capture_requested {
            self.capture_requested = false;
            match renderer.render_to_rgba(scene) {
                Ok(pixels) => self.capture_result = Some(pixels),
                Err(_) => self.failed = true,
            }
        } else {
            let _ = renderer.draw(scene);
        }
    }

    pub(crate) fn request_capture(&mut self) {
        self.capture_requested = true;
        self.capture_result = None;
    }

    pub(crate) fn take_capture(&mut self) -> Option<(u32, u32, Vec<u8>)> {
        let pixels = self.capture_result.take()?;
        let size = self.renderer.as_ref()?.viewport_size();
        Some((
            size.width.0.max(1) as u32,
            size.height.0.max(1) as u32,
            pixels,
        ))
    }

    pub(crate) fn is_failed(&self) -> bool {
        self.failed
    }

    pub(crate) fn resize(&mut self, size: Size<Pixels>) {
        if self.failed {
            return;
        }
        let scale_factor = self.scale_factor();
        unsafe {
            let _: () = msg_send![self.view.as_ptr(), setFrame: ns_rect(
                0.0,
                0.0,
                f64::from(size.width).max(1.0),
                f64::from(size.height).max(1.0)
            )];
            sync_metal_layer(self.view.as_ptr());
        }
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.update_drawable_size(gpui::size(
                DevicePixels((f32::from(size.width) * scale_factor).max(1.0) as i32),
                DevicePixels((f32::from(size.height) * scale_factor).max(1.0) as i32),
            ));
        }
    }

    /// Reconfigure the drawable when the host moves the view between displays
    /// without changing its logical frame.
    pub(crate) fn backing_scale_changed(&mut self, size: Size<Pixels>) {
        if self.failed {
            return;
        }
        let scale_factor = self.scale_factor();
        unsafe { sync_metal_layer(self.view.as_ptr()) };
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.update_drawable_size(gpui::size(
                DevicePixels((f32::from(size.width) * scale_factor).max(1.0) as i32),
                DevicePixels((f32::from(size.height) * scale_factor).max(1.0) as i32),
            ));
        }
    }

    pub(crate) fn set_visible(&mut self, visible: bool) {
        if !self.failed {
            unsafe {
                let _: () =
                    msg_send![self.view.as_ptr(), setHidden: if visible { NO } else { YES }];
                if visible {
                    self.start_timer();
                    let _: () = msg_send![self.view.as_ptr(), setNeedsDisplay: YES];
                } else {
                    self.stop_timer();
                }
            }
        }
    }

    pub(crate) fn set_focus(&mut self, focused: bool) -> bool {
        if self.failed {
            return false;
        }
        unsafe {
            let window: *mut Object = msg_send![self.view.as_ptr(), window];
            if window.is_null() {
                return false;
            }
            if focused {
                let result: BOOL = msg_send![window, makeFirstResponder: self.view.as_ptr()];
                result == YES
            } else {
                let _: () = msg_send![window, makeFirstResponder: ptr::null_mut::<Object>()];
                true
            }
        }
    }

    pub(crate) fn is_visible(&self) -> bool {
        if self.failed {
            return false;
        }
        unsafe {
            let hidden: BOOL = msg_send![self.view.as_ptr(), isHiddenOrHasHiddenAncestor];
            let window: *mut Object = msg_send![self.view.as_ptr(), window];
            let visible: BOOL = if window.is_null() {
                NO
            } else {
                msg_send![window, isVisible]
            };
            let miniaturized: BOOL = if window.is_null() {
                NO
            } else {
                msg_send![window, isMiniaturized]
            };
            hidden == NO && visible == YES && miniaturized == NO
        }
    }

    pub(crate) fn sprite_atlas(&self) -> std::sync::Arc<dyn PlatformAtlas> {
        self.renderer
            .as_ref()
            .map(|renderer| renderer.sprite_atlas().clone())
            .expect("native renderer atlas missing")
    }

    pub(crate) fn gpu_specs(&self) -> Option<GpuSpecs> {
        self.renderer.as_ref().map(WgpuRenderer::gpu_specs)
    }

    pub(crate) fn scale_factor(&self) -> f32 {
        unsafe { view_scale_factor(self.view.as_ptr()) }
    }

    pub(crate) fn window_handle(&self) -> Result<WindowHandle<'static>, HandleError> {
        let view =
            NonNull::new(self.view.as_ptr().cast::<c_void>()).ok_or(HandleError::NotSupported)?;
        let handle = AppKitWindowHandle::new(view);
        Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::AppKit(handle)) })
    }

    pub(crate) fn display_handle(&self) -> Result<DisplayHandle<'static>, HandleError> {
        Ok(DisplayHandle::appkit())
    }

    pub(crate) fn quarantine(&mut self) {
        if self.failed {
            return;
        }
        self.failed = true;
        self.stop_timer();
        unsafe {
            let view = self.view.as_ptr();
            (*view).set_ivar("owner", 0_usize);
            let _: () = msg_send![view, setHidden: YES];
        }
        if let Some(mut renderer) = self.renderer.take() {
            renderer.destroy();
        }
    }

    pub(crate) fn clear_owner(&mut self) {
        unsafe {
            (&mut *self.view.as_ptr()).set_ivar("owner", 0_usize);
        }
    }

    pub(crate) fn close(&mut self) {
        self.stop_timer();
        if let Some(mut renderer) = self.renderer.take() {
            renderer.destroy();
        }
        unsafe {
            let _: () = msg_send![self.layer.as_ptr(), removeFromSuperlayer];
            let _: () = msg_send![self.layer.as_ptr(), release];
            let _: () = msg_send![self.view.as_ptr(), removeFromSuperview];
            let _: () = msg_send![self.view.as_ptr(), release];
        }
        self.owner_token.take();
    }

    fn start_timer(&mut self) {
        if self.timer.is_none() {
            let timer: *mut Object = unsafe {
                msg_send![
                    class!(NSTimer),
                    scheduledTimerWithTimeInterval: 0.016_f64
                    target: self.view.as_ptr()
                    selector: sel!(toyboxGpuiTick:)
                    userInfo: ptr::null_mut::<Object>()
                    repeats: YES
                ]
            };
            self.timer = NonNull::new(timer);
        }
    }

    fn stop_timer(&mut self) {
        if let Some(timer) = self.timer.take() {
            unsafe {
                let _: () = msg_send![timer.as_ptr(), invalidate];
            }
        }
    }
}

struct PendingChild {
    view: NonNull<Object>,
    layer: Option<NonNull<Object>>,
}

impl PendingChild {
    fn new(view: NonNull<Object>) -> Self {
        Self { view, layer: None }
    }

    unsafe fn commit(
        mut self,
        layer: NonNull<Object>,
        renderer: WgpuRenderer,
        owner_token: Box<Weak<WindowState>>,
    ) -> anyhow::Result<NativeChild> {
        self.layer = Some(layer);
        let child = NativeChild {
            view: self.view,
            layer,
            renderer: Some(renderer),
            timer: None,
            owner_token: Some(owner_token),
            capture_requested: false,
            capture_result: None,
            failed: false,
        };
        self.layer = None;
        std::mem::forget(self);
        Ok(child)
    }
}

impl Drop for PendingChild {
    fn drop(&mut self) {
        unsafe {
            if let Some(layer) = self.layer {
                let _: () = msg_send![layer.as_ptr(), removeFromSuperlayer];
                let _: () = msg_send![layer.as_ptr(), release];
            }
            let _: () = msg_send![self.view.as_ptr(), removeFromSuperview];
            let _: () = msg_send![self.view.as_ptr(), release];
        }
    }
}

static CLASS_REGISTRATION: Mutex<()> = Mutex::new(());

unsafe fn new_view(class_name: &'static str, size: Size<Pixels>) -> Option<NonNull<Object>> {
    let class = editor_view_class(class_name)?;
    let view: *mut Object = msg_send![class, alloc];
    let view: *mut Object = msg_send![view, initWithFrame: ns_rect(
        0.0,
        0.0,
        f64::from(size.width).max(1.0),
        f64::from(size.height).max(1.0)
    )];
    let view = NonNull::new(view)?;
    (&mut *view.as_ptr()).set_ivar("owner", 0_usize);
    Some(view)
}

fn editor_view_class(class_name: &'static str) -> Option<&'static Class> {
    let name = format!(
        "{class_name}_ToyboxGpui_{:x}",
        editor_view_class as *const () as usize
    );
    if let Some(class) = Class::get(&name) {
        return Some(class);
    }
    let _lock = CLASS_REGISTRATION.lock().ok()?;
    if let Some(class) = Class::get(&name) {
        return Some(class);
    }
    let mut decl = ClassDecl::new(&name, class!(NSView))?;
    decl.add_ivar::<usize>("owner");
    unsafe {
        decl.add_method(
            sel!(drawRect:),
            draw_rect as extern "C" fn(&Object, Sel, NSRect),
        );
        decl.add_method(
            sel!(toyboxGpuiTick:),
            timer_tick as extern "C" fn(&Object, Sel, *mut Object),
        );
        decl.add_method(
            sel!(setFrameSize:),
            set_frame_size as extern "C" fn(&Object, Sel, NSSize),
        );
        decl.add_method(
            sel!(viewDidChangeBackingProperties),
            view_did_change_backing_properties as extern "C" fn(&Object, Sel),
        );
        decl.add_method(
            sel!(mouseDown:),
            mouse_down as extern "C" fn(&Object, Sel, *mut Object),
        );
        decl.add_method(
            sel!(mouseUp:),
            mouse_up as extern "C" fn(&Object, Sel, *mut Object),
        );
        decl.add_method(
            sel!(mouseDragged:),
            mouse_dragged as extern "C" fn(&Object, Sel, *mut Object),
        );
        decl.add_method(
            sel!(mouseMoved:),
            mouse_moved as extern "C" fn(&Object, Sel, *mut Object),
        );
        decl.add_method(
            sel!(scrollWheel:),
            scroll_wheel as extern "C" fn(&Object, Sel, *mut Object),
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
            sel!(keyUp:),
            key_up as extern "C" fn(&Object, Sel, *mut Object),
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
        decl.add_method(
            sel!(insertText:replacementRange:),
            insert_text as extern "C" fn(&Object, Sel, *mut Object, NSRange),
        );
        decl.add_method(
            sel!(insertText:),
            insert_text_simple as extern "C" fn(&Object, Sel, *mut Object),
        );
        decl.add_method(
            sel!(setMarkedText:selectedRange:replacementRange:),
            set_marked_text as extern "C" fn(&Object, Sel, *mut Object, NSRange, NSRange),
        );
        decl.add_method(sel!(unmarkText), unmark_text as extern "C" fn(&Object, Sel));
        decl.add_method(
            sel!(hasMarkedText),
            has_marked_text as extern "C" fn(&Object, Sel) -> BOOL,
        );
        decl.add_method(
            sel!(markedRange),
            marked_range as extern "C" fn(&Object, Sel) -> NSRange,
        );
        decl.add_method(
            sel!(selectedRange),
            selected_range as extern "C" fn(&Object, Sel) -> NSRange,
        );
        decl.add_method(
            sel!(validAttributesForMarkedText),
            valid_attributes as extern "C" fn(&Object, Sel) -> *mut Object,
        );
        decl.add_method(
            sel!(doCommandBySelector:),
            do_command as extern "C" fn(&Object, Sel, Sel),
        );
        decl.add_method(
            sel!(firstRectForCharacterRange:actualRange:),
            first_rect as extern "C" fn(&Object, Sel, NSRange, *mut Object) -> NSRect,
        );
        decl.add_method(
            sel!(characterIndexForPoint:),
            character_index as extern "C" fn(&Object, Sel, NSPoint) -> usize,
        );
    }
    Some(decl.register())
}

unsafe fn create_metal_layer(view: *mut Object) -> Option<NonNull<Object>> {
    let backing: *mut Object = msg_send![view, layer];
    if backing.is_null() {
        return None;
    }
    let layer: *mut Object = msg_send![class!(CAMetalLayer), new];
    let layer = NonNull::new(layer)?;
    let _: () = msg_send![backing, addSublayer: layer.as_ptr()];
    Some(layer)
}

unsafe fn sync_metal_layer(view: *mut Object) {
    let backing: *mut Object = msg_send![view, layer];
    if backing.is_null() {
        return;
    }
    let layers: *mut Object = msg_send![backing, sublayers];
    if layers.is_null() {
        return;
    }
    let count: usize = msg_send![layers, count];
    if count == 0 {
        return;
    }
    let layer: *mut Object = msg_send![layers, objectAtIndex: count - 1];
    if layer.is_null() {
        return;
    }
    let bounds: NSRect = msg_send![backing, bounds];
    let window: *mut Object = msg_send![view, window];
    let scale: f64 = if window.is_null() {
        1.0
    } else {
        msg_send![window, backingScaleFactor]
    };
    let _: () = msg_send![class!(CATransaction), begin];
    let _: () = msg_send![class!(CATransaction), setDisableActions: YES];
    let _: () = msg_send![layer, setFrame: bounds];
    let _: () = msg_send![layer, setContentsScale: scale];
    let _: () = msg_send![class!(CATransaction), commit];
}

fn ns_rect(x: f64, y: f64, width: f64, height: f64) -> NSRect {
    NSRect {
        origin: NSPoint { x, y },
        size: NSSize { width, height },
    }
}

unsafe fn view_scale_factor(view: *mut Object) -> f32 {
    let window: *mut Object = msg_send![view, window];
    if window.is_null() {
        1.0
    } else {
        let scale: f64 = msg_send![window, backingScaleFactor];
        (scale as f32).max(0.01)
    }
}

fn owner(this: &Object) -> Option<Rc<WindowState>> {
    unsafe {
        let token = *(*this).get_ivar::<usize>("owner") as *const Weak<WindowState>;
        token.as_ref().and_then(Weak::upgrade)
    }
}

fn native_callback(this: &Object, operation: &str, callback: impl FnOnce(&WindowState)) {
    unsafe {
        let _: *mut Object = msg_send![this, retain];
    }
    let Some(owner) = owner(this) else {
        unsafe {
            let _: () = msg_send![this, release];
        }
        return;
    };
    let failed = crate::gui_panic::contain(operation, || callback(owner.as_ref())).is_none();
    if failed {
        unsafe { quarantine_view(this) };
    }
    unsafe {
        let _: () = msg_send![this, release];
    }
}

unsafe fn quarantine_view(view: &Object) {
    let view = view as *const Object as *mut Object;
    (*view).set_ivar("owner", 0_usize);
    let _: () = msg_send![view, setHidden: YES];
}

extern "C" fn draw_rect(this: &Object, _cmd: Sel, _dirty: NSRect) {
    native_callback(this, "GPUI AppKit draw", |owner| owner.request_frame());
}

extern "C" fn timer_tick(this: &Object, _cmd: Sel, _timer: *mut Object) {
    native_callback(this, "GPUI AppKit timer", |owner| owner.native_tick());
}

extern "C" fn set_frame_size(this: &Object, _cmd: Sel, size: NSSize) {
    // `setFrame:` reaches this override through `setFrameSize:`. Preserve
    // NSView's geometry update before synchronizing GPUI; otherwise a reused
    // VST3 view keeps the previous child frame after remove/attach and native
    // coordinates no longer match GPUI's hitboxes.
    unsafe {
        let _: () = msg_send![super(this, class!(NSView)), setFrameSize: size];
    }
    native_callback(this, "GPUI AppKit resize", |owner| {
        owner.native_resize(size.width as f32, size.height as f32);
    });
    unsafe {
        let _: () = msg_send![this, setNeedsDisplay: YES];
    }
}

extern "C" fn view_did_change_backing_properties(this: &Object, _cmd: Sel) {
    native_callback(this, "GPUI AppKit backing scale", |owner| unsafe {
        let bounds: NSRect = msg_send![this, bounds];
        owner.native_backing_scale_changed(bounds.size.width as f32, bounds.size.height as f32);
    });
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

extern "C" fn mouse_down(this: &Object, _cmd: Sel, event: *mut Object) {
    native_callback(this, "GPUI AppKit mouse down", |owner| {
        owner.native_mouse_down_focus();
        let _ = owner.dispatch_input(mouse_input(this, event, true, false));
    });
}

extern "C" fn mouse_up(this: &Object, _cmd: Sel, event: *mut Object) {
    native_callback(this, "GPUI AppKit mouse up", |owner| {
        let _ = owner.dispatch_input(mouse_input(this, event, false, false));
    });
}

extern "C" fn mouse_dragged(this: &Object, _cmd: Sel, event: *mut Object) {
    native_callback(this, "GPUI AppKit mouse drag", |owner| {
        let _ = owner.dispatch_input(mouse_input(this, event, false, true));
    });
}

extern "C" fn mouse_moved(this: &Object, _cmd: Sel, event: *mut Object) {
    native_callback(this, "GPUI AppKit mouse move", |owner| {
        let _ = owner.dispatch_input(PlatformInput::MouseMove(MouseMoveEvent {
            position: event_position(this, event),
            pressed_button: None,
            modifiers: event_modifiers(event),
        }));
    });
}

extern "C" fn scroll_wheel(this: &Object, _cmd: Sel, event: *mut Object) {
    native_callback(this, "GPUI AppKit scroll", |owner| unsafe {
        let dx: f64 = msg_send![event, scrollingDeltaX];
        let dy: f64 = msg_send![event, scrollingDeltaY];
        let _ = owner.dispatch_input(PlatformInput::ScrollWheel(ScrollWheelEvent {
            position: event_position(this, event),
            delta: ScrollDelta::Pixels(Point::new(px(dx as f32), px(dy as f32))),
            modifiers: event_modifiers(event),
            touch_phase: TouchPhase::Moved,
        }));
    });
}

extern "C" fn flags_changed(this: &Object, _cmd: Sel, event: *mut Object) {
    native_callback(this, "GPUI AppKit modifiers", |owner| {
        let _ = owner.dispatch_input(PlatformInput::ModifiersChanged(ModifiersChangedEvent {
            modifiers: event_modifiers(event),
            capslock: gpui::Capslock::default(),
        }));
    });
}

extern "C" fn key_down(this: &Object, _cmd: Sel, event: *mut Object) {
    native_callback(this, "GPUI AppKit key down", |owner| unsafe {
        let flags = event_modifiers(event);
        let ignored: *mut Object = msg_send![event, charactersIgnoringModifiers];
        let key_char = ns_string(ignored);
        let key = key_name(key_char.as_deref());
        let token = event_identity(event).map(|identity| owner.native_event_token(identity));
        let result = owner.dispatch_native_key(
            PlatformInput::KeyDown(KeyDownEvent {
                keystroke: gpui::Keystroke {
                    modifiers: flags,
                    key,
                    key_char: key_char.clone(),
                },
                is_held: false,
                prefer_character_input: false,
            }),
            token,
        );
        if result.propagate && !flags.control && !flags.alt && !flags.platform {
            let events: *mut Object = msg_send![class!(NSArray), arrayWithObject: event];
            let _: () = msg_send![this, interpretKeyEvents: events];
        }
    });
}

extern "C" fn key_up(this: &Object, _cmd: Sel, event: *mut Object) {
    native_callback(this, "GPUI AppKit key up", |owner| unsafe {
        let ignored: *mut Object = msg_send![event, charactersIgnoringModifiers];
        let key_char = ns_string(ignored);
        let token = event_identity(event).map(|identity| owner.native_event_token(identity));
        let _ = owner.dispatch_native_key(
            PlatformInput::KeyUp(KeyUpEvent {
                keystroke: gpui::Keystroke {
                    modifiers: event_modifiers(event),
                    key: key_name(key_char.as_deref()),
                    key_char: None,
                },
            }),
            token,
        );
    });
}

extern "C" fn insert_text(this: &Object, _cmd: Sel, text: *mut Object, _range: NSRange) {
    native_callback(this, "GPUI AppKit insert text", |owner| {
        if let Some(text) = ns_string(text) {
            let token = current_event_identity().map(|identity| owner.native_event_token(identity));
            owner.dispatch_native_text(&text, token);
        }
    });
}

extern "C" fn insert_text_simple(this: &Object, _cmd: Sel, text: *mut Object) {
    insert_text(
        this,
        _cmd,
        text,
        NSRange {
            location: NS_NOT_FOUND,
            length: 0,
        },
    );
}

extern "C" fn set_marked_text(
    this: &Object,
    _cmd: Sel,
    text: *mut Object,
    selected: NSRange,
    replacement: NSRange,
) {
    native_callback(this, "GPUI AppKit marked text", |owner| {
        if let Some(text) = ns_string(text) {
            let replacement = range_option(replacement);
            let selected = range_option(selected);
            owner.dispatch_marked_text(replacement, &text, selected);
        }
    });
}

extern "C" fn unmark_text(this: &Object, _cmd: Sel) {
    native_callback(this, "GPUI AppKit unmark text", |owner| owner.unmark_text());
}

extern "C" fn has_marked_text(this: &Object, _cmd: Sel) -> BOOL {
    if owner(this)
        .and_then(|owner| owner.marked_text_range())
        .is_some()
    {
        YES
    } else {
        NO
    }
}

extern "C" fn marked_range(this: &Object, _cmd: Sel) -> NSRange {
    owner(this)
        .and_then(|owner| owner.marked_text_range())
        .map_or(
            NSRange {
                location: NS_NOT_FOUND,
                length: 0,
            },
            |range| NSRange {
                location: range.start,
                length: range.end.saturating_sub(range.start),
            },
        )
}

extern "C" fn selected_range(this: &Object, _cmd: Sel) -> NSRange {
    owner(this)
        .and_then(|owner| owner.selected_text_range())
        .map_or(
            NSRange {
                location: 0,
                length: 0,
            },
            |range| NSRange {
                location: range.start,
                length: range.end.saturating_sub(range.start),
            },
        )
}

extern "C" fn valid_attributes(_this: &Object, _cmd: Sel) -> *mut Object {
    ptr::null_mut()
}

extern "C" fn do_command(_this: &Object, _cmd: Sel, _selector: Sel) {}

extern "C" fn first_rect(this: &Object, _cmd: Sel, range: NSRange, actual: *mut Object) -> NSRect {
    if !actual.is_null() {
        unsafe { *(actual.cast::<NSRange>()) = range };
    }
    owner(this)
        .and_then(|owner| owner.bounds_for_range(range_option(range).unwrap_or(0..0)))
        .map_or(ns_rect(0.0, 0.0, 1.0, 16.0), |bounds| {
            ns_rect(
                f64::from(bounds.origin.x),
                f64::from(bounds.origin.y),
                f64::from(bounds.size.width).max(1.0),
                f64::from(bounds.size.height).max(1.0),
            )
        })
}

extern "C" fn character_index(this: &Object, _cmd: Sel, point: NSPoint) -> usize {
    owner(this)
        .and_then(|owner| owner.character_index_for_point(point.x as f32, point.y as f32))
        .unwrap_or(0)
}

fn event_position(view: &Object, event: *mut Object) -> Point<Pixels> {
    unsafe {
        let point: NSPoint = msg_send![event, locationInWindow];
        let point: NSPoint =
            msg_send![view, convertPoint: point fromView: ptr::null_mut::<Object>()];
        Point::new(px(point.x as f32), px(point.y as f32))
    }
}

unsafe fn event_identity(event: *mut Object) -> Option<NativeEventIdentity> {
    if event.is_null() {
        return None;
    }
    let event_type: u64 = msg_send![event, type];
    let event_type = match event_type {
        10 | 11 => event_type as u16,
        _ => return None,
    };
    let timestamp: f64 = msg_send![event, timestamp];
    let key_code: u16 = msg_send![event, keyCode];
    let window_number: isize = msg_send![event, windowNumber];
    Some(NativeEventIdentity::Mac {
        timestamp_bits: timestamp.to_bits(),
        key_code,
        event_type,
        window: window_number.max(0) as u64,
    })
}

fn event_modifiers(event: *mut Object) -> Modifiers {
    let flags: u64 = unsafe { msg_send![event, modifierFlags] };
    Modifiers {
        control: flags & CONTROL != 0,
        alt: flags & OPTION != 0,
        shift: flags & SHIFT != 0,
        platform: flags & COMMAND != 0,
        function: flags & FUNCTION != 0,
    }
}

fn mouse_input(view: &Object, event: *mut Object, down: bool, dragged: bool) -> PlatformInput {
    let position = event_position(view, event);
    let modifiers = event_modifiers(event);
    let button = MouseButton::Left;
    let click_count: usize = unsafe { msg_send![event, clickCount] };
    if dragged {
        PlatformInput::MouseMove(MouseMoveEvent {
            position,
            pressed_button: Some(button),
            modifiers,
        })
    } else if down {
        PlatformInput::MouseDown(MouseDownEvent {
            button,
            position,
            modifiers,
            click_count,
            first_mouse: true,
        })
    } else {
        PlatformInput::MouseUp(MouseUpEvent {
            button,
            position,
            modifiers,
            click_count,
        })
    }
}

fn ns_string(value: *mut Object) -> Option<String> {
    if value.is_null() {
        return None;
    }
    unsafe {
        let bytes: *const i8 = msg_send![value, UTF8String];
        if bytes.is_null() {
            None
        } else {
            std::ffi::CStr::from_ptr(bytes)
                .to_str()
                .ok()
                .map(str::to_owned)
        }
    }
}

unsafe fn cocoa_string(value: &str) -> Option<NonNull<Object>> {
    let allocated: *mut Object = msg_send![class!(NSString), alloc];
    let initialized: *mut Object = msg_send![
        allocated,
        initWithBytes: value.as_ptr().cast::<c_void>()
        length: value.len()
        encoding: 4_usize
    ];
    NonNull::new(initialized)
}

fn key_name(key_char: Option<&str>) -> String {
    let Some(value) = key_char else {
        return "unknown".to_string();
    };
    match value.chars().next() {
        Some('\u{f700}') => "up".to_string(),
        Some('\u{f701}') => "down".to_string(),
        Some('\u{f702}') => "left".to_string(),
        Some('\u{f703}') => "right".to_string(),
        Some('\u{f728}') => "backspace".to_string(),
        Some('\u{f729}') => "home".to_string(),
        Some('\u{f72b}') => "end".to_string(),
        Some('\u{1b}') => "escape".to_string(),
        Some('\r' | '\n') => "enter".to_string(),
        Some('\t') => "tab".to_string(),
        Some(' ') => "space".to_string(),
        Some(character) => character.to_lowercase().collect(),
        None => value.to_string(),
    }
}

fn range_option(range: NSRange) -> Option<std::ops::Range<usize>> {
    (range.location != NS_NOT_FOUND)
        .then_some(range.location..range.location.saturating_add(range.length))
}

#[cfg(test)]
mod tests {
    use super::key_name;

    #[test]
    fn key_name_is_stable_for_key_down_and_key_up_characters() {
        for (characters, expected) in [
            (Some(" "), "space"),
            (Some("\r"), "enter"),
            (Some("\n"), "enter"),
            (Some("A"), "a"),
            (Some("z"), "z"),
        ] {
            assert_eq!(key_name(characters), expected);
        }
        assert_eq!(key_name(None), "unknown");
    }
}
