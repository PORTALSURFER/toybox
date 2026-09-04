//! Radiant-backed Win32 host for embedded CLAP and VST3 editor views.
#![allow(
    clippy::missing_docs_in_private_items,
    unexpected_cfgs,
    unsafe_op_in_unsafe_fn
)]

use std::cell::{Cell, RefCell};
use std::marker::PhantomData;
use std::num::NonZeroIsize;
use std::rc::Rc;
use std::thread::{self, ThreadId};

use radiant::gui::types::{Point, Vector2};
use radiant::runtime::{
    EmbeddedVelloRenderer, EmbeddedVelloSurfaceHandle, Event, NativeTextOptions, Renderer,
    SurfacePaintPlan,
};
use radiant::theme::DpiScale;
use radiant::widgets::{KeyboardModifiers, PointerButton, PointerModifiers, WidgetKey};
use raw_window_handle_06::{
    RawDisplayHandle as RawDisplayHandle06, RawWindowHandle as RawWindowHandle06,
    Win32WindowHandle as Win32WindowHandle06, WindowsDisplayHandle,
};
use windows::Win32::Foundation::{HINSTANCE, HMODULE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, InvalidateRect, PAINTSTRUCT};
use windows::Win32::System::LibraryLoader::{
    GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
    GetModuleHandleExW, GetModuleHandleW,
};
use windows::Win32::UI::Controls::WM_MOUSELEAVE;
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetCapture, GetFocus, GetKeyState, ReleaseCapture, SetCapture, SetFocus, TME_LEAVE,
    TRACKMOUSEEVENT, TrackMouseEvent, VK_BACK, VK_CONTROL, VK_DELETE, VK_DOWN, VK_END, VK_ESCAPE,
    VK_HOME, VK_LEFT, VK_MENU, VK_RETURN, VK_RIGHT, VK_SHIFT, VK_SPACE, VK_TAB, VK_UP,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CS_DBLCLKS, CS_HREDRAW, CS_VREDRAW, CreateWindowExW, DLGC_WANTALLKEYS, DLGC_WANTCHARS,
    DefWindowProcW, DestroyWindow, GWLP_USERDATA, GetClientRect, IsWindow, LoadCursorW,
    MA_ACTIVATE, RegisterClassW, SW_HIDE, SW_SHOW, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOZORDER,
    SetParent, SetTimer, SetWindowLongPtrW, SetWindowPos, ShowWindow, WM_CANCELMODE,
    WM_CAPTURECHANGED, WM_CHAR, WM_DPICHANGED, WM_DPICHANGED_AFTERPARENT, WM_ERASEBKGND,
    WM_GETDLGCODE, WM_KEYDOWN, WM_KEYUP, WM_KILLFOCUS, WM_LBUTTONDBLCLK, WM_LBUTTONDOWN,
    WM_LBUTTONUP, WM_MBUTTONDBLCLK, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEACTIVATE, WM_MOUSEMOVE,
    WM_MOUSEWHEEL, WM_NCDESTROY, WM_NCHITTEST, WM_PAINT, WM_RBUTTONDBLCLK, WM_RBUTTONDOWN,
    WM_RBUTTONUP, WM_SETFOCUS, WM_SIZE, WM_SYSCHAR, WM_SYSKEYDOWN, WM_SYSKEYUP, WM_TIMER,
    WNDCLASSW, WS_CHILD, WS_CLIPCHILDREN, WS_CLIPSIBLINGS,
};
use windows::core::PCWSTR;

use super::vst3_key_down_to_input_char;
use super::{HostedGui, RadiantEditor, logical_size_to_physical, physical_size_to_logical};

const TIMER_ID: usize = 1;
const TIMER_INTERVAL_MS: u32 = 33;
const MK_SHIFT: u32 = 0x0004;
const MK_CONTROL: u32 = 0x0008;
const WM_UNICHAR: u32 = 0x0109;
const UNICODE_NOCHAR: usize = 0xffff;
const WM_CHAR_SURROGATE_MIN: u16 = 0xd800;
const WM_CHAR_SURROGATE_MAX: u16 = 0xdfff;
const WM_CHAR_HIGH_SURROGATE_MAX: u16 = 0xdbff;
const WHEEL_DELTA: f32 = 120.0;
const WHEEL_LINES: f32 = 40.0;
const MAX_CLASS_NAME_UNITS: usize = 128;

/// Registration is process-wide while each class is owned by this backend.
static WINDOW_CLASS_REGISTRATION: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Select the sole source of keyboard input for a hosted native child.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum KeyboardDeliveryMode {
    /// CLAP/native hosts deliver keyboard messages directly to the child HWND.
    #[default]
    Native,
    /// VST3 hosts deliver keyboard input through `IPlugView` callbacks.
    CallbackOnly,
}

impl KeyboardDeliveryMode {
    /// Return whether native keyboard messages must be swallowed.
    const fn suppresses_native_messages(self) -> bool {
        matches!(self, Self::CallbackOnly)
    }
}

/// Identify whether a DPI message supplies a scale or requires a window query.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DpiChangeKind {
    /// The message wParam contains the new DPI.
    MessageDpi,
    /// The child window contains the effective DPI after its parent changed.
    WindowDpi,
}

/// Classify the DPI messages handled by the embedded child window.
fn dpi_change_kind(message: u32) -> Option<DpiChangeKind> {
    match message {
        WM_DPICHANGED => Some(DpiChangeKind::MessageDpi),
        WM_DPICHANGED_AFTERPARENT => Some(DpiChangeKind::WindowDpi),
        _ => None,
    }
}

/// A native child window and the retained Radiant editor it owns.
struct WindowState {
    hwnd: HWND,
    editor: Option<Box<dyn RadiantEditor>>,
    renderer: Option<EmbeddedVelloRenderer>,
    orphaned_editor: Rc<RefCell<Option<Box<dyn RadiantEditor>>>>,
    state_token: Rc<Cell<Option<*mut WindowState>>>,
    size: Rc<Cell<Option<(u32, u32)>>>,
    dpi_scale: Rc<Cell<DpiScale>>,
    keyboard_mode: Rc<Cell<KeyboardDeliveryMode>>,
    active_button: Option<PointerButton>,
    tracking_mouse: bool,
    cancellation_in_progress: bool,
    pending_high_surrogate: Option<u16>,
    last_renderer_size: Option<(u32, u32, DpiScale)>,
}

impl WindowState {
    /// Build state before the renderer is initialized from the child HWND.
    fn new(
        hwnd: HWND,
        editor: Box<dyn RadiantEditor>,
        orphaned_editor: Rc<RefCell<Option<Box<dyn RadiantEditor>>>>,
        state_token: Rc<Cell<Option<*mut WindowState>>>,
        size: Rc<Cell<Option<(u32, u32)>>>,
        dpi_scale: Rc<Cell<DpiScale>>,
        keyboard_mode: Rc<Cell<KeyboardDeliveryMode>>,
    ) -> Self {
        Self {
            hwnd,
            editor: Some(editor),
            renderer: None,
            orphaned_editor,
            state_token,
            size,
            dpi_scale,
            keyboard_mode,
            active_button: None,
            tracking_mouse: false,
            cancellation_in_progress: false,
            pending_high_surrogate: None,
            last_renderer_size: None,
        }
    }

    /// Initialize the embedded renderer and synchronize the actual client size.
    unsafe fn initialize_renderer(
        &mut self,
        module: HINSTANCE,
        text_options: &NativeTextOptions,
    ) -> bool {
        let (width, height) = client_size(self.hwnd).unwrap_or((1, 1));
        let dpi_scale = self.dpi_scale.get();
        let logical_size = logical_size_for_renderer(width, height, dpi_scale);
        let Some(handle) = embedded_surface_handle(self.hwnd, module) else {
            return false;
        };
        let Ok(renderer) = (unsafe {
            EmbeddedVelloRenderer::new_with_text_options(
                handle,
                logical_size,
                dpi_scale,
                text_options,
            )
        }) else {
            return false;
        };
        self.renderer = Some(renderer);
        self.resize_physical(width, height);
        true
    }

    /// Resize Radiant in logical points while keeping the renderer in physical pixels.
    fn resize_physical(&mut self, width: u32, height: u32) {
        let width = width.max(1);
        let height = height.max(1);
        let dpi_scale = self.dpi_scale.get();
        if self.last_renderer_size == Some((width, height, dpi_scale)) {
            self.size.set(Some((width, height)));
            return;
        }
        let (logical_width, logical_height) = physical_size_to_logical(width, height, dpi_scale);
        let logical_size = logical_size_for_renderer(width, height, dpi_scale);
        if let Some(editor) = self.editor.as_mut() {
            editor.resize(logical_width, logical_height);
        }
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.resize(logical_size, dpi_scale);
        }
        self.size.set(Some((width, height)));
        self.last_renderer_size = Some((width, height, dpi_scale));
        self.invalidate();
    }

    /// Render one bounded paint callback on the creating Windows UI thread.
    fn paint(&mut self) {
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };
        let Some(editor) = self.editor.as_mut() else {
            return;
        };
        let plan: &SurfacePaintPlan = editor.paint_plan();
        let _ = renderer.render(plan);
    }

    /// Return whether the editor requested the next timer-driven frame.
    fn needs_realtime_redraw(&self) -> bool {
        self.editor
            .as_ref()
            .is_some_and(|editor| editor.needs_realtime_redraw())
    }

    /// Schedule one native paint without creating a worker thread.
    fn invalidate(&self) {
        unsafe {
            let _ = InvalidateRect(Some(self.hwnd), None, false);
        }
    }

    /// Release mouse capture only when this child still owns it.
    fn release_capture_if_owned(&self) {
        unsafe {
            if GetCapture() == self.hwnd {
                let _ = ReleaseCapture();
            }
        }
    }

    /// Convert a physical client point to Radiant logical coordinates.
    fn logical_point(&self, x: i32, y: i32) -> Point {
        let dpi_scale = self.dpi_scale.get();
        Point::new(
            dpi_scale.physical_to_logical(x as f32),
            dpi_scale.physical_to_logical(y as f32),
        )
    }

    /// Read the modifier state associated with a native mouse message.
    fn native_modifiers(&self, wparam: WPARAM) -> PointerModifiers {
        let flags = wparam.0 as u32;
        let control = flags & MK_CONTROL != 0 || unsafe { GetKeyState(VK_CONTROL.0 as i32) } < 0;
        let shift = flags & MK_SHIFT != 0 || unsafe { GetKeyState(VK_SHIFT.0 as i32) } < 0;
        let alt = unsafe { GetKeyState(VK_MENU.0 as i32) } < 0;
        PointerModifiers {
            command: control,
            shift,
            alt,
        }
    }

    /// Read keyboard modifiers without interpreting a virtual key as mouse flags.
    fn keyboard_modifiers() -> PointerModifiers {
        PointerModifiers {
            command: unsafe { GetKeyState(VK_CONTROL.0 as i32) } < 0,
            shift: unsafe { GetKeyState(VK_SHIFT.0 as i32) } < 0,
            alt: unsafe { GetKeyState(VK_MENU.0 as i32) } < 0,
        }
    }

    /// Read native keyboard modifiers without projecting Control into command.
    fn native_keyboard_modifiers(pointer_modifiers: PointerModifiers) -> KeyboardModifiers {
        KeyboardModifiers {
            command: false,
            control: pointer_modifiers.command,
            shift: pointer_modifiers.shift,
            alt: pointer_modifiers.alt,
        }
    }

    /// Read the signed client coordinates encoded in a mouse LPARAM.
    fn mouse_position(lparam: LPARAM) -> (i32, i32) {
        let value = lparam.0 as u64;
        (
            (((value & 0xffff) as u16) as i16).into(),
            (((value >> 16) as u16) as i16).into(),
        )
    }

    /// Dispatch the current modifier state before a pointer event.
    fn dispatch_modifiers(&mut self, modifiers: PointerModifiers) {
        if let Some(editor) = self.editor.as_mut() {
            editor.dispatch_event(Event::pointer_modifiers_changed(modifiers));
        }
    }

    /// Dispatch a pointer movement in logical coordinates.
    fn pointer_move(&mut self, x: i32, y: i32, modifiers: PointerModifiers) {
        self.dispatch_modifiers(modifiers);
        let position = self.logical_point(x, y);
        if let Some(editor) = self.editor.as_mut() {
            editor.dispatch_event(Event::pointer_move(position));
        }
        self.invalidate();
    }

    /// Clear hover state at the exact logical leave sentinel used by Radiant.
    fn pointer_leave(&mut self, modifiers: PointerModifiers) {
        self.dispatch_modifiers(modifiers);
        if let Some(editor) = self.editor.as_mut() {
            editor.dispatch_event(Event::pointer_move(Point::new(-1.0, -1.0)));
        }
        self.invalidate();
    }

    /// Dispatch a button press or double-click and take native mouse capture.
    fn pointer_press(
        &mut self,
        x: i32,
        y: i32,
        button: PointerButton,
        modifiers: PointerModifiers,
        double_click: bool,
    ) {
        // A new press must not orphan the previous native gesture if a host
        // delivered an overlapping button-down sequence.
        if self.active_button.is_some() {
            self.cancel_native_interaction(false);
        }
        self.active_button = Some(button);
        unsafe {
            let _ = SetFocus(Some(self.hwnd));
            let _ = SetCapture(self.hwnd);
        }
        self.dispatch_modifiers(modifiers);
        let position = self.logical_point(x, y);
        if let Some(editor) = self.editor.as_mut() {
            let event = if double_click {
                Event::pointer_double_click(position, button, modifiers)
            } else {
                Event::pointer_press(position, button, modifiers)
            };
            editor.dispatch_event(event);
        }
        self.invalidate();
    }

    /// Dispatch a button release and release capture when the gesture ends.
    fn pointer_release(
        &mut self,
        x: i32,
        y: i32,
        button: PointerButton,
        modifiers: PointerModifiers,
    ) {
        // Native button-up messages can arrive out of order when another
        // button is pressed while this child owns capture. Only the button
        // that established the current gesture may end it; an unmatched
        // release must leave the active gesture and capture untouched.
        if self.active_button != Some(button) {
            return;
        }
        self.active_button = None;
        self.dispatch_modifiers(modifiers);
        let position = self.logical_point(x, y);
        if let Some(editor) = self.editor.as_mut() {
            editor.dispatch_event(Event::pointer_release(position, button, modifiers));
        }
        self.release_capture_if_owned();
        self.invalidate();
    }

    /// Dispatch one wheel sample using the native cursor location.
    fn pointer_wheel(&mut self, wparam: WPARAM) {
        let mut point = POINT::default();
        if unsafe { windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut point) }.is_err() {
            return;
        }
        if !unsafe { windows::Win32::Graphics::Gdi::ScreenToClient(self.hwnd, &mut point) }
            .as_bool()
        {
            return;
        }
        let delta = signed_word(wparam.0 as isize, 16) as f32 / WHEEL_DELTA * WHEEL_LINES;
        let modifiers = self.native_modifiers(wparam);
        self.dispatch_modifiers(modifiers);
        let position = self.logical_point(point.x, point.y);
        if let Some(editor) = self.editor.as_mut() {
            editor.dispatch_event(Event::Scroll {
                position,
                delta: Vector2::new(0.0, delta),
                modifiers,
                timestamp: None,
                sequence_range: None,
            });
        }
        self.invalidate();
    }

    /// Arm one WM_MOUSELEAVE notification for the native child.
    fn arm_mouse_leave(&mut self) {
        if self.tracking_mouse {
            return;
        }
        let mut tracking = TRACKMOUSEEVENT {
            cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
            dwFlags: TME_LEAVE,
            hwndTrack: self.hwnd,
            dwHoverTime: 0,
        };
        if unsafe { TrackMouseEvent(&mut tracking) }.is_ok() {
            self.tracking_mouse = true;
        }
    }

    /// Translate a native virtual key into a Radiant semantic key.
    fn widget_key(virtual_key: u16) -> Option<WidgetKey> {
        Some(match virtual_key {
            value if value == VK_RETURN.0 => WidgetKey::Enter,
            value if value == VK_TAB.0 => WidgetKey::Tab,
            value if value == VK_SPACE.0 => WidgetKey::Space,
            value if value == VK_BACK.0 => WidgetKey::Backspace,
            value if value == VK_DELETE.0 => WidgetKey::Delete,
            value if value == VK_LEFT.0 => WidgetKey::ArrowLeft,
            value if value == VK_RIGHT.0 => WidgetKey::ArrowRight,
            value if value == VK_UP.0 => WidgetKey::ArrowUp,
            value if value == VK_DOWN.0 => WidgetKey::ArrowDown,
            value if value == VK_HOME.0 => WidgetKey::Home,
            value if value == VK_END.0 => WidgetKey::End,
            _ => return None,
        })
    }

    /// Dispatch one native semantic key-down message.
    fn native_key_down(&mut self, virtual_key: u16) -> bool {
        let pointer_modifiers = Self::keyboard_modifiers();
        let keyboard_modifiers = Self::native_keyboard_modifiers(pointer_modifiers);
        self.dispatch_modifiers(pointer_modifiers);
        let handled = if virtual_key == VK_ESCAPE.0 {
            self.editor
                .as_mut()
                .is_some_and(|editor| editor.cancel_text_entry())
        } else if let Some(key) = Self::widget_key(virtual_key) {
            self.editor
                .as_mut()
                .is_some_and(|editor| editor.dispatch_key_press(key, keyboard_modifiers))
        } else {
            false
        };
        self.invalidate();
        handled
    }

    /// Dispatch one UTF-16 unit after assembling surrogate pairs.
    fn native_character_unit(&mut self, unit: u16) -> bool {
        if (WM_CHAR_SURROGATE_MIN..=WM_CHAR_SURROGATE_MAX).contains(&unit) {
            if unit <= WM_CHAR_HIGH_SURROGATE_MAX {
                self.pending_high_surrogate = Some(unit);
                return false;
            }
            if let Some(high) = self.pending_high_surrogate.take()
                && let Some(character) = utf16_surrogate_pair_to_char(high, unit)
            {
                return self.native_character(character);
            }
            return false;
        }
        if let Some(high) = self.pending_high_surrogate.take()
            && let Some(character) = char::from_u32(u32::from(high))
        {
            let _ = self.native_character(character);
        }
        char::from_u32(u32::from(unit)).is_some_and(|character| self.native_character(character))
    }

    /// Dispatch one VST3 UTF-16 callback unit after assembling surrogate pairs.
    fn callback_character_unit(
        &mut self,
        unit: u16,
        pointer_modifiers: PointerModifiers,
        keyboard_modifiers: KeyboardModifiers,
    ) -> bool {
        if (WM_CHAR_SURROGATE_MIN..=WM_CHAR_SURROGATE_MAX).contains(&unit) {
            if unit <= WM_CHAR_HIGH_SURROGATE_MAX {
                self.pending_high_surrogate = Some(unit);
                return false;
            }
            if let Some(high) = self.pending_high_surrogate.take()
                && let Some(character) = utf16_surrogate_pair_to_char(high, unit)
            {
                return self.dispatch_callback_character(
                    character,
                    pointer_modifiers,
                    keyboard_modifiers,
                );
            }
            return false;
        }
        self.pending_high_surrogate = None;
        char::from_u32(u32::from(unit)).is_some_and(|character| {
            self.dispatch_callback_character(character, pointer_modifiers, keyboard_modifiers)
        })
    }

    /// Dispatch a complete VST3 callback character with host-provided modifiers.
    fn dispatch_callback_character(
        &mut self,
        character: char,
        pointer_modifiers: PointerModifiers,
        keyboard_modifiers: KeyboardModifiers,
    ) -> bool {
        self.editor.as_mut().is_some_and(|editor| {
            dispatch_key_character(
                editor.as_mut(),
                character,
                pointer_modifiers,
                keyboard_modifiers,
            )
        })
    }

    /// Dispatch one native character or command shortcut.
    fn native_character(&mut self, character: char) -> bool {
        if matches!(
            character,
            '\u{8}' | '\u{9}' | '\u{a}' | '\u{d}' | '\u{1b}' | '\u{7f}'
        ) || character.is_control()
        {
            return false;
        }
        let modifiers = Self::keyboard_modifiers();
        let handled = self.editor.as_mut().is_some_and(|editor| {
            if modifiers.command && !modifiers.alt {
                editor.dispatch_shortcut(character, modifiers)
            } else {
                editor.dispatch_character(character)
            }
        });
        self.invalidate();
        handled
    }

    /// Dispatch one host VST3 key-down callback through this window's state.
    fn host_key_down(&mut self, key: u16, key_code: i16, modifiers: i16) -> bool {
        if !self.keyboard_mode.get().suppresses_native_messages() {
            return false;
        }
        let pointer_modifiers = vst3_pointer_modifiers(modifiers);
        let keyboard_modifiers = vst3_keyboard_modifiers(modifiers);
        if key_code == 0 {
            self.dispatch_modifiers(pointer_modifiers);
            let handled = self.callback_character_unit(key, pointer_modifiers, keyboard_modifiers);
            self.invalidate();
            return handled;
        }
        self.pending_high_surrogate = None;
        let handled = dispatch_vst3_key_down(
            self.editor.as_mut().map(Box::as_mut),
            key,
            key_code,
            modifiers,
        );
        self.invalidate();
        handled
    }

    /// Dispatch one host VST3 key-up callback as a modifier update.
    fn host_key_up(&mut self, modifiers: i16) -> bool {
        if !self.keyboard_mode.get().suppresses_native_messages() {
            return false;
        }
        if let Some(editor) = self.editor.as_mut() {
            dispatch_vst3_key_up(editor.as_mut(), modifiers);
            self.invalidate();
        }
        false
    }

    /// Apply a host focus request to the child window without crossing threads.
    fn host_focus(&mut self, focused: bool) -> bool {
        if focused {
            unsafe {
                let _ = SetFocus(Some(self.hwnd));
                GetFocus() == self.hwnd
            }
        } else if unsafe { GetFocus() == self.hwnd } {
            unsafe {
                let _ = SetFocus(None);
                GetFocus() != self.hwnd
            }
        } else {
            true
        }
    }

    /// Cancel Radiant pointer/focus state when Windows revokes native focus or capture.
    fn cancel_native_interaction(&mut self, clear_runtime_focus: bool) {
        self.cancel_native_interaction_with_redraw(clear_runtime_focus, true);
    }

    /// Cancel native interaction without invalidating a window being torn down.
    fn cancel_before_teardown(&mut self) {
        self.cancel_native_interaction_with_redraw(true, false);
    }

    /// Dispatch the cancellation contract before releasing native capture.
    fn cancel_native_interaction_with_redraw(&mut self, clear_runtime_focus: bool, redraw: bool) {
        if self.cancellation_in_progress {
            return;
        }
        self.cancellation_in_progress = true;
        let had_active_button = self.active_button.take().is_some();
        self.tracking_mouse = false;
        if let Some(editor) = self.editor.as_mut() {
            if had_active_button {
                // Radiant must see cancellation while native capture still
                // belongs to this child; releasing it first can re-enter the
                // window procedure and lose the active gesture's owner.
                editor.dispatch_event(Event::pointer_capture_cancelled());
            }
            if clear_runtime_focus {
                editor.dispatch_event(Event::clear_focus());
            }
        }
        self.release_capture_if_owned();
        if redraw {
            self.invalidate();
        }
        self.cancellation_in_progress = false;
    }

    /// Apply an effective DPI and resize from the host-authoritative client area.
    fn apply_dpi_change(&mut self, dpi_scale: DpiScale) {
        self.dpi_scale.set(dpi_scale);
        let (width, height) = client_size(self.hwnd)
            .or_else(|| self.size.get())
            .unwrap_or((1, 1));
        self.resize_physical(width, height);
    }

    /// Update the effective DPI from the DPI supplied by WM_DPICHANGED.
    fn dpi_changed(&mut self, wparam: WPARAM, _lparam: LPARAM) {
        let dpi = (wparam.0 as u32 & 0xffff).max(1);
        self.apply_dpi_change(DpiScale::new(f64::from(dpi) / f64::from(96_u32)));
    }

    /// Update the effective DPI after the parent has completed its DPI change.
    fn dpi_changed_after_parent(&mut self) {
        self.apply_dpi_change(window_dpi(self.hwnd));
    }

    /// Process one message on the creating thread and return a handled result.
    unsafe fn handle_message(
        &mut self,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> Option<LRESULT> {
        if suppresses_native_keyboard_message(self.keyboard_mode.get(), message) {
            return Some(LRESULT(0));
        }
        if let Some(kind) = dpi_change_kind(message) {
            match kind {
                DpiChangeKind::MessageDpi => self.dpi_changed(wparam, lparam),
                DpiChangeKind::WindowDpi => self.dpi_changed_after_parent(),
            }
            return Some(LRESULT(0));
        }
        match message {
            WM_PAINT => {
                let mut paint = PAINTSTRUCT::default();
                unsafe {
                    let _ = BeginPaint(self.hwnd, &mut paint);
                }
                self.paint();
                unsafe {
                    let _ = EndPaint(self.hwnd, &paint);
                }
                Some(LRESULT(0))
            }
            WM_ERASEBKGND => Some(LRESULT(1)),
            WM_TIMER if wparam.0 == TIMER_ID => {
                if self.needs_realtime_redraw() {
                    self.invalidate();
                }
                Some(LRESULT(0))
            }
            WM_SIZE => {
                let (width, height) = client_size(self.hwnd).unwrap_or_else(|| {
                    (
                        (lparam.0 as u32 & 0xffff).max(1),
                        ((lparam.0 as u32 >> 16) & 0xffff).max(1),
                    )
                });
                self.resize_physical(width, height);
                Some(LRESULT(0))
            }
            WM_MOUSEMOVE => {
                let (x, y) = Self::mouse_position(lparam);
                self.arm_mouse_leave();
                self.pointer_move(x, y, self.native_modifiers(wparam));
                Some(LRESULT(0))
            }
            WM_MOUSELEAVE => {
                self.tracking_mouse = false;
                self.pointer_leave(self.native_modifiers(wparam));
                Some(LRESULT(0))
            }
            WM_LBUTTONDOWN | WM_LBUTTONDBLCLK => {
                let (x, y) = Self::mouse_position(lparam);
                self.pointer_press(
                    x,
                    y,
                    PointerButton::Primary,
                    self.native_modifiers(wparam),
                    message == WM_LBUTTONDBLCLK,
                );
                Some(LRESULT(0))
            }
            WM_LBUTTONUP => {
                let (x, y) = Self::mouse_position(lparam);
                self.pointer_release(x, y, PointerButton::Primary, self.native_modifiers(wparam));
                Some(LRESULT(0))
            }
            WM_RBUTTONDOWN | WM_RBUTTONDBLCLK => {
                let (x, y) = Self::mouse_position(lparam);
                self.pointer_press(
                    x,
                    y,
                    PointerButton::Secondary,
                    self.native_modifiers(wparam),
                    message == WM_RBUTTONDBLCLK,
                );
                Some(LRESULT(0))
            }
            WM_RBUTTONUP => {
                let (x, y) = Self::mouse_position(lparam);
                self.pointer_release(
                    x,
                    y,
                    PointerButton::Secondary,
                    self.native_modifiers(wparam),
                );
                Some(LRESULT(0))
            }
            WM_MBUTTONDOWN | WM_MBUTTONDBLCLK => {
                let (x, y) = Self::mouse_position(lparam);
                self.pointer_press(
                    x,
                    y,
                    PointerButton::Auxiliary,
                    self.native_modifiers(wparam),
                    message == WM_MBUTTONDBLCLK,
                );
                Some(LRESULT(0))
            }
            WM_MBUTTONUP => {
                let (x, y) = Self::mouse_position(lparam);
                self.pointer_release(
                    x,
                    y,
                    PointerButton::Auxiliary,
                    self.native_modifiers(wparam),
                );
                Some(LRESULT(0))
            }
            WM_CANCELMODE | WM_CAPTURECHANGED => {
                self.cancel_native_interaction(false);
                Some(LRESULT(0))
            }
            WM_MOUSEWHEEL => {
                self.pointer_wheel(wparam);
                Some(LRESULT(0))
            }
            WM_SETFOCUS => {
                self.invalidate();
                Some(LRESULT(0))
            }
            WM_KILLFOCUS => {
                self.cancel_native_interaction(true);
                Some(LRESULT(0))
            }
            WM_GETDLGCODE => Some(LRESULT(dialog_code_for_keyboard_mode(
                self.keyboard_mode.get(),
            ))),
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                let virtual_key = wparam.0 as u16;
                let _ = self.native_key_down(virtual_key);
                Some(LRESULT(0))
            }
            WM_KEYUP | WM_SYSKEYUP => Some(LRESULT(0)),
            WM_CHAR | WM_SYSCHAR => {
                let _ = self.native_character_unit(wparam.0 as u16);
                Some(LRESULT(0))
            }
            WM_UNICHAR if wparam.0 == UNICODE_NOCHAR => Some(LRESULT(1)),
            WM_UNICHAR => {
                if let Some(character) = char::from_u32(wparam.0 as u32) {
                    let _ = self.native_character(character);
                }
                Some(LRESULT(0))
            }
            WM_MOUSEACTIVATE => Some(LRESULT(MA_ACTIVATE as isize)),
            WM_NCHITTEST => Some(LRESULT(1)),
            _ => None,
        }
    }
}

impl Drop for WindowState {
    /// Stop native timer/capture before dropping the renderer and editor.
    fn drop(&mut self) {
        // Parent destruction and WM_NCDESTROY can bypass the normal close
        // path. Cancel while the editor is still retained so an abandoned
        // gesture cannot survive into a later reopen.
        self.cancel_before_teardown();
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::KillTimer(Some(self.hwnd), TIMER_ID);
        }
        self.state_token.set(None);
        drop(self.renderer.take());
        if let Some(editor) = self.editor.take() {
            let mut orphaned_editor = self.orphaned_editor.borrow_mut();
            if orphaned_editor.is_none() {
                *orphaned_editor = Some(editor);
            }
        }
    }
}

/// Windows Radiant host whose editor and renderer remain on one UI thread.
pub(crate) struct RadiantWindowsHostedGui {
    parent: Option<HWND>,
    hwnd: Option<HWND>,
    size: Rc<Cell<Option<(u32, u32)>>>,
    dpi_scale: Rc<Cell<DpiScale>>,
    explicit_size: Cell<Option<(u32, u32)>>,
    default_logical_size: Cell<(u32, u32)>,
    class_name: &'static str,
    editor: Option<Box<dyn RadiantEditor>>,
    orphaned_editor: Rc<RefCell<Option<Box<dyn RadiantEditor>>>>,
    state_token: Rc<Cell<Option<*mut WindowState>>>,
    keyboard_mode: Rc<Cell<KeyboardDeliveryMode>>,
    text_options: NativeTextOptions,
    owner_thread: Option<ThreadId>,
    /// Make the !Send UI-thread contract explicit even for Send editors.
    not_send: PhantomData<Rc<RefCell<()>>>,
}

impl RadiantWindowsHostedGui {
    /// Create a reusable host with a retained editor awaiting a parent HWND.
    pub(crate) fn new(
        class_name: &'static str,
        editor: impl RadiantEditor,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            parent: None,
            hwnd: None,
            size: Rc::new(Cell::new(None)),
            dpi_scale: Rc::new(Cell::new(DpiScale::ONE)),
            explicit_size: Cell::new(None),
            default_logical_size: Cell::new((width.max(1), height.max(1))),
            class_name,
            editor: Some(Box::new(editor)),
            orphaned_editor: Rc::new(RefCell::new(None)),
            state_token: Rc::new(Cell::new(None)),
            keyboard_mode: Rc::new(Cell::new(KeyboardDeliveryMode::Native)),
            text_options: super::bundled_text_options(),
            // The constructor establishes the UI-affinity owner. This keeps
            // every later `&self` mutation (resize, show, focus, and input)
            // rejectable before a native child exists as well as afterward.
            owner_thread: Some(thread::current().id()),
            not_send: PhantomData,
        }
    }

    /// Configure embedded font options for the Windows renderer.
    pub(crate) fn with_text_options(mut self, options: NativeTextOptions) -> Self {
        self.text_options = options;
        self
    }

    /// Claim the creating UI thread or reject a cross-thread operation.
    fn claim_owner_thread(&mut self) -> bool {
        let current = thread::current().id();
        match self.owner_thread {
            Some(owner) => owner == current,
            None => {
                self.owner_thread = Some(current);
                true
            }
        }
    }

    /// Check whether native operations are being requested on the owner thread.
    fn is_owner_thread(&self) -> bool {
        self.owner_thread
            .is_none_or(|owner| owner == thread::current().id())
    }

    /// Return the current host-facing size or the default at the effective DPI.
    fn hosted_size(&self) -> Option<(u32, u32)> {
        self.size.get().or_else(|| {
            let (width, height) = self.default_logical_size.get();
            Some(logical_size_to_physical(
                width,
                height,
                self.dpi_scale.get(),
            ))
        })
    }

    /// Return the live child state associated with an HWND.
    unsafe fn state_ptr(hwnd: HWND) -> Option<*mut WindowState> {
        let pointer = unsafe {
            windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(hwnd, GWLP_USERDATA)
        };
        (pointer != 0).then_some(pointer as *mut WindowState)
    }

    /// Return state only when the HWND still belongs to this hosted instance.
    fn state_ptr_matches(&self, hwnd: HWND) -> Option<*mut WindowState> {
        let expected = self.state_token.get()?;
        let window_exists = unsafe { IsWindow(Some(hwnd)) }.as_bool();
        if !window_exists {
            return None;
        }
        let actual = unsafe { Self::state_ptr(hwnd) }?;
        state_pointer_matches(window_exists, Some(expected), Some(actual)).then_some(expected)
    }

    /// Return the live state pointer for the current child HWND.
    fn live_state_ptr(&self) -> Option<*mut WindowState> {
        self.hwnd.and_then(|hwnd| self.state_ptr_matches(hwnd))
    }

    /// Recover editor ownership after WM_NCDESTROY ran before a close request.
    fn recover_orphaned_editor(&mut self) {
        if self.editor.is_none() {
            self.editor = self.orphaned_editor.borrow_mut().take();
        }
    }

    /// Fence a parent-destroyed or otherwise stale child without touching a reused HWND.
    fn clear_dead_window(&mut self) {
        if let Some(hwnd) = self.hwnd {
            if self.live_state_ptr().is_none() {
                self.hwnd = None;
                self.reclaim_dead_state(Some(hwnd));
                self.recover_orphaned_editor();
            }
        } else if self.state_token.get().is_some() {
            // A defensive close path can clear `hwnd` before DestroyWindow
            // synchronously reaches WM_NCDESTROY. Do not strand that state.
            self.reclaim_dead_state(None);
            self.recover_orphaned_editor();
        }
    }

    /// Reclaim state after creation or renderer/timer initialization failed.
    fn discard_created_window(&mut self, hwnd: HWND, state_pointer: *mut WindowState) {
        // A reentrant parent destruction may already have run WM_NCDESTROY.
        // Never dereference the raw pointer unless the owner fence still names
        // this exact allocation.
        let owns_state = self.state_token.get() == Some(state_pointer);
        let editor = owns_state.then(|| unsafe {
            // Keep the editor present while cancellation is dispatched; the
            // state is about to be retained or destroyed after this failure.
            (*state_pointer).cancel_before_teardown();
            (*state_pointer).editor.take()
        });
        let state_destroyed = if owns_state {
            self.reclaim_dead_state(Some(hwnd))
        } else {
            false
        };
        if !state_destroyed {
            unsafe {
                if Self::state_ptr(hwnd) == Some(state_pointer) {
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                }
                let _ = DestroyWindow(hwnd);
            }
        }
        self.editor = editor
            .flatten()
            .or_else(|| self.orphaned_editor.borrow_mut().take());
    }

    /// Reclaim state whose child disappeared before the owner saw `close`.
    fn reclaim_dead_state(&mut self, hwnd: Option<HWND>) -> bool {
        let Some(state_pointer) = self.state_token.take() else {
            return false;
        };

        // A reentrant close can clear the outer `hwnd` before the original
        // DestroyWindow call has reached WM_NCDESTROY. Read the owned HWND
        // while the state is still alive so its userdata can be fenced before
        // the allocation is reclaimed.
        let state_hwnd = hwnd.or_else(|| unsafe { Some((*state_pointer).hwnd) });
        let mut destroy_window = None;
        if let Some(hwnd) = state_hwnd {
            let window_exists = unsafe { IsWindow(Some(hwnd)) }.as_bool();
            let actual = unsafe { Self::state_ptr(hwnd) };
            if actual == Some(state_pointer) {
                unsafe {
                    // The state is still installed only in this branch. Clear
                    // it before dropping so WM_NCDESTROY cannot double-drop it.
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                }
                destroy_window = window_exists.then_some(hwnd);
            }
        }

        unsafe {
            drop(Box::from_raw(state_pointer));
            if let Some(hwnd) = destroy_window {
                let _ = DestroyWindow(hwnd);
            }
        }
        destroy_window.is_some()
    }

    /// Attach a validated parent HWND and refresh its effective DPI.
    fn set_parent(&mut self, parent: raw_window_handle::Win32WindowHandle) {
        if !self.claim_owner_thread() {
            return;
        }
        let candidate = HWND(parent.hwnd);
        if parent.hwnd.is_null() || !unsafe { IsWindow(Some(candidate)) }.as_bool() {
            self.parent = None;
            return;
        }
        self.clear_dead_window();
        let previous_scale = self.dpi_scale.get();
        let next_scale = window_dpi(candidate);
        if self.hwnd.is_none() && next_scale != previous_scale {
            if let Some(size) = self.size.get() {
                self.size.set(Some(rescale_physical_size(
                    size,
                    previous_scale,
                    next_scale,
                )));
            }
            if let Some(size) = self.explicit_size.get() {
                self.explicit_size.set(Some(rescale_physical_size(
                    size,
                    previous_scale,
                    next_scale,
                )));
            }
        }
        self.dpi_scale.set(next_scale);
        if let Some(hwnd) = self.hwnd
            && unsafe { SetParent(hwnd, Some(candidate)) }.is_err()
        {
            self.dpi_scale.set(previous_scale);
            return;
        }
        self.parent = Some(candidate);
        if let Some(hwnd) = self.hwnd
            && let Some(pointer) = self.live_state_ptr()
        {
            let (width, height) = client_size(hwnd)
                .or_else(|| self.size.get())
                .unwrap_or((1, 1));
            unsafe { (*pointer).resize_physical(width, height) };
        }
    }

    /// Create the child HWND, install fenced state, and initialize Vello.
    fn open_view(&mut self) -> bool {
        if !self.claim_owner_thread() {
            return false;
        }
        self.clear_dead_window();
        self.recover_orphaned_editor();
        let Some(parent) = self.parent else {
            return false;
        };
        if !unsafe { IsWindow(Some(parent)) }.as_bool() {
            self.parent = None;
            return false;
        }
        if self.live_state_ptr().is_some() {
            return true;
        }
        let Some(editor) = self.editor.take() else {
            return false;
        };
        let Some(module) = (unsafe { module_handle() }) else {
            self.editor = Some(editor);
            return false;
        };
        let class_name = class_name_units(self.class_name);
        if class_name.is_empty() || class_name.len() > MAX_CLASS_NAME_UNITS {
            self.editor = Some(editor);
            return false;
        }
        if !unsafe { register_window_class(&class_name, module) } {
            self.editor = Some(editor);
            return false;
        }
        let (default_width, default_height) = self.default_logical_size.get();
        let (width, height) = self
            .size
            .get()
            .or_else(|| self.explicit_size.get())
            .unwrap_or_else(|| {
                logical_size_to_physical(default_width, default_height, self.dpi_scale.get())
            });
        let Ok(hwnd) = (unsafe {
            CreateWindowExW(
                Default::default(),
                PCWSTR(class_name.as_ptr()),
                PCWSTR(class_name.as_ptr()),
                WS_CHILD | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
                0,
                0,
                saturating_i32(width),
                saturating_i32(height),
                Some(parent),
                None,
                Some(module),
                None,
            )
        }) else {
            self.editor = Some(editor);
            return false;
        };
        let state = Box::new(WindowState::new(
            hwnd,
            editor,
            Rc::clone(&self.orphaned_editor),
            Rc::clone(&self.state_token),
            Rc::clone(&self.size),
            Rc::clone(&self.dpi_scale),
            Rc::clone(&self.keyboard_mode),
        ));
        let state_pointer = Box::into_raw(state);
        self.state_token.set(Some(state_pointer));
        let state_installed = unsafe {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_pointer as isize);
            Self::state_ptr(hwnd) == Some(state_pointer)
        };
        if !state_installed {
            self.discard_created_window(hwnd, state_pointer);
            return false;
        }
        let initialized =
            unsafe { (*state_pointer).initialize_renderer(module, &self.text_options) };
        if !initialized {
            self.discard_created_window(hwnd, state_pointer);
            return false;
        }
        let timer = unsafe { SetTimer(Some(hwnd), TIMER_ID, TIMER_INTERVAL_MS, None) };
        if timer == 0 {
            self.discard_created_window(hwnd, state_pointer);
            return false;
        }
        unsafe {
            let _ = ShowWindow(hwnd, SW_HIDE);
        }
        self.hwnd = Some(hwnd);
        self.size.set(Some(
            client_size(hwnd).unwrap_or((width.max(1), height.max(1))),
        ));
        true
    }

    /// Close the child and take the retained editor back out of fenced state.
    fn close_view(&mut self) {
        if !self.is_owner_thread() {
            return;
        }
        self.clear_dead_window();
        self.recover_orphaned_editor();
        let Some(hwnd) = self.hwnd else {
            return;
        };
        let Some(pointer) = self.state_ptr_matches(hwnd) else {
            self.hwnd = None;
            self.reclaim_dead_state(Some(hwnd));
            self.recover_orphaned_editor();
            return;
        };
        self.hwnd = None;
        // DestroyWindow may synchronously deliver capture/focus/NCDestroy
        // messages. Dispatch cancellation before taking the editor so those
        // messages cannot strand a Radiant pointer gesture.
        let editor = unsafe {
            (*pointer).cancel_before_teardown();
            (*pointer).editor.take()
        };
        let destroyed = unsafe { DestroyWindow(hwnd).is_ok() };
        if destroyed {
            // DestroyWindow normally synchronously delivers WM_NCDESTROY. Keep
            // the fence defensive in case a host implementation returns early.
            if self.state_token.get() == Some(pointer) {
                unsafe {
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                    drop(Box::from_raw(pointer));
                }
            }
            self.editor = editor.or_else(|| self.orphaned_editor.borrow_mut().take());
        } else {
            // Do not leave a renderer/editor allocation behind if the first
            // destroy attempt raced with native teardown. The reclaim path
            // clears userdata before dropping and retries destruction when the
            // state is still installed.
            self.reclaim_dead_state(Some(hwnd));
            self.editor = editor.or_else(|| self.orphaned_editor.borrow_mut().take());
        }
    }

    /// Show the existing child without recreating editor state.
    pub(crate) fn show(&self) -> bool {
        if !self.is_owner_thread() {
            return false;
        }
        let Some(hwnd) = self.hwnd else {
            return false;
        };
        if self.state_ptr_matches(hwnd).is_none() {
            return false;
        }
        unsafe {
            let _ = ShowWindow(hwnd, SW_SHOW);
            let _ = InvalidateRect(Some(hwnd), None, false);
        }
        true
    }

    /// Hide the existing child while retaining editor and renderer state.
    pub(crate) fn hide(&self) {
        if !self.is_owner_thread() {
            return;
        }
        if let Some(hwnd) = self
            .hwnd
            .filter(|hwnd| self.state_ptr_matches(*hwnd).is_some())
        {
            unsafe {
                let _ = ShowWindow(hwnd, SW_HIDE);
            }
        }
    }

    /// Apply an external UI scale exactly once at the renderer boundary.
    pub(crate) fn set_scale(&self, scale: f64) {
        if !self.is_owner_thread() {
            return;
        }
        let old_scale = self.dpi_scale.get();
        let new_scale = DpiScale::new(scale);
        if new_scale == old_scale {
            return;
        }
        let live_hwnd = self
            .hwnd
            .filter(|hwnd| self.state_ptr_matches(*hwnd).is_some());
        let current_physical = live_hwnd
            .and_then(client_size)
            .or_else(|| self.size.get())
            .unwrap_or_else(|| {
                let (width, height) = self.default_logical_size.get();
                logical_size_to_physical(width, height, old_scale)
            });
        let (width, height) = rescale_physical_size(current_physical, old_scale, new_scale);
        let previous_size = self.size.get();
        let previous_explicit_size = self.explicit_size.get();
        if let Some(size) = self.size.get() {
            self.size
                .set(Some(rescale_physical_size(size, old_scale, new_scale)));
        }
        if let Some(size) = self.explicit_size.get() {
            self.explicit_size
                .set(Some(rescale_physical_size(size, old_scale, new_scale)));
        }
        self.dpi_scale.set(new_scale);
        let Some(hwnd) = live_hwnd else {
            return;
        };
        let Some(pointer) = self.live_state_ptr() else {
            return;
        };
        unsafe {
            let resized = SetWindowPos(
                hwnd,
                None,
                0,
                0,
                saturating_i32(width),
                saturating_i32(height),
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            )
            .is_ok();
            if !resized {
                self.dpi_scale.set(old_scale);
                self.size.set(previous_size);
                self.explicit_size.set(previous_explicit_size);
                return;
            }
            if self.state_ptr_matches(hwnd) == Some(pointer) {
                (*pointer).resize_physical(width, height);
            }
        }
    }

    /// Forward a VST3 host key-down callback through the child state.
    fn forward_key_down(&self, key: u16, key_code: i16, modifiers: i16) -> bool {
        if !self.is_owner_thread() {
            return false;
        }
        let Some(pointer) = self.live_state_ptr() else {
            return false;
        };
        unsafe { (*pointer).host_key_down(key, key_code, modifiers) }
    }

    /// Forward a VST3 host key-up callback through the child state.
    fn forward_key_up(&self, _key: u16, _key_code: i16, modifiers: i16) -> bool {
        if !self.is_owner_thread() {
            return false;
        }
        let Some(pointer) = self.live_state_ptr() else {
            return false;
        };
        unsafe { (*pointer).host_key_up(modifiers) }
    }

    /// Forward a VST3 host focus callback to the native child HWND.
    fn forward_focus(&self, focused: bool) -> bool {
        if !self.is_owner_thread() {
            return false;
        }
        let Some(pointer) = self.live_state_ptr() else {
            return false;
        };
        unsafe { (*pointer).host_focus(focused) }
    }
}

impl Drop for RadiantWindowsHostedGui {
    /// Destroy the child on its owner thread and retain the editor when possible.
    fn drop(&mut self) {
        self.close_view();
    }
}

impl HostedGui for RadiantWindowsHostedGui {
    /// Attach the host-owned Win32 parent after validating it with IsWindow.
    fn set_parent_raw(&mut self, parent: raw_window_handle::RawWindowHandle) {
        if !self.claim_owner_thread() {
            return;
        }
        let raw_window_handle::RawWindowHandle::Win32(parent) = parent else {
            self.parent = None;
            return;
        };
        self.set_parent(parent);
    }

    /// Create the native child and initialize its renderer on the UI thread.
    fn open(&mut self) -> bool {
        self.open_view()
    }

    /// Remove the native child while retaining the editor for reopen.
    fn close(&mut self) {
        self.close_view();
    }

    /// Return the latest physical-pixel size exposed to the host.
    fn last_size(&self) -> Option<(u32, u32)> {
        self.hosted_size()
    }

    /// Replace the default logical size used before the first open.
    fn set_default_size(&mut self, width: u32, height: u32) {
        if !self.claim_owner_thread() {
            return;
        }
        self.default_logical_size.set((width.max(1), height.max(1)));
        if self.hwnd.is_none() {
            self.explicit_size.set(None);
            self.size.set(None);
        }
    }

    /// Select callback-only keyboard delivery for VST3 or native delivery for CLAP.
    fn set_callback_keyboard_mode(&mut self, callback_only: bool) {
        if !self.claim_owner_thread() {
            return;
        }
        let mode = if callback_only {
            KeyboardDeliveryMode::CallbackOnly
        } else {
            KeyboardDeliveryMode::Native
        };
        let previous = self.keyboard_mode.get();
        self.keyboard_mode.set(mode);
        if previous != mode
            && let Some(pointer) = self.live_state_ptr()
        {
            unsafe {
                (*pointer).pending_high_surrogate = None;
            }
        }
    }

    /// Show the already-open child view.
    fn show(&self) -> bool {
        RadiantWindowsHostedGui::show(self)
    }

    /// Convert logical dimensions to physical host pixels at effective DPI.
    fn host_size_from_logical(&self, width: u32, height: u32) -> (u32, u32) {
        logical_size_to_physical(width, height, self.dpi_scale.get())
    }

    /// Convert physical host pixels to logical editor dimensions at effective DPI.
    fn logical_size_from_host(&self, width: u32, height: u32) -> (u32, u32) {
        physical_size_to_logical(width, height, self.dpi_scale.get())
    }

    /// Request a local physical-pixel resize without host callback feedback.
    fn request_resize(&self, width: u32, height: u32) {
        if !self.is_owner_thread() {
            return;
        }
        let width = width.max(1);
        let height = height.max(1);
        self.explicit_size.set(Some((width, height)));
        let Some(hwnd) = self.hwnd else {
            self.size.set(Some((width, height)));
            return;
        };
        if self.live_state_ptr().is_none() {
            return;
        }
        unsafe {
            let _ = SetWindowPos(
                hwnd,
                None,
                0,
                0,
                saturating_i32(width),
                saturating_i32(height),
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
    }

    /// Forward one host key-down callback to the child state.
    fn on_key_down(&self, key: u16, key_code: i16, modifiers: i16) -> bool {
        self.forward_key_down(key, key_code, modifiers)
    }

    /// Forward one host key-up callback to the child state.
    fn on_key_up(&self, key: u16, key_code: i16, modifiers: i16) -> bool {
        self.forward_key_up(key, key_code, modifiers)
    }

    /// Forward one host focus callback to the child HWND.
    fn on_focus(&self, focused: bool) -> bool {
        self.forward_focus(focused)
    }
}

/// Decode a signed 16-bit word from a Win32 packed LPARAM/WPARAM.
fn signed_word(value: isize, shift: usize) -> i16 {
    (((value as u64 >> shift) & 0xffff) as u16) as i16
}

/// Clamp a host size to the signed Win32 coordinate range.
fn saturating_i32(value: u32) -> i32 {
    value.min(i32::MAX as u32) as i32
}

/// Decode one valid UTF-16 surrogate pair without passing invalid units to Radiant.
fn utf16_surrogate_pair_to_char(high: u16, low: u16) -> Option<char> {
    if !(WM_CHAR_SURROGATE_MIN..=WM_CHAR_HIGH_SURROGATE_MAX).contains(&high)
        || !(WM_CHAR_HIGH_SURROGATE_MAX + 1..=WM_CHAR_SURROGATE_MAX).contains(&low)
    {
        return None;
    }
    let scalar = 0x1_0000
        + (u32::from(high) - u32::from(WM_CHAR_SURROGATE_MIN)) * 0x400
        + (u32::from(low) - u32::from(WM_CHAR_HIGH_SURROGATE_MAX + 1));
    char::from_u32(scalar)
}

/// Return the effective DPI of a live host window, defaulting to 96 DPI.
fn window_dpi(hwnd: HWND) -> DpiScale {
    dpi_scale_from_dpi(unsafe { GetDpiForWindow(hwnd) })
}

/// Convert a Win32 DPI value into a sanitized Radiant scale.
fn dpi_scale_from_dpi(dpi: u32) -> DpiScale {
    if dpi == 0 {
        return DpiScale::ONE;
    }
    DpiScale::new(f64::from(dpi) / f64::from(96_u32))
}

/// Use fractional logical dimensions for the renderer while the editor API remains integer.
fn logical_size_for_renderer(width: u32, height: u32, dpi_scale: DpiScale) -> Vector2 {
    Vector2::new(
        dpi_scale.physical_to_logical(width.max(1) as f32),
        dpi_scale.physical_to_logical(height.max(1) as f32),
    )
}

/// Preserve logical dimensions while moving a physical size between DPIs.
fn rescale_physical_size(size: (u32, u32), from_scale: DpiScale, to_scale: DpiScale) -> (u32, u32) {
    let logical = physical_size_to_logical(size.0, size.1, from_scale);
    logical_size_to_physical(logical.0, logical.1, to_scale)
}

/// Return a positive child client size when Win32 reports a minimized zero size.
fn client_size(hwnd: HWND) -> Option<(u32, u32)> {
    let mut rect = RECT::default();
    unsafe { GetClientRect(hwnd, &mut rect) }.ok()?;
    Some((
        (rect.right - rect.left).max(1) as u32,
        (rect.bottom - rect.top).max(1) as u32,
    ))
}

/// Validate the incarnation fence before any raw window-state dereference.
fn state_pointer_matches(
    window_exists: bool,
    expected: Option<*mut WindowState>,
    actual: Option<*mut WindowState>,
) -> bool {
    window_exists && expected.is_some() && expected == actual
}

/// Build a bounded, null-terminated UTF-16 window class name.
fn class_name_units(name: &str) -> Vec<u16> {
    let mut units: Vec<u16> = name.encode_utf16().filter(|unit| *unit != 0).collect();
    if units.is_empty() {
        return Vec::new();
    }
    units.truncate(MAX_CLASS_NAME_UNITS - 1);
    units.push(0);
    units
}

/// Resolve this module's HINSTANCE for class registration and renderer handles.
unsafe fn module_handle() -> Option<HINSTANCE> {
    let mut module = HMODULE::default();
    if unsafe {
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            PCWSTR(window_proc as *const () as *const u16),
            &mut module,
        )
    }
    .is_err()
    {
        module = unsafe { GetModuleHandleW(None).ok()? };
    }
    Some(HINSTANCE(module.0))
}

/// Register one class whose WndProc owns the backend state fence.
unsafe fn register_window_class(name: &[u16], module: HINSTANCE) -> bool {
    let Ok(_guard) = WINDOW_CLASS_REGISTRATION.lock() else {
        return false;
    };
    let cursor = unsafe { LoadCursorW(None, windows::Win32::UI::WindowsAndMessaging::IDC_ARROW) }
        .unwrap_or_default();
    let class = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW | CS_DBLCLKS,
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: module,
        hIcon: Default::default(),
        hCursor: cursor,
        hbrBackground: Default::default(),
        lpszMenuName: PCWSTR::null(),
        lpszClassName: PCWSTR(name.as_ptr()),
    };
    (unsafe { RegisterClassW(&class) != 0 })
        || unsafe {
            windows::Win32::Foundation::GetLastError().0
                == windows::Win32::Foundation::WIN32_ERROR(1410).0
        }
}

/// Build the raw handle pair accepted by Radiant's embedded Vello renderer.
unsafe fn embedded_surface_handle(
    hwnd: HWND,
    module: HINSTANCE,
) -> Option<EmbeddedVelloSurfaceHandle> {
    let hwnd = NonZeroIsize::new(hwnd.0 as isize)?;
    let mut window = Win32WindowHandle06::new(hwnd);
    window.hinstance = NonZeroIsize::new(module.0 as isize);
    let display = RawDisplayHandle06::Windows(WindowsDisplayHandle::new());
    let window = RawWindowHandle06::Win32(window);
    Some(unsafe { EmbeddedVelloSurfaceHandle::from_raw(display, window) })
}

/// Convert VST3 modifier flags into a stable identity representation.
fn host_modifier_bits(modifiers: i16) -> u8 {
    #[cfg(feature = "vst3")]
    {
        use toybox_vst3_ffi::Steinberg::KeyModifier_::{
            kAlternateKey, kCommandKey, kControlKey, kShiftKey,
        };
        let modifiers = i64::from(modifiers);
        u8::from(modifiers & (kCommandKey as i64 | kControlKey as i64) != 0)
            | (u8::from(modifiers & kShiftKey as i64 != 0) << 1)
            | (u8::from(modifiers & kAlternateKey as i64 != 0) << 2)
    }
    #[cfg(not(feature = "vst3"))]
    {
        let modifiers = i64::from(modifiers);
        // VST3's modifier bits are part of the callback ABI, not the host
        // window system's modifier mask: shift=1, alternate=2, command=4,
        // control=8. On Windows, hosts normally report control for Ctrl.
        u8::from(modifiers & (4 | 8) != 0)
            | (u8::from(modifiers & 1 != 0) << 1)
            | (u8::from(modifiers & 2 != 0) << 2)
    }
}

/// Return whether a native keyboard message belongs to callback-only VST3 input.
fn suppresses_native_keyboard_message(mode: KeyboardDeliveryMode, message: u32) -> bool {
    mode.suppresses_native_messages()
        && matches!(
            message,
            WM_KEYDOWN | WM_SYSKEYDOWN | WM_KEYUP | WM_SYSKEYUP | WM_CHAR | WM_SYSCHAR | WM_UNICHAR
        )
}

/// Dispatch one VST3 callback using Steinberg virtual-key semantics.
fn dispatch_vst3_key_down(
    editor: Option<&mut dyn RadiantEditor>,
    key: u16,
    key_code: i16,
    modifiers: i16,
) -> bool {
    let Some(editor) = editor else {
        return false;
    };
    let pointer_modifiers = vst3_pointer_modifiers(modifiers);
    let keyboard_modifiers = vst3_keyboard_modifiers(modifiers);
    editor.dispatch_event(Event::pointer_modifiers_changed(pointer_modifiers));

    #[cfg(feature = "vst3")]
    {
        use toybox_vst3_ffi::Steinberg::VirtualKeyCodes_::{
            KEY_BACK, KEY_DELETE, KEY_DOWN, KEY_END, KEY_ENTER, KEY_ESCAPE, KEY_HOME, KEY_LEFT,
            KEY_RETURN, KEY_RIGHT, KEY_SPACE, KEY_TAB, KEY_UP,
        };

        let key_code = i64::from(key_code);
        let semantic_key = match key_code {
            value if value == KEY_ENTER as i64 || value == KEY_RETURN as i64 => {
                Some(WidgetKey::Enter)
            }
            value if value == KEY_TAB as i64 => Some(WidgetKey::Tab),
            value if value == KEY_BACK as i64 => Some(WidgetKey::Backspace),
            value if value == KEY_DELETE as i64 => Some(WidgetKey::Delete),
            value if value == KEY_SPACE as i64 => Some(WidgetKey::Space),
            value if value == KEY_LEFT as i64 => Some(WidgetKey::ArrowLeft),
            value if value == KEY_RIGHT as i64 => Some(WidgetKey::ArrowRight),
            value if value == KEY_UP as i64 => Some(WidgetKey::ArrowUp),
            value if value == KEY_DOWN as i64 => Some(WidgetKey::ArrowDown),
            value if value == KEY_HOME as i64 => Some(WidgetKey::Home),
            value if value == KEY_END as i64 => Some(WidgetKey::End),
            _ => None,
        };
        if let Some(key) = semantic_key {
            if pointer_modifiers.command {
                return false;
            }
            return editor.dispatch_key_press(key, keyboard_modifiers);
        }
        if key_code == KEY_ESCAPE as i64 {
            if pointer_modifiers.command {
                return false;
            }
            return editor.cancel_text_entry();
        }
    }

    let character = vst3_key_down_to_input_char(key, key_code);
    let Some(character) = character else {
        return false;
    };
    dispatch_key_character(editor, character, pointer_modifiers, keyboard_modifiers)
}

/// Dispatch one VST3 callback character as a semantic or text intent.
fn dispatch_key_character(
    editor: &mut dyn RadiantEditor,
    character: char,
    pointer_modifiers: PointerModifiers,
    keyboard_modifiers: KeyboardModifiers,
) -> bool {
    let semantic_key = match character {
        '\u{8}' => Some(WidgetKey::Backspace),
        '\u{7f}' => Some(WidgetKey::Delete),
        '\t' => Some(WidgetKey::Tab),
        '\r' | '\n' => Some(WidgetKey::Enter),
        ' ' => Some(WidgetKey::Space),
        '\u{1c}' => Some(WidgetKey::ArrowLeft),
        '\u{1d}' => Some(WidgetKey::ArrowRight),
        '\u{1e}' => Some(WidgetKey::ArrowUp),
        '\u{1f}' => Some(WidgetKey::ArrowDown),
        '\u{f729}' => Some(WidgetKey::Home),
        '\u{f72b}' => Some(WidgetKey::End),
        '\u{1b}' => {
            if pointer_modifiers.command {
                return false;
            }
            return editor.cancel_text_entry();
        }
        _ => None,
    };
    if let Some(key) = semantic_key {
        if pointer_modifiers.command {
            return false;
        }
        return editor.dispatch_key_press(key, keyboard_modifiers);
    }
    if character.is_control() {
        return false;
    }
    if pointer_modifiers.command {
        editor.dispatch_shortcut(character, pointer_modifiers)
    } else {
        editor.dispatch_character(character)
    }
}

/// Return the dialog-code response that keeps callback-only VST3 input out of Win32 delivery.
fn dialog_code_for_keyboard_mode(mode: KeyboardDeliveryMode) -> isize {
    if mode.suppresses_native_messages() {
        0
    } else {
        (DLGC_WANTALLKEYS | DLGC_WANTCHARS) as isize
    }
}

/// Dispatch a VST3 key-up as the latest modifier state.
fn dispatch_vst3_key_up(editor: &mut dyn RadiantEditor, modifiers: i16) {
    editor.dispatch_event(Event::pointer_modifiers_changed(vst3_pointer_modifiers(
        modifiers,
    )));
}

/// Convert VST3 modifiers into Radiant's platform-neutral modifier state.
fn vst3_pointer_modifiers(modifiers: i16) -> PointerModifiers {
    let modifiers = host_modifier_bits(modifiers);
    PointerModifiers {
        command: modifiers & 1 != 0,
        shift: modifiers & 2 != 0,
        alt: modifiers & 4 != 0,
    }
}

/// Convert VST3 modifiers into lossless semantic-key modifier state.
fn vst3_keyboard_modifiers(modifiers: i16) -> KeyboardModifiers {
    #[cfg(feature = "vst3")]
    {
        use toybox_vst3_ffi::Steinberg::KeyModifier_::{
            kAlternateKey, kCommandKey, kControlKey, kShiftKey,
        };
        let modifiers = i64::from(modifiers);
        KeyboardModifiers {
            command: modifiers & kCommandKey as i64 != 0,
            control: modifiers & kControlKey as i64 != 0,
            shift: modifiers & kShiftKey as i64 != 0,
            alt: modifiers & kAlternateKey as i64 != 0,
        }
    }
    #[cfg(not(feature = "vst3"))]
    {
        let modifiers = i64::from(modifiers);
        KeyboardModifiers {
            command: modifiers & 4 != 0,
            control: modifiers & 8 != 0,
            shift: modifiers & 1 != 0,
            alt: modifiers & 2 != 0,
        }
    }
}

/// Fence state lookup and drop at WM_NCDESTROY so stale HWND messages cannot use it.
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let pointer = unsafe { RadiantWindowsHostedGui::state_ptr(hwnd) };
    if let Some(pointer) = pointer
        && let Some(result) = unsafe { (*pointer).handle_message(message, wparam, lparam) }
    {
        return result;
    }
    if message == WM_NCDESTROY
        && let Some(pointer) = pointer
    {
        unsafe {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            drop(Box::from_raw(pointer));
        }
    }
    unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "vst3")]
    use std::cell::{Cell, RefCell};
    #[cfg(feature = "vst3")]
    use std::rc::Rc;

    #[cfg(feature = "vst3")]
    use super::dispatch_vst3_key_down;
    #[cfg(feature = "vst3")]
    use super::host_modifier_bits;
    use super::{
        DpiChangeKind, KeyboardDeliveryMode, WM_UNICHAR, dialog_code_for_keyboard_mode,
        dpi_change_kind, state_pointer_matches, suppresses_native_keyboard_message,
        utf16_surrogate_pair_to_char,
    };
    #[cfg(feature = "vst3")]
    use crate::radiant_gui::RadiantEditor;
    #[cfg(feature = "vst3")]
    use radiant::gui::types::Point;
    #[cfg(feature = "vst3")]
    use radiant::runtime::{Event, SurfacePaintPlan};
    #[cfg(feature = "vst3")]
    use radiant::theme::DpiScale;
    #[cfg(feature = "vst3")]
    use radiant::theme::ThemeTokens;
    #[cfg(feature = "vst3")]
    use radiant::widgets::{PointerButton, PointerModifiers, WidgetKey};
    #[cfg(feature = "vst3")]
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        WM_CHAR, WM_DPICHANGED, WM_DPICHANGED_AFTERPARENT, WM_GETDLGCODE, WM_KEYDOWN, WM_KEYUP,
        WM_SIZE, WM_SYSCHAR, WM_SYSKEYDOWN, WM_SYSKEYUP,
    };

    #[test]
    fn dpi_dispatch_uses_window_query_after_parent_change() {
        assert_eq!(
            dpi_change_kind(WM_DPICHANGED),
            Some(DpiChangeKind::MessageDpi)
        );
        assert_eq!(
            dpi_change_kind(WM_DPICHANGED_AFTERPARENT),
            Some(DpiChangeKind::WindowDpi)
        );
        assert_eq!(dpi_change_kind(WM_SIZE), None);
    }

    #[cfg(feature = "vst3")]
    struct KeyRecordingEditor {
        plan: SurfacePaintPlan,
        characters: Vec<char>,
        shortcuts: Vec<(char, PointerModifiers)>,
        keys: Vec<WidgetKey>,
        key_modifiers: Vec<KeyboardModifiers>,
        canceled: bool,
        events: Rc<RefCell<Vec<Event>>>,
    }

    #[cfg(feature = "vst3")]
    impl KeyRecordingEditor {
        fn new() -> Self {
            Self::with_events(Rc::new(RefCell::new(Vec::new())))
        }

        fn with_events(events: Rc<RefCell<Vec<Event>>>) -> Self {
            Self {
                plan: SurfacePaintPlan::empty(&ThemeTokens::default()),
                characters: Vec::new(),
                shortcuts: Vec::new(),
                keys: Vec::new(),
                key_modifiers: Vec::new(),
                canceled: false,
                events,
            }
        }
    }

    #[cfg(feature = "vst3")]
    impl RadiantEditor for KeyRecordingEditor {
        fn resize(&mut self, _width: u32, _height: u32) {}

        fn dispatch_event(&mut self, event: Event) {
            self.events.borrow_mut().push(event);
        }

        fn paint_plan(&mut self) -> &SurfacePaintPlan {
            &self.plan
        }

        fn needs_realtime_redraw(&self) -> bool {
            false
        }

        fn dispatch_key_press(&mut self, key: WidgetKey, modifiers: KeyboardModifiers) -> bool {
            self.keys.push(key);
            self.key_modifiers.push(modifiers);
            true
        }

        fn dispatch_character(&mut self, character: char) -> bool {
            self.characters.push(character);
            true
        }

        fn dispatch_shortcut(&mut self, character: char, modifiers: PointerModifiers) -> bool {
            self.shortcuts.push((character, modifiers));
            false
        }

        fn cancel_text_entry(&mut self) -> bool {
            self.canceled = true;
            true
        }
    }

    #[cfg(feature = "vst3")]
    fn test_window_state(events: Rc<RefCell<Vec<Event>>>) -> super::WindowState {
        super::WindowState::new(
            HWND(std::ptr::null_mut()),
            Box::new(KeyRecordingEditor::with_events(events)),
            Rc::new(RefCell::new(None)),
            Rc::new(Cell::new(None)),
            Rc::new(Cell::new(None)),
            Rc::new(Cell::new(DpiScale::ONE)),
            Rc::new(Cell::new(KeyboardDeliveryMode::Native)),
        )
    }

    #[cfg(feature = "vst3")]
    #[test]
    fn pointer_cancellation_is_idempotent_and_preserves_focus_without_clear() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let mut state = test_window_state(Rc::clone(&events));
        state.active_button = Some(PointerButton::Primary);

        state.cancel_native_interaction(false);
        state.cancel_native_interaction(false);

        assert_eq!(*events.borrow(), vec![Event::pointer_capture_cancelled()]);
        assert!(state.active_button.is_none());
    }

    #[cfg(feature = "vst3")]
    #[test]
    fn focus_loss_cancels_pointer_before_clearing_focus() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let mut state = test_window_state(Rc::clone(&events));
        state.active_button = Some(PointerButton::Primary);

        state.cancel_native_interaction(true);

        assert_eq!(
            *events.borrow(),
            vec![Event::pointer_capture_cancelled(), Event::clear_focus()]
        );
    }

    #[cfg(feature = "vst3")]
    #[test]
    fn teardown_cancels_before_retaining_the_editor() {
        let events = Rc::new(RefCell::new(Vec::new()));
        {
            let mut state = test_window_state(Rc::clone(&events));
            state.active_button = Some(PointerButton::Primary);
        }

        assert_eq!(
            *events.borrow(),
            vec![Event::pointer_capture_cancelled(), Event::clear_focus()]
        );
    }

    #[cfg(feature = "vst3")]
    #[test]
    fn unmatched_button_up_does_not_end_the_current_capture() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let mut state = test_window_state(Rc::clone(&events));
        let modifiers = PointerModifiers::default();

        state.pointer_press(10, 20, PointerButton::Primary, modifiers, false);
        state.pointer_press(30, 40, PointerButton::Secondary, modifiers, false);
        state.pointer_release(50, 60, PointerButton::Primary, modifiers);

        assert_eq!(state.active_button, Some(PointerButton::Secondary));
        assert_eq!(
            *events.borrow(),
            vec![
                Event::pointer_modifiers_changed(modifiers),
                Event::pointer_press(Point::new(10.0, 20.0), PointerButton::Primary, modifiers,),
                Event::pointer_capture_cancelled(),
                Event::pointer_modifiers_changed(modifiers),
                Event::pointer_press(Point::new(30.0, 40.0), PointerButton::Secondary, modifiers,),
            ]
        );

        state.pointer_release(50, 60, PointerButton::Secondary, modifiers);

        assert!(state.active_button.is_none());
        assert_eq!(
            *events.borrow(),
            vec![
                Event::pointer_modifiers_changed(modifiers),
                Event::pointer_press(Point::new(10.0, 20.0), PointerButton::Primary, modifiers,),
                Event::pointer_capture_cancelled(),
                Event::pointer_modifiers_changed(modifiers),
                Event::pointer_press(Point::new(30.0, 40.0), PointerButton::Secondary, modifiers,),
                Event::pointer_modifiers_changed(modifiers),
                Event::pointer_release(Point::new(50.0, 60.0), PointerButton::Secondary, modifiers,),
            ]
        );
    }

    #[cfg(feature = "vst3")]
    #[test]
    fn vst3_callback_translates_steinberg_virtual_keys() {
        use toybox_vst3_ffi::Steinberg::VirtualKeyCodes_::{
            KEY_BACK, KEY_DELETE, KEY_DOWN, KEY_END, KEY_ENTER, KEY_ESCAPE, KEY_HOME, KEY_LEFT,
            KEY_RETURN, KEY_RIGHT, KEY_SPACE, KEY_TAB, KEY_UP,
        };

        let mut editor = KeyRecordingEditor::new();
        let semantic_keys = [
            (KEY_BACK, WidgetKey::Backspace),
            (KEY_TAB, WidgetKey::Tab),
            (KEY_RETURN, WidgetKey::Enter),
            (KEY_ENTER, WidgetKey::Enter),
            (KEY_SPACE, WidgetKey::Space),
            (KEY_END, WidgetKey::End),
            (KEY_HOME, WidgetKey::Home),
            (KEY_LEFT, WidgetKey::ArrowLeft),
            (KEY_UP, WidgetKey::ArrowUp),
            (KEY_RIGHT, WidgetKey::ArrowRight),
            (KEY_DOWN, WidgetKey::ArrowDown),
            (KEY_DELETE, WidgetKey::Delete),
        ];

        for (key_code, expected) in semantic_keys {
            assert!(dispatch_vst3_key_down(
                Some(&mut editor),
                0,
                key_code as i16,
                0
            ));
            assert_eq!(editor.keys.last(), Some(&expected));
        }

        assert!(dispatch_vst3_key_down(
            Some(&mut editor),
            0,
            KEY_ESCAPE as i16,
            0
        ));
        assert!(editor.canceled);
        assert!(editor.characters.is_empty());
    }

    #[cfg(feature = "vst3")]
    #[test]
    fn vst3_key_down_preserves_shift_and_unshifted_keyboard_modifiers() {
        use toybox_vst3_ffi::Steinberg::KeyModifier_::kShiftKey;
        use toybox_vst3_ffi::Steinberg::VirtualKeyCodes_::KEY_UP;

        let events = Rc::new(RefCell::new(Vec::new()));
        let mut editor = KeyRecordingEditor::with_events(Rc::clone(&events));

        assert!(dispatch_vst3_key_down(
            Some(&mut editor),
            0,
            KEY_UP as i16,
            kShiftKey as i16,
        ));
        assert!(dispatch_vst3_key_down(
            Some(&mut editor),
            0,
            KEY_UP as i16,
            0
        ));

        assert_eq!(
            editor.key_modifiers,
            vec![
                KeyboardModifiers {
                    shift: true,
                    ..KeyboardModifiers::default()
                },
                KeyboardModifiers::default(),
            ]
        );
        assert_eq!(
            *events.borrow(),
            vec![
                Event::pointer_modifiers_changed(PointerModifiers {
                    shift: true,
                    ..PointerModifiers::default()
                }),
                Event::pointer_modifiers_changed(PointerModifiers::default()),
            ]
        );
    }

    #[cfg(feature = "vst3")]
    #[test]
    fn vst3_callback_falls_back_to_unicode_text_and_command_shortcuts() {
        use toybox_vst3_ffi::Steinberg::KeyModifier_::{kCommandKey, kShiftKey};
        use toybox_vst3_ffi::Steinberg::VirtualKeyCodes_::{KEY_ESCAPE, KEY_LEFT};

        let mut editor = KeyRecordingEditor::new();
        let modifiers = (kCommandKey | kShiftKey) as i16;

        assert!(dispatch_vst3_key_down(Some(&mut editor), 'ß' as u16, 0, 0));
        assert!(!dispatch_vst3_key_down(
            Some(&mut editor),
            'z' as u16,
            0,
            modifiers
        ));
        assert!(!dispatch_vst3_key_down(
            Some(&mut editor),
            0,
            KEY_LEFT as i16,
            modifiers
        ));
        assert!(!dispatch_vst3_key_down(
            Some(&mut editor),
            0,
            KEY_ESCAPE as i16,
            modifiers
        ));

        assert_eq!(editor.characters, vec!['ß']);
        assert!(editor.keys.is_empty());
        assert!(editor.key_modifiers.is_empty());
        assert_eq!(
            editor.shortcuts,
            vec![(
                'z',
                PointerModifiers {
                    command: true,
                    shift: true,
                    alt: false,
                }
            )]
        );
        assert!(!editor.canceled);
    }

    #[test]
    fn callback_only_mode_suppresses_every_native_keyboard_message() {
        for message in [
            WM_KEYDOWN,
            WM_SYSKEYDOWN,
            WM_KEYUP,
            WM_SYSKEYUP,
            WM_CHAR,
            WM_SYSCHAR,
            WM_UNICHAR,
        ] {
            assert!(suppresses_native_keyboard_message(
                KeyboardDeliveryMode::CallbackOnly,
                message
            ));
        }
    }

    #[test]
    fn native_mode_does_not_suppress_keyboard_messages() {
        for message in [WM_KEYDOWN, WM_KEYUP, WM_CHAR, WM_UNICHAR] {
            assert!(!suppresses_native_keyboard_message(
                KeyboardDeliveryMode::Native,
                message
            ));
        }
    }

    #[test]
    fn dialog_code_is_not_a_native_keyboard_message() {
        assert!(!suppresses_native_keyboard_message(
            KeyboardDeliveryMode::CallbackOnly,
            WM_GETDLGCODE
        ));
    }

    #[test]
    fn callback_only_mode_declines_native_dialog_keyboard_delivery() {
        assert_eq!(
            dialog_code_for_keyboard_mode(KeyboardDeliveryMode::CallbackOnly),
            0
        );
        assert_ne!(
            dialog_code_for_keyboard_mode(KeyboardDeliveryMode::Native),
            0
        );
    }

    #[test]
    fn utf16_surrogate_pairs_are_decoded_without_accepting_lone_units() {
        assert_eq!(utf16_surrogate_pair_to_char(0xd83d, 0xde00), Some('😀'));
        assert_eq!(utf16_surrogate_pair_to_char(0xd83d, b'a' as u16), None);
        assert_eq!(utf16_surrogate_pair_to_char(b'a' as u16, 0xde00), None);
    }

    #[test]
    fn preopen_physical_sizes_are_rescaled_when_the_parent_dpi_changes() {
        use super::rescale_physical_size;
        use radiant::theme::DpiScale;

        assert_eq!(
            rescale_physical_size((420, 282), DpiScale::ONE, DpiScale::new(1.5)),
            (630, 423)
        );
    }

    #[cfg(feature = "vst3")]
    #[test]
    fn vst3_modifier_bits_are_not_interpreted_as_win32_or_cocoa_masks() {
        assert_eq!(host_modifier_bits(1 | 2 | 4), 0b111);
        assert_eq!(host_modifier_bits(1), 0b010);
        assert_eq!(host_modifier_bits(2), 0b100);
        assert_eq!(host_modifier_bits(4), 0b001);
        assert_eq!(host_modifier_bits(8), 0b001);
    }

    #[test]
    fn stale_or_parent_destroyed_state_fails_the_incarnation_fence() {
        let current = std::ptr::dangling_mut::<super::WindowState>();
        let stale = current.wrapping_add(1);
        assert!(state_pointer_matches(true, Some(current), Some(current)));
        assert!(!state_pointer_matches(false, Some(current), Some(current)));
        assert!(!state_pointer_matches(true, Some(current), Some(stale)));
        assert!(!state_pointer_matches(true, Some(current), None));
    }
}
