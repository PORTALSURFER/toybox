//! Win32 child window and explicit WGPU surface bridge for embedded GPUI.

#![allow(
    clippy::missing_docs_in_private_items,
    unexpected_cfgs,
    unsafe_op_in_unsafe_fn
)]

use std::ffi::c_void;
use std::mem::size_of;
use std::num::NonZeroIsize;
use std::ptr::{self, NonNull};
use std::rc::{Rc, Weak};
use std::sync::Mutex;

use gpui::{
    DevicePixels, GpuSpecs, KeyDownEvent, KeyUpEvent, Modifiers, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, Pixels, PlatformAtlas, PlatformInput, Point, Scene, ScrollDelta,
    ScrollWheelEvent, Size, TouchPhase, px,
};
use gpui_wgpu::{WgpuRenderer, WgpuSurfaceConfig};
use raw_window_handle_06::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawWindowHandle,
    Win32WindowHandle, WindowHandle, WindowsDisplayHandle,
};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, GetSysColorBrush, PAINTSTRUCT};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CS_HREDRAW, CS_OWNDC, CS_VREDRAW, CreateWindowExW, DefWindowProcW, DestroyWindow,
    GWLP_USERDATA, GetWindowLongPtrW, IDC_ARROW, IsWindowVisible, LoadCursorW, RegisterClassExW,
    SW_HIDE, SW_SHOW, SetFocus, SetWindowLongPtrW, ShowWindow, UnregisterClassW, WM_CHAR,
    WM_DPICHANGED, WM_ERASEBKGND, WM_KEYDOWN, WM_KEYUP,
    WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_NCDESTROY, WM_PAINT,
    WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SIZE, WNDCLASSEXW, WNDPROC, WS_CHILD, WS_TABSTOP, WS_VISIBLE,
};
use windows::core::PCWSTR;

use super::super::WindowState;

pub(crate) fn parent_scale_factor(parent: raw_window_handle::RawWindowHandle) -> f32 {
    let raw = match parent {
        raw_window_handle::RawWindowHandle::Win32(handle) => handle.hwnd,
        _ => ptr::null_mut(),
    };
    let hwnd = HWND(raw as isize);
    if hwnd.is_invalid() {
        1.0
    } else {
        unsafe { GetDpiForWindow(hwnd).max(1) as f32 / 96.0 }
    }
}

const SHIFT_MASK: i16 = 0x8000;
const CONTROL_MASK: i16 = 0x8000;
const OPTION_MASK: i16 = 0x8000;
const COMMAND_MASK: i16 = 0x8000;

#[derive(Clone, Debug)]
struct ViewHandle(HWND);

unsafe impl Send for ViewHandle {}
unsafe impl Sync for ViewHandle {}

impl HasWindowHandle for ViewHandle {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let Some(hwnd) = NonZeroIsize::new(self.0.0) else {
            return Err(HandleError::NotSupported);
        };
        let handle = Win32WindowHandle::new(hwnd);
        Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::Win32(handle)) })
    }
}

impl HasDisplayHandle for ViewHandle {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Ok(unsafe {
            DisplayHandle::borrow_raw(RawWindowHandle::Windows(WindowsDisplayHandle::new().into()))
        })
    }
}

/// One independently owned Win32 GPUI child window.
pub(crate) struct NativeChild {
    hwnd: HWND,
    renderer: Option<WgpuRenderer>,
    handle: ViewHandle,
    owner_token: Option<Box<Weak<WindowState>>>,
    class_name: Vec<u16>,
    instance: HINSTANCE,
    scale_factor: f32,
    capture_requested: bool,
    capture_result: Option<Vec<u8>>,
    failed: bool,
}

unsafe impl Send for NativeChild {}
unsafe impl Sync for NativeChild {}

impl NativeChild {
    pub(crate) fn new(
        parent: raw_window_handle::RawWindowHandle,
        class_name: &'static str,
        owner: Weak<WindowState>,
        gpu_context: gpui_wgpu::GpuContext,
        size: Size<Pixels>,
        _callback_keyboard_only: bool,
        _visibility_callback: std::rc::Rc<std::cell::RefCell<Option<Box<dyn FnMut(bool)>>>>,
    ) -> anyhow::Result<Self> {
        let parent = match parent {
            raw_window_handle::RawWindowHandle::Win32(handle) => handle.hwnd,
            _ => ptr::null_mut(),
        };
        let parent = HWND(parent as isize);
        if parent.is_invalid() {
            anyhow::bail!("missing Win32 parent HWND");
        }
        let instance = unsafe { HINSTANCE(GetModuleHandleW(None)?.0) };
        let scale_factor = unsafe { GetDpiForWindow(parent).max(1) as f32 / 96.0 };
        let class_name = register_class(class_name, instance)?;
        let mut owner_token = Box::new(owner);
        let frame = [
            (f32::from(size.width) * scale_factor).max(1.0) as i32,
            (f32::from(size.height) * scale_factor).max(1.0) as i32,
        ];
        let hwnd = unsafe {
            CreateWindowExW(
                Default::default(),
                PCWSTR(class_name.as_ptr()),
                PCWSTR::null(),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP,
                0,
                0,
                frame[0],
                frame[1],
                Some(parent),
                None,
                Some(instance),
                Some((&mut *owner_token as *mut Weak<WindowState>).cast::<c_void>()),
            )?
        };
        let handle = ViewHandle(hwnd);
        let renderer = match unsafe {
            WgpuRenderer::new_with_surface_target(
                gpu_context,
                &handle,
                gpui_wgpu::wgpu::SurfaceTargetUnsafe::RawHandle {
                    raw_display_handle: None,
                    raw_window_handle: handle.window_handle()?.as_raw(),
                },
                WgpuSurfaceConfig {
                    size: gpui::size(DevicePixels(frame[0]), DevicePixels(frame[1])),
                    transparent: false,
                    preferred_present_mode: None,
                },
                None,
            )
        } {
            Ok(renderer) => renderer,
            Err(error) => {
                unsafe { DestroyWindow(hwnd) }?;
                unregister_class(&class_name, instance);
                return Err(anyhow::anyhow!(
                    "failed to initialize GPUI WGPU renderer: {error}"
                ));
            }
        };
        unsafe {
            let _: isize = SetWindowLongPtrW(
                hwnd,
                GWLP_USERDATA,
                (&mut *owner_token as *mut Weak<WindowState>) as isize,
            );
        }
        Ok(Self {
            hwnd,
            renderer: Some(renderer),
            handle,
            owner_token: Some(owner_token),
            class_name,
            instance,
            scale_factor,
            capture_requested: false,
            capture_result: None,
            failed: false,
        })
    }

    pub(crate) fn draw(&mut self, scene: &Scene) {
        if self.failed {
            return;
        }
        if let Some(renderer) = self.renderer.as_mut() {
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
    }

    pub(crate) fn request_capture(&mut self) {
        self.capture_requested = true;
        self.capture_result = None;
    }

    pub(crate) fn take_capture(&mut self) -> Option<(u32, u32, Vec<u8>)> {
        let pixels = self.capture_result.take()?;
        let size = self.renderer.as_ref()?.viewport_size();
        Some((size.width.0.max(1) as u32, size.height.0.max(1) as u32, pixels))
    }

    pub(crate) fn is_failed(&self) -> bool {
        self.failed
    }

    pub(crate) fn resize(&mut self, size: Size<Pixels>) {
        if self.failed {
            return;
        }
        let width = (f32::from(size.width) * self.scale_factor).max(1.0) as i32;
        let height = (f32::from(size.height) * self.scale_factor).max(1.0) as i32;
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowPos(
                self.hwnd,
                None,
                0,
                0,
                width,
                height,
                windows::Win32::UI::WindowsAndMessaging::SWP_NOMOVE
                    | windows::Win32::UI::WindowsAndMessaging::SWP_NOZORDER
                    | windows::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE,
            );
        }
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.update_drawable_size(gpui::size(DevicePixels(width), DevicePixels(height)));
        }
    }

    pub(crate) fn set_visible(&mut self, visible: bool) {
        if !self.failed {
            unsafe {
                ShowWindow(self.hwnd, if visible { SW_SHOW } else { SW_HIDE });
                if visible {
                    let _ = windows::Win32::UI::WindowsAndMessaging::InvalidateRect(
                        self.hwnd, None, false,
                    );
                }
            }
        }
    }

    pub(crate) fn set_focus(&mut self, focused: bool) -> bool {
        if self.failed {
            return false;
        }
        unsafe {
            if focused {
                SetFocus(Some(self.hwnd)).is_ok()
            } else {
                SetFocus(None).is_ok()
            }
        }
    }

    pub(crate) fn is_visible(&self) -> bool {
        !self.failed && unsafe { IsWindowVisible(self.hwnd).as_bool() }
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
        self.scale_factor
    }

    pub(crate) fn dpi_changed(&mut self, logical_size: Size<Pixels>) {
        self.scale_factor = unsafe { GetDpiForWindow(self.hwnd).max(1) as f32 / 96.0 };
        self.resize(logical_size);
    }

    pub(crate) fn window_handle(&self) -> Result<WindowHandle<'static>, HandleError> {
        self.handle.window_handle()
    }

    pub(crate) fn display_handle(&self) -> Result<DisplayHandle<'static>, HandleError> {
        Ok(unsafe {
            DisplayHandle::borrow_raw(RawWindowHandle::Windows(WindowsDisplayHandle::new().into()))
        })
    }

    pub(crate) fn quarantine(&mut self) {
        if self.failed {
            return;
        }
        self.failed = true;
        unsafe {
            let _: isize = SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, 0);
            ShowWindow(self.hwnd, SW_HIDE);
        }
        if let Some(mut renderer) = self.renderer.take() {
            renderer.destroy();
        }
    }

    pub(crate) fn clear_owner(&mut self) {
        unsafe {
            let _: isize = SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, 0);
        }
    }

    pub(crate) fn close(&mut self) {
        if let Some(mut renderer) = self.renderer.take() {
            renderer.destroy();
        }
        unsafe {
            let _: isize = SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, 0);
            let _ = DestroyWindow(self.hwnd);
        }
        self.owner_token.take();
        unregister_class(&self.class_name, self.instance);
    }
}

static CLASS_REGISTRATION: Mutex<()> = Mutex::new(());

fn register_class(class_name: &'static str, instance: HINSTANCE) -> anyhow::Result<Vec<u16>> {
    let name = format!("{class_name}_ToyboxGpui_{:x}", window_proc as usize);
    let mut wide: Vec<u16> = name.encode_utf16().collect();
    wide.push(0);
    let _lock = CLASS_REGISTRATION
        .lock()
        .map_err(|_| anyhow::anyhow!("Win32 class registry poisoned"))?;
    let cursor = unsafe { LoadCursorW(None, IDC_ARROW)? };
    unsafe {
        let class = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW | CS_OWNDC,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance,
            hIcon: None,
            hCursor: cursor,
            hbrBackground: GetSysColorBrush(0),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: PCWSTR(wide.as_ptr()),
            hIconSm: None,
        };
        if RegisterClassExW(&class) == 0 {
            let error = windows::core::Error::from_win32();
            if error.code().0 != 1410 {
                return Err(anyhow::anyhow!("RegisterClassExW failed: {error}"));
            }
        }
    }
    Ok(wide)
}

fn unregister_class(class_name: &[u16], instance: HINSTANCE) {
    unsafe {
        let _ = UnregisterClassW(PCWSTR(class_name.as_ptr()), Some(instance));
    }
}

fn owner(hwnd: HWND) -> Option<Rc<WindowState>> {
    unsafe {
        let token = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Weak<WindowState>;
        token.as_ref().and_then(Weak::upgrade)
    }
}

fn native_callback(hwnd: HWND, operation: &str, callback: impl FnOnce(&WindowState)) {
    let Some(owner) = owner(hwnd) else {
        return;
    };
    if crate::gui_panic::contain(operation, || callback(owner.as_ref())).is_none() {
        unsafe {
            let _: isize = SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            ShowWindow(hwnd, SW_HIDE);
        }
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_PAINT => {
            let mut paint = PAINTSTRUCT::default();
            BeginPaint(hwnd, &mut paint);
            native_callback(hwnd, "GPUI Win32 paint", |owner| owner.request_frame());
            EndPaint(hwnd, &paint);
            LRESULT(0)
        }
        WM_DPICHANGED => {
            native_callback(hwnd, "GPUI Win32 DPI change", |owner| {
                owner.native_dpi_changed();
            });
            LRESULT(0)
        }
        WM_SIZE => {
            let width = (lparam.0 as u32 & 0xffff) as f32;
            let height = ((lparam.0 as u32 >> 16) & 0xffff) as f32;
            native_callback(hwnd, "GPUI Win32 resize", |owner| {
                owner.native_resize_device(width, height)
            });
            LRESULT(0)
        }
        WM_LBUTTONDOWN | WM_RBUTTONDOWN => {
            native_callback(hwnd, "GPUI Win32 mouse down", |owner| {
                owner.native_mouse_down_focus();
                let button = if message == WM_LBUTTONDOWN {
                    MouseButton::Left
                } else {
                    MouseButton::Right
                };
                let _ = owner.dispatch_input(PlatformInput::MouseDown(MouseDownEvent {
                    button,
                    position: mouse_position(lparam),
                    modifiers: modifiers(),
                    click_count: 1,
                    first_mouse: true,
                }));
            });
            LRESULT(0)
        }
        WM_LBUTTONUP | WM_RBUTTONUP => {
            native_callback(hwnd, "GPUI Win32 mouse up", |owner| {
                let button = if message == WM_LBUTTONUP {
                    MouseButton::Left
                } else {
                    MouseButton::Right
                };
                let _ = owner.dispatch_input(PlatformInput::MouseUp(MouseUpEvent {
                    button,
                    position: mouse_position(lparam),
                    modifiers: modifiers(),
                    click_count: 1,
                }));
            });
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            native_callback(hwnd, "GPUI Win32 mouse move", |owner| {
                let _ = owner.dispatch_input(PlatformInput::MouseMove(MouseMoveEvent {
                    position: mouse_position(lparam),
                    pressed_button: None,
                    modifiers: modifiers(),
                }));
            });
            LRESULT(0)
        }
        WM_MOUSEWHEEL => {
            native_callback(hwnd, "GPUI Win32 wheel", |owner| {
                let delta = ((wparam.0 >> 16) & 0xffff) as i16 as f32 / 120.0;
                let _ = owner.dispatch_input(PlatformInput::ScrollWheel(ScrollWheelEvent {
                    position: mouse_position(lparam),
                    delta: ScrollDelta::Lines(Point::new(0.0, -delta)),
                    modifiers: modifiers(),
                    touch_phase: TouchPhase::Moved,
                }));
            });
            LRESULT(0)
        }
        WM_KEYDOWN => {
            native_callback(hwnd, "GPUI Win32 key down", |owner| {
                let key = key_name(wparam.0 as u32);
                let _ = owner.dispatch_native_key(PlatformInput::KeyDown(KeyDownEvent {
                    keystroke: gpui::Keystroke {
                        modifiers: modifiers(),
                        key,
                        key_char: None,
                    },
                    is_held: (lparam.0 & (1 << 30)) != 0,
                    prefer_character_input: false,
                }));
            });
            LRESULT(0)
        }
        WM_KEYUP => {
            native_callback(hwnd, "GPUI Win32 key up", |owner| {
                let _ = owner.dispatch_native_key(PlatformInput::KeyUp(KeyUpEvent {
                    keystroke: gpui::Keystroke {
                        modifiers: modifiers(),
                        key: key_name(wparam.0 as u32),
                        key_char: None,
                    },
                }));
            });
            LRESULT(0)
        }
        WM_CHAR => {
            native_callback(hwnd, "GPUI Win32 text input", |owner| {
                if let Some(character) = char::from_u32(wparam.0 as u32) {
                    let mut text = String::new();
                    text.push(character);
                    owner.dispatch_text(&text);
                }
            });
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_NCDESTROY => {
            let _: isize = SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            DefWindowProcW(hwnd, message, wparam, lparam)
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

fn modifiers() -> Modifiers {
    let down = |key| unsafe { GetKeyState(key.0 as i32) < 0 };
    Modifiers {
        control: down(VK_CONTROL),
        alt: down(VK_MENU),
        shift: down(VK_SHIFT),
        platform: down(VK_LWIN) || down(VK_RWIN),
        function: false,
    }
}

fn mouse_position(lparam: LPARAM) -> Point<Pixels> {
    let x = (lparam.0 as u32 & 0xffff) as i16 as f32;
    let y = ((lparam.0 as u32 >> 16) & 0xffff) as i16 as f32;
    Point::new(px(x), px(y))
}

fn key_name(key: u32) -> String {
    match key {
        0x08 => "backspace",
        0x09 => "tab",
        0x0d => "enter",
        0x1b => "escape",
        0x20 => "space",
        0x21 => "pageup",
        0x22 => "pagedown",
        0x23 => "end",
        0x24 => "home",
        0x25 => "left",
        0x26 => "up",
        0x27 => "right",
        0x28 => "down",
        0x2e => "delete",
        0x70..=0x7b => return format!("f{}", key - 0x6f),
        value if (0x30..=0x39).contains(&value) || (0x41..=0x5a).contains(&value) => {
            return char::from_u32(value)
                .unwrap_or('?')
                .to_ascii_lowercase()
                .to_string();
        }
        _ => "unknown",
    }
    .to_string()
}
