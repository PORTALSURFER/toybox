//! Win32 child window and explicit WGPU surface bridge for embedded GPUI.

#![allow(
    clippy::missing_docs_in_private_items,
    unexpected_cfgs,
    unsafe_op_in_unsafe_fn
)]

use std::ffi::c_void;
use std::mem::size_of;
use std::num::NonZeroIsize;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicU64, Ordering};

use gpui::{
    ClipboardItem, DevicePixels, GpuSpecs, KeyDownEvent, KeyUpEvent, Modifiers, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, PlatformAtlas, PlatformInput, Point,
    Scene, ScrollDelta, ScrollWheelEvent, Size, TouchPhase, px,
};
use gpui_wgpu::{WgpuRenderer, WgpuSurfaceConfig};
use raw_window_handle_06::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, Win32WindowHandle, WindowHandle, WindowsDisplayHandle,
};
use windows::Win32::Foundation::{
    GlobalFree, HANDLE, HGLOBAL, HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM,
};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, COLOR_WINDOW, EndPaint, GetSysColorBrush, InvalidateRect, PAINTSTRUCT,
    ScreenToClient,
};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Memory::{
    GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock,
};
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::Input::Ime::{
    GCS_COMPSTR, GCS_RESULTSTR, IME_COMPOSITION_STRING, ImmGetCompositionStringW, ImmGetContext,
    ImmReleaseContext,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, SetFocus, VIRTUAL_KEY, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
};
use windows::Win32::UI::WindowsAndMessaging::KillTimer;
use windows::Win32::UI::WindowsAndMessaging::{
    CS_HREDRAW, CS_OWNDC, CS_VREDRAW, CreateWindowExW, DefWindowProcW, DestroyWindow, GA_ROOT,
    GWLP_USERDATA, GetAncestor, GetMessageTime, GetWindowLongPtrW, IDC_ARROW, IsIconic,
    IsWindowVisible, LoadCursorW, RegisterClassExW, SW_HIDE, SW_SHOW, SetTimer, SetWindowLongPtrW,
    ShowWindow, UnregisterClassW, WM_CHAR, WM_DPICHANGED, WM_DPICHANGED_AFTERPARENT, WM_ERASEBKGND,
    WM_IME_COMPOSITION, WM_IME_ENDCOMPOSITION, WM_IME_STARTCOMPOSITION, WM_KEYDOWN, WM_KEYUP,
    WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_NCDESTROY, WM_PAINT,
    WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SIZE, WM_TIMER, WNDCLASSEXW, WS_CHILD, WS_TABSTOP, WS_VISIBLE,
};
use windows::core::PCWSTR;

use super::super::WindowState;

const UI_TIMER_ID: usize = 1;
const UI_TIMER_INTERVAL_MS: u32 = 16;
const CF_UNICODETEXT_FORMAT: u32 = 13;

pub(crate) fn parent_scale_factor(parent: raw_window_handle::RawWindowHandle) -> f32 {
    let hwnd = match parent {
        raw_window_handle::RawWindowHandle::Win32(handle) => HWND(handle.hwnd),
        _ => HWND::default(),
    };
    if hwnd.is_invalid() {
        1.0
    } else {
        unsafe { GetDpiForWindow(hwnd).max(1) as f32 / 96.0 }
    }
}

#[derive(Clone, Debug)]
struct ViewHandle(HWND);

unsafe impl Send for ViewHandle {}
unsafe impl Sync for ViewHandle {}

impl HasWindowHandle for ViewHandle {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let Some(hwnd) = NonZeroIsize::new(self.0.0 as isize) else {
            return Err(HandleError::NotSupported);
        };
        let handle = Win32WindowHandle::new(hwnd);
        Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::Win32(handle)) })
    }
}

impl HasDisplayHandle for ViewHandle {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Ok(unsafe {
            DisplayHandle::borrow_raw(RawDisplayHandle::Windows(WindowsDisplayHandle::new()))
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
    timer_id: usize,
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
        let parent = match parent {
            raw_window_handle::RawWindowHandle::Win32(handle) => HWND(handle.hwnd),
            _ => HWND::default(),
        };
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
        let hwnd = match unsafe {
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
            )
        } {
            Ok(hwnd) => hwnd,
            Err(error) => {
                unregister_class(&class_name, instance);
                return Err(error.into());
            }
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
        let timer_id = unsafe { SetTimer(Some(hwnd), UI_TIMER_ID, UI_TIMER_INTERVAL_MS, None) };
        if timer_id == 0 {
            let mut renderer = renderer;
            renderer.destroy();
            unsafe { DestroyWindow(hwnd) }?;
            unregister_class(&class_name, instance);
            return Err(anyhow::anyhow!("SetTimer failed for GPUI child window"));
        }
        Ok(Self {
            hwnd,
            renderer: Some(renderer),
            handle,
            owner_token: Some(owner_token),
            class_name,
            instance,
            scale_factor,
            timer_id,
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
                    if self.timer_id == 0 {
                        self.timer_id =
                            SetTimer(Some(self.hwnd), UI_TIMER_ID, UI_TIMER_INTERVAL_MS, None);
                    }
                    let _ = InvalidateRect(Some(self.hwnd), None, false);
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
            if focused {
                SetFocus(Some(self.hwnd)).is_ok()
            } else {
                SetFocus(None).is_ok()
            }
        }
    }

    pub(crate) fn is_visible(&self) -> bool {
        if self.failed || !unsafe { IsWindowVisible(self.hwnd).as_bool() } {
            return false;
        }
        let root = unsafe { GetAncestor(self.hwnd, GA_ROOT) };
        root.is_invalid() || !unsafe { IsIconic(root).as_bool() }
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

    pub(crate) fn raw_handle(&self) -> *mut c_void {
        self.hwnd.0
    }

    pub(crate) fn dpi_changed(&mut self, logical_size: Size<Pixels>) {
        self.scale_factor = unsafe { GetDpiForWindow(self.hwnd).max(1) as f32 / 96.0 };
        self.resize(logical_size);
    }

    pub(crate) fn window_handle(&self) -> Result<WindowHandle<'static>, HandleError> {
        let Some(hwnd) = NonZeroIsize::new(self.hwnd.0 as isize) else {
            return Err(HandleError::NotSupported);
        };
        let handle = Win32WindowHandle::new(hwnd);
        Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::Win32(handle)) })
    }

    pub(crate) fn display_handle(&self) -> Result<DisplayHandle<'static>, HandleError> {
        Ok(unsafe {
            DisplayHandle::borrow_raw(RawDisplayHandle::Windows(WindowsDisplayHandle::new()))
        })
    }

    pub(crate) fn quarantine(&mut self) {
        if self.failed {
            return;
        }
        self.failed = true;
        self.stop_timer();
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
        self.stop_timer();
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

    fn stop_timer(&mut self) {
        if self.timer_id != 0 {
            unsafe {
                let _ = KillTimer(Some(self.hwnd), self.timer_id);
            }
            self.timer_id = 0;
        }
    }
}

static NEXT_CLASS_ID: AtomicU64 = AtomicU64::new(1);

fn register_class(class_name: &'static str, instance: HINSTANCE) -> anyhow::Result<Vec<u16>> {
    // Register every child independently. Reusing a class name lets the first
    // editor to close unregister the class while a sibling editor still owns
    // windows of that class, which can silently break the remaining editor.
    let serial = NEXT_CLASS_ID.fetch_add(1, Ordering::Relaxed);
    let name = format!(
        "{class_name}_ToyboxGpui_{:x}_{:x}_{serial:x}",
        window_proc as usize, instance.0 as usize,
    );
    let mut wide: Vec<u16> = name.encode_utf16().collect();
    wide.push(0);
    let cursor = unsafe { LoadCursorW(None, IDC_ARROW)? };
    unsafe {
        let class = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW | CS_OWNDC,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance,
            hIcon: Default::default(),
            hCursor: cursor,
            hbrBackground: GetSysColorBrush(COLOR_WINDOW),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: PCWSTR(wide.as_ptr()),
            hIconSm: Default::default(),
        };
        if RegisterClassExW(&class) == 0 {
            let error = windows::core::Error::from_thread();
            return Err(anyhow::anyhow!("RegisterClassExW failed: {error}"));
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
            let _ = KillTimer(Some(hwnd), UI_TIMER_ID);
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
        WM_TIMER if wparam.0 == UI_TIMER_ID => {
            native_callback(hwnd, "GPUI Win32 timer", WindowState::native_tick);
            LRESULT(0)
        }
        WM_DPICHANGED | WM_DPICHANGED_AFTERPARENT => {
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
                    position: client_mouse_position(lparam, owner.scale_factor()),
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
                    position: client_mouse_position(lparam, owner.scale_factor()),
                    modifiers: modifiers(),
                    click_count: 1,
                }));
            });
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            native_callback(hwnd, "GPUI Win32 mouse move", |owner| {
                let _ = owner.dispatch_input(PlatformInput::MouseMove(MouseMoveEvent {
                    position: client_mouse_position(lparam, owner.scale_factor()),
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
                    position: wheel_mouse_position(hwnd, lparam, owner.scale_factor()),
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
                let token =
                    owner.native_event_token(key_event_identity(hwnd, wparam, lparam, true));
                let _ = owner.dispatch_native_key(
                    PlatformInput::KeyDown(KeyDownEvent {
                        keystroke: gpui::Keystroke {
                            modifiers: modifiers(),
                            key,
                            key_char: None,
                        },
                        is_held: (lparam.0 & (1 << 30)) != 0,
                        prefer_character_input: false,
                    }),
                    Some(token),
                );
            });
            LRESULT(0)
        }
        WM_KEYUP => {
            native_callback(hwnd, "GPUI Win32 key up", |owner| {
                let token =
                    owner.native_event_token(key_event_identity(hwnd, wparam, lparam, false));
                let _ = owner.dispatch_native_key(
                    PlatformInput::KeyUp(KeyUpEvent {
                        keystroke: gpui::Keystroke {
                            modifiers: modifiers(),
                            key: key_name(wparam.0 as u32),
                            key_char: None,
                        },
                    }),
                    Some(token),
                );
            });
            LRESULT(0)
        }
        WM_IME_STARTCOMPOSITION => LRESULT(0),
        WM_IME_COMPOSITION => {
            let message_time = unsafe { GetMessageTime() as u32 };
            let flags = lparam.0 as u32;
            native_callback(hwnd, "GPUI Win32 IME composition", |owner| {
                let token = owner.windows_text_token(hwnd.0 as usize, message_time);
                if flags & GCS_RESULTSTR.0 != 0 {
                    if let Some(text) = ime_string(hwnd, GCS_RESULTSTR) {
                        owner.windows_ime_composition(&text, true, token);
                    }
                }
                if flags & GCS_COMPSTR.0 != 0 {
                    if let Some(text) = ime_string(hwnd, GCS_COMPSTR) {
                        owner.windows_ime_composition(&text, false, token);
                    }
                }
            });
            LRESULT(0)
        }
        WM_IME_ENDCOMPOSITION => {
            native_callback(hwnd, "GPUI Win32 IME end", WindowState::windows_ime_end);
            LRESULT(0)
        }
        WM_CHAR => {
            let message_time = unsafe { GetMessageTime() as u32 };
            native_callback(hwnd, "GPUI Win32 text input", |owner| {
                let token = owner.windows_text_token(hwnd.0 as usize, message_time);
                owner.windows_char(wparam.0 as u16, modifiers(), token);
            });
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_NCDESTROY => {
            let _ = KillTimer(Some(hwnd), UI_TIMER_ID);
            let _: isize = SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            DefWindowProcW(hwnd, message, wparam, lparam)
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

fn modifiers() -> Modifiers {
    let down = |key: VIRTUAL_KEY| unsafe { GetKeyState(key.0 as i32) < 0 };
    Modifiers {
        control: down(VK_CONTROL),
        alt: down(VK_MENU),
        shift: down(VK_SHIFT),
        platform: down(VK_LWIN) || down(VK_RWIN),
        function: false,
    }
}

fn client_mouse_position(lparam: LPARAM, scale_factor: f32) -> Point<Pixels> {
    let (x, y) = lparam_position(lparam);
    scaled_position(x, y, scale_factor)
}

fn wheel_mouse_position(hwnd: HWND, lparam: LPARAM, scale_factor: f32) -> Point<Pixels> {
    let (x, y) = lparam_position(lparam);
    let mut point = POINT { x, y };
    if unsafe { ScreenToClient(hwnd, &mut point).as_bool() } {
        scaled_position(point.x, point.y, scale_factor)
    } else {
        scaled_position(x, y, scale_factor)
    }
}

fn lparam_position(lparam: LPARAM) -> (i32, i32) {
    (
        (lparam.0 as u32 & 0xffff) as i16 as i32,
        ((lparam.0 as u32 >> 16) & 0xffff) as i16 as i32,
    )
}

fn scaled_position(x: i32, y: i32, scale_factor: f32) -> Point<Pixels> {
    let scale_factor = scale_factor.max(0.01);
    Point::new(px(x as f32 / scale_factor), px(y as f32 / scale_factor))
}

fn key_event_identity(
    hwnd: HWND,
    wparam: WPARAM,
    lparam: LPARAM,
    down: bool,
) -> super::super::NativeEventIdentity {
    let raw = lparam.0 as u64;
    super::super::NativeEventIdentity::Windows {
        message_time: unsafe { GetMessageTime() as u32 },
        window: hwnd.0 as usize as u64,
        virtual_key: wparam.0 as u32,
        scan_code: (((raw >> 16) & 0xff) as u16) | if raw & (1 << 24) != 0 { 0x100 } else { 0 },
        kind: if down { 1 } else { 2 },
    }
}

fn ime_string(hwnd: HWND, kind: IME_COMPOSITION_STRING) -> Option<String> {
    unsafe {
        let context = ImmGetContext(hwnd);
        if context.0.is_null() {
            return None;
        }
        let byte_count = ImmGetCompositionStringW(context, kind, None, 0);
        if byte_count < 0 {
            let _ = ImmReleaseContext(hwnd, context);
            return None;
        }
        let mut units = vec![0_u16; (byte_count as usize).div_ceil(2)];
        let read = if byte_count == 0 {
            0
        } else {
            ImmGetCompositionStringW(
                context,
                kind,
                Some(units.as_mut_ptr().cast::<c_void>()),
                byte_count as u32,
            )
        };
        let _ = ImmReleaseContext(hwnd, context);
        if read < 0 {
            return None;
        }
        Some(String::from_utf16_lossy(&units[..(read as usize / 2)]))
    }
}

pub(crate) fn read_clipboard() -> Option<ClipboardItem> {
    unsafe {
        OpenClipboard(None).ok()?;
        let result = (|| {
            let handle = GetClipboardData(CF_UNICODETEXT_FORMAT).ok()?;
            let global = HGLOBAL(handle.0);
            let pointer = GlobalLock(global) as *const u16;
            if pointer.is_null() {
                return None;
            }
            let units = GlobalSize(global) / size_of::<u16>();
            let slice = std::slice::from_raw_parts(pointer, units);
            let length = slice.iter().position(|unit| *unit == 0).unwrap_or(units);
            let text = String::from_utf16_lossy(&slice[..length]);
            let _ = GlobalUnlock(global);
            Some(ClipboardItem::new_string(text))
        })();
        let _ = CloseClipboard();
        result
    }
}

pub(crate) fn write_clipboard(item: ClipboardItem) {
    let Some(text) = item.text() else {
        return;
    };
    unsafe {
        if OpenClipboard(None).is_err() || EmptyClipboard().is_err() {
            let _ = CloseClipboard();
            return;
        }
        let mut units: Vec<u16> = text.encode_utf16().collect();
        units.push(0);
        let bytes = units.len().saturating_mul(size_of::<u16>());
        let Ok(global) = GlobalAlloc(GMEM_MOVEABLE, bytes) else {
            let _ = CloseClipboard();
            return;
        };
        let pointer = GlobalLock(global) as *mut u16;
        if pointer.is_null() {
            let _ = GlobalFree(Some(global));
            let _ = CloseClipboard();
            return;
        }
        std::ptr::copy_nonoverlapping(units.as_ptr(), pointer, units.len());
        let _ = GlobalUnlock(global);
        if SetClipboardData(CF_UNICODETEXT_FORMAT, Some(HANDLE(global.0))).is_err() {
            let _ = GlobalFree(Some(global));
        }
        let _ = CloseClipboard();
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_coordinates_convert_physical_pixels_to_logical_points() {
        let lparam = LPARAM((75_u32 << 16 | 150) as isize);
        assert_eq!(
            client_mouse_position(lparam, 1.5),
            Point::new(px(100.0), px(50.0))
        );
    }

    #[test]
    fn signed_client_coordinates_are_preserved_before_dpi_conversion() {
        let lparam = LPARAM(((-12_i16 as u16 as u32) << 16 | (-9_i16 as u16 as u32)) as isize);
        assert_eq!(
            client_mouse_position(lparam, 1.5),
            Point::new(px(-6.0), px(-8.0))
        );
    }
}
