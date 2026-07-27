//! Minimal host-owned Win32 child view for Radiant's embedded Vello renderer.
//!
//! The window procedure deliberately mirrors the Patchbay host lifecycle but
//! owns a Radiant editor and paint plan directly. The host creates the child on
//! the caller's native GUI thread; a future factory-based entry point can move
//! this state creation into a dedicated thread without changing the facade.

#![allow(unsafe_op_in_unsafe_fn)]

use super::{HostedGui, RadiantEditor};
use radiant::gui::types::{Point, Vector2};
use radiant::runtime::{
    EmbeddedVelloRenderer, EmbeddedVelloSurfaceHandle, Event, SurfacePaintPlan,
};
use radiant::theme::DpiScale;
use radiant::widgets::{PointerButton, PointerModifiers};
use raw_window_handle::RawWindowHandle;
use raw_window_handle_06::{
    RawDisplayHandle, RawWindowHandle as RawWindowHandle06, Win32WindowHandle, WindowsDisplayHandle,
};
use std::cell::Cell;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::num::NonZeroIsize;
use std::os::windows::ffi::OsStrExt;
use std::ptr::NonNull;
use std::sync::{Mutex, OnceLock};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, PAINTSTRUCT};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CS_HREDRAW, CS_VREDRAW, CreateWindowExW, DefWindowProcW, DestroyWindow, GWLP_USERDATA,
    GetWindowLongPtrW, InvalidateRect, RegisterClassW, SW_HIDE, SW_SHOW, ScreenToClient, SetTimer,
    SetWindowLongPtrW, ShowWindow, WM_DPICHANGED, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE,
    WM_MOUSEWHEEL, WM_NCCREATE, WM_NCDESTROY, WM_PAINT, WM_SIZE, WM_TIMER, WNDCLASSW, WS_CHILD,
    WS_CLIPCHILDREN, WS_CLIPSIBLINGS,
};
use windows::core::PCWSTR;

struct Win32EditorState {
    editor: Box<dyn RadiantEditor>,
    renderer: Option<EmbeddedVelloRenderer>,
    size: (u32, u32),
    scale: DpiScale,
}

/// Native Win32 Radiant child host.
pub struct RadiantVst3HostedGui {
    parent: Option<HWND>,
    root: Cell<Option<HWND>>,
    state: Cell<Option<NonNull<Win32EditorState>>>,
    editor: Option<Box<dyn RadiantEditor>>,
    editor_factory: Option<Box<dyn FnOnce() -> Box<dyn RadiantEditor>>>,
    default_size: Cell<(u32, u32)>,
    class_name: &'static str,
    scale: Cell<f64>,
}

impl RadiantVst3HostedGui {
    /// Create a host-owned Win32 view.
    pub fn new(
        class_name: &'static str,
        editor: impl RadiantEditor,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            parent: None,
            root: Cell::new(None),
            state: Cell::new(None),
            editor: Some(Box::new(editor)),
            editor_factory: None,
            default_size: Cell::new((width.max(1), height.max(1))),
            class_name,
            scale: Cell::new(1.0),
        }
    }

    /// Construct a host whose editor is created when the native child opens.
    ///
    /// Hosts that require thread-affine editor construction should use this
    /// entry point from their native GUI thread callback.
    pub fn new_with_factory(
        class_name: &'static str,
        factory: impl FnOnce() -> Box<dyn RadiantEditor> + 'static,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            parent: None,
            root: Cell::new(None),
            state: Cell::new(None),
            editor: None,
            editor_factory: Some(Box::new(factory)),
            default_size: Cell::new((width.max(1), height.max(1))),
            class_name,
            scale: Cell::new(1.0),
        }
    }

    /// Show the child window without rebuilding the editor.
    pub fn show(&self) {
        if let Some(hwnd) = self.root.get() {
            unsafe {
                ShowWindow(hwnd, SW_SHOW);
            }
        }
    }

    /// Hide the child window without destroying the editor.
    pub fn hide(&self) {
        if let Some(hwnd) = self.root.get() {
            unsafe {
                ShowWindow(hwnd, SW_HIDE);
            }
        }
    }

    /// Apply a host DPI change; the renderer reads the current scale on resize.
    pub fn set_scale(&self, scale: f64) {
        let scale = if scale.is_finite() && scale > 0.0 {
            scale
        } else {
            1.0
        };
        self.scale.set(scale);
        let (width, height) = self.default_size.get();
        self.request_resize(width, height);
    }

    fn state_ptr(&self) -> Option<NonNull<Win32EditorState>> {
        self.state.get().filter(|ptr| ptr.as_ptr() as usize > 1)
    }

    fn open_view(&mut self) -> bool {
        if self.root.get().is_some() {
            return true;
        }
        let Some(parent) = self.parent else {
            return false;
        };
        if !register_class(self.class_name) {
            return false;
        }
        let editor = self
            .editor
            .take()
            .or_else(|| self.editor_factory.take().map(|factory| factory()))
        else {
            return false;
        };
        let mut state = Box::new(Win32EditorState {
            editor,
            renderer: None,
            size: self.default_size.get(),
            scale: DpiScale::new(self.scale.get()),
        });
        let state_ptr = NonNull::from(state.as_mut());
        let title = wide(self.class_name);
        let instance = unsafe { GetModuleHandleW(None).unwrap_or_default() };
        let hwnd = unsafe {
            CreateWindowExW(
                Default::default(),
                PCWSTR(title.as_ptr()),
                PCWSTR(title.as_ptr()),
                WS_CHILD | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
                0,
                0,
                self.default_size.get().0 as i32,
                self.default_size.get().1 as i32,
                Some(parent),
                None,
                instance,
                Some(state_ptr.as_ptr().cast()),
            )
        };
        let Ok(hwnd) = hwnd else {
            self.editor = Some(state.editor);
            drop(state);
            return false;
        };
        let raw_window = win32_surface_handle(hwnd, instance);
        let renderer = unsafe {
            EmbeddedVelloRenderer::new(
                raw_window,
                Vector2::new(
                    self.default_size.get().0 as f32,
                    self.default_size.get().1 as f32,
                ),
                DpiScale::new(self.scale.get()),
            )
        };
        let Ok(renderer) = renderer else {
            unsafe {
                DestroyWindow(hwnd).ok();
            }
            self.editor = Some(state.editor);
            drop(state);
            return false;
        };
        state.renderer = Some(renderer);
        unsafe {
            (*state_ptr.as_ptr())
                .editor
                .resize(self.default_size.get().0, self.default_size.get().1);
        }
        let _ = Box::into_raw(state);
        self.root.set(Some(hwnd));
        self.state.set(Some(state_ptr));
        unsafe {
            SetTimer(hwnd, 1, 16, None);
            ShowWindow(hwnd, SW_SHOW);
        }
        true
    }

    fn close_view(&mut self) {
        if let Some(hwnd) = self.root.get() {
            let destroyed = unsafe { DestroyWindow(hwnd).is_ok() };
            if !destroyed {
                return;
            }
            self.root.set(None);
        }
        if let Some(state) = self.state.take().filter(|ptr| ptr.as_ptr() as usize > 1) {
            unsafe {
                let state = Box::from_raw(state.as_ptr());
                self.editor = Some(state.editor);
            }
        }
    }
}

impl HostedGui for RadiantVst3HostedGui {
    fn set_parent_raw(&mut self, parent: RawWindowHandle) {
        if let RawWindowHandle::Win32(handle) = parent {
            self.parent = Some(HWND(handle.hwnd));
        }
    }

    fn open(&mut self) -> bool {
        self.open_view()
    }
    fn close(&mut self) {
        self.close_view();
    }
    fn last_size(&self) -> Option<(u32, u32)> {
        self.state_ptr()
            .map(|state| unsafe { (*state.as_ptr()).size })
            .or(Some(self.default_size.get()))
    }

    fn request_resize(&self, width: u32, height: u32) {
        let width = width.max(1);
        let height = height.max(1);
        self.default_size.set((width, height));
        if let Some(state) = self.state_ptr() {
            unsafe {
                (*state.as_ptr()).size = (width, height);
                (*state.as_ptr()).editor.resize(width, height);
                if let Some(renderer) = (*state.as_ptr()).renderer.as_mut() {
                    (*state.as_ptr()).scale = DpiScale::new(self.scale.get());
                    renderer.resize(
                        Vector2::new(width as f32, height as f32),
                        (*state.as_ptr()).scale,
                    );
                }
            }
        }
        if let Some(hwnd) = self.root.get() {
            unsafe {
                windows::Win32::UI::WindowsAndMessaging::SetWindowPos(
                    hwnd,
                    None,
                    0,
                    0,
                    width as i32,
                    height as i32,
                    windows::Win32::UI::WindowsAndMessaging::SWP_NOMOVE
                        | windows::Win32::UI::WindowsAndMessaging::SWP_NOZORDER,
                );
            }
        }
    }

    fn on_key_down(&self, _key: u16, _key_code: i16, _modifiers: i16) -> bool {
        false
    }
}

fn register_class(class_name: &'static str) -> bool {
    static REGISTERED: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();
    let registered = REGISTERED.get_or_init(|| Mutex::new(HashSet::new()));
    let mut registered = registered
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if registered.contains(class_name) {
        return true;
    }
    let atom = {
        let name = wide(class_name);
        let instance = unsafe { GetModuleHandleW(None).unwrap_or_default() };
        let class = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            hInstance: HINSTANCE(instance.0),
            lpszClassName: PCWSTR(name.as_ptr()),
            ..Default::default()
        };
        unsafe { RegisterClassW(&class) }
    };
    if atom.0 != 0 {
        registered.insert(class_name);
        true
    } else {
        false
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        let create = &*(lparam.0 as *const windows::Win32::UI::WindowsAndMessaging::CREATESTRUCTW);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize);
    }
    if message == WM_NCDESTROY {
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
        return DefWindowProcW(hwnd, message, wparam, lparam);
    }
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Win32EditorState;
    if ptr.is_null() {
        return DefWindowProcW(hwnd, message, wparam, lparam);
    }
    let state = &mut *ptr;
    match message {
        WM_PAINT => {
            let mut paint = PAINTSTRUCT::default();
            BeginPaint(hwnd, &mut paint);
            if let Some(renderer) = state.renderer.as_mut() {
                let plan: &SurfacePaintPlan = state.editor.paint_plan();
                let _ = renderer.render(plan);
            }
            EndPaint(hwnd, &paint);
            LRESULT(0)
        }
        WM_TIMER => {
            InvalidateRect(hwnd, None, false);
            LRESULT(0)
        }
        WM_SIZE => {
            let width = (lparam.0 as u32 & 0xffff).max(1);
            let height = ((lparam.0 as u32 >> 16) & 0xffff).max(1);
            state.size = (width, height);
            state.editor.resize(width, height);
            if let Some(renderer) = state.renderer.as_mut() {
                renderer.resize(Vector2::new(width as f32, height as f32), state.scale);
            }
            InvalidateRect(hwnd, None, false);
            LRESULT(0)
        }
        WM_DPICHANGED => {
            let scale = dpi_scale_from_wparam(wparam.0);
            state.scale = DpiScale::new(scale);
            if let Some(renderer) = state.renderer.as_mut() {
                renderer.resize(
                    Vector2::new(state.size.0 as f32, state.size.1 as f32),
                    state.scale,
                );
            }
            InvalidateRect(hwnd, None, false);
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            state.editor.dispatch_event(Event::pointer_move(Point::new(
                (lparam.0 as i16) as f32,
                ((lparam.0 >> 16) as i16) as f32,
            )));
            InvalidateRect(hwnd, None, false);
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            state.editor.dispatch_event(Event::pointer_press(
                Point::new((lparam.0 as i16) as f32, ((lparam.0 >> 16) as i16) as f32),
                PointerButton::Primary,
                PointerModifiers::default(),
            ));
            InvalidateRect(hwnd, None, false);
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            state.editor.dispatch_event(Event::pointer_release(
                Point::new((lparam.0 as i16) as f32, ((lparam.0 >> 16) as i16) as f32),
                PointerButton::Primary,
                PointerModifiers::default(),
            ));
            InvalidateRect(hwnd, None, false);
            LRESULT(0)
        }
        WM_MOUSEWHEEL => {
            let delta = ((wparam.0 >> 16) as i16) as f32 / 120.0;
            let mut point = POINT {
                x: lparam.0 as i16 as i32,
                y: (lparam.0 >> 16) as i16 as i32,
            };
            ScreenToClient(hwnd, &mut point).ok();
            state.editor.dispatch_event(Event::scroll(
                Point::new(point.x as f32, point.y as f32),
                Vector2::new(0.0, -delta),
            ));
            InvalidateRect(hwnd, None, false);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

fn dpi_scale_from_wparam(value: usize) -> f64 {
    let dpi = ((value >> 16) & 0xffff) as f64;
    if dpi > 0.0 { dpi / 96.0 } else { 1.0 }
}

#[cfg(test)]
mod tests {
    use super::dpi_scale_from_wparam;

    #[test]
    fn dpi_scale_uses_high_word_of_dpi_message() {
        assert!((dpi_scale_from_wparam(144usize << 16) - 1.5).abs() < f64::EPSILON);
        assert_eq!(dpi_scale_from_wparam(0), 1.0);
    }
}

fn wide(value: &str) -> Vec<u16> {
    OsStr::new(value)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

unsafe fn win32_surface_handle(
    hwnd: HWND,
    instance: windows::Win32::Foundation::HMODULE,
) -> EmbeddedVelloSurfaceHandle {
    let hwnd = NonZeroIsize::new(hwnd.0 as isize).expect("CreateWindowExW returned a null HWND");
    let mut handle = Win32WindowHandle::new(hwnd);
    handle.hinstance = NonZeroIsize::new(instance.0 as isize);
    EmbeddedVelloSurfaceHandle::from_raw(
        RawDisplayHandle::Windows(WindowsDisplayHandle::new()),
        RawWindowHandle06::Win32(handle),
    )
}
