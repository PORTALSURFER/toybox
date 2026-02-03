//! Win32 window creation and message handling.

use crate::canvas::{Canvas, Point, Size};
use crate::host::{GuiError, InputState};
use crate::renderer::Renderer;
use crate::ui::{Layout, Theme, Ui, UiState};
use raw_window_handle_06::{
    DisplayHandle, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle as RawWindowHandle06, Win32WindowHandle, WindowHandle as WindowHandle06,
    WindowsDisplayHandle,
};
use std::ffi::OsStr;
use std::num::NonZeroIsize;
use std::os::windows::ffi::OsStrExt;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, PAINTSTRUCT};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{ReleaseCapture, SetCapture};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, LoadCursorW,
    PeekMessageW, PostQuitMessage, RegisterClassW, SetTimer, SetWindowLongPtrW, SetWindowPos,
    TranslateMessage, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA, HMENU, MSG,
    PM_REMOVE, SWP_NOZORDER, WM_DESTROY, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE,
    WM_MOUSEWHEEL, WM_NCDESTROY, WM_PAINT, WM_SIZE, WM_TIMER, WNDCLASSW, WS_CHILD,
    WS_CLIPSIBLINGS, WS_CLIPCHILDREN, WS_VISIBLE,
};

const TIMER_ID: usize = 1;
const TIMER_INTERVAL_MS: u32 = 16;

/// Thin wrapper around an HWND for cross-thread use.
#[derive(Clone, Debug)]
pub struct WindowHandle {
    hwnd: HWND,
}

impl WindowHandle {
    /// Return the underlying HWND.
    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }
}

/// A window type that exposes raw window handles for wgpu surfaces.
pub struct SurfaceWindow {
    hwnd: HWND,
    hinstance: HINSTANCE,
}

impl HasWindowHandle for SurfaceWindow {
    fn window_handle(&self) -> Result<WindowHandle06<'_>, raw_window_handle_06::HandleError> {
        let hwnd = NonZeroIsize::new(self.hwnd.0 as isize).expect("HWND must be non-null");
        let mut handle = Win32WindowHandle::new(hwnd);
        if let Some(hinstance) = NonZeroIsize::new(self.hinstance.0 as isize) {
            handle.hinstance = Some(hinstance);
        }
        Ok(unsafe { WindowHandle06::borrow_raw(RawWindowHandle06::Win32(handle)) })
    }
}

impl HasDisplayHandle for SurfaceWindow {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, raw_window_handle_06::HandleError> {
        let display = WindowsDisplayHandle::new();
        Ok(unsafe { DisplayHandle::borrow_raw(RawDisplayHandle::Windows(display)) })
    }
}

unsafe impl Send for SurfaceWindow {}
unsafe impl Sync for SurfaceWindow {}

struct WindowState<State, Init, Frame>
where
    Init: FnMut(&mut Ui<'_>, &mut State) + Send + 'static,
    Frame: FnMut(&mut Ui<'_>, &mut State) + Send + 'static,
    State: Send + 'static,
{
    hwnd: HWND,
    renderer: Renderer,
    canvas: Canvas,
    input: InputState,
    ui_state: UiState,
    layout: Layout,
    layout_origin: Point,
    theme: Theme,
    state: State,
    on_init: Init,
    on_frame: Frame,
    resize_request: Arc<AtomicU64>,
    last_size: Arc<AtomicU64>,
    aspect_ratio: Arc<AtomicU32>,
    initialized: bool,
}

impl<State, Init, Frame> WindowState<State, Init, Frame>
where
    Init: FnMut(&mut Ui<'_>, &mut State) + Send + 'static,
    Frame: FnMut(&mut Ui<'_>, &mut State) + Send + 'static,
    State: Send + 'static,
{
    fn handle_message(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> bool {
        match message {
            WM_SIZE => {
                self.on_resize();
                true
            }
            WM_MOUSEMOVE => {
                let x = (lparam.0 & 0xFFFF) as i16 as i32;
                let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
                self.input.pointer_pos = Point { x, y };
                true
            }
            WM_LBUTTONDOWN => {
                self.input.mouse_down = true;
                self.input.mouse_pressed = true;
                unsafe { SetCapture(self.hwnd) };
                true
            }
            WM_LBUTTONUP => {
                self.input.mouse_down = false;
                self.input.mouse_released = true;
                unsafe {
                    let _ = ReleaseCapture();
                }
                true
            }
            WM_MOUSEWHEEL => {
                let delta = ((wparam.0 >> 16) & 0xFFFF) as i16 as f32 / 120.0;
                self.input.wheel_delta += delta;
                true
            }
            WM_PAINT => {
                unsafe {
                    let mut paint = PAINTSTRUCT::default();
                    BeginPaint(self.hwnd, &mut paint);
                    EndPaint(self.hwnd, &paint);
                }
                self.render_frame();
                true
            }
            WM_TIMER => {
                if wparam.0 == TIMER_ID {
                    self.render_frame();
                    true
                } else {
                    false
                }
            }
            WM_DESTROY => {
                unsafe { PostQuitMessage(0) };
                true
            }
            _ => false,
        }
    }

    fn on_resize(&mut self) {
        let mut rect = windows::Win32::Foundation::RECT::default();
        unsafe {
            GetClientRect(self.hwnd, &mut rect);
        }
        let width = (rect.right - rect.left).max(1) as u32;
        let height = (rect.bottom - rect.top).max(1) as u32;
        self.last_size
            .store(pack_size(width, height), Ordering::Release);
        self.canvas.resize(width, height);
        self.renderer.resize(Size { width, height });
    }

    fn render_frame(&mut self) {
        if !self.initialized {
            let mut ui = Ui::new(
                &mut self.canvas,
                &self.input,
                &mut self.ui_state,
                &mut self.layout,
                &self.theme,
            );
            (self.on_init)(&mut ui, &mut self.state);
            self.initialized = true;
        }

        if let Some((width, height)) = unpack_size(self.resize_request.swap(0, Ordering::AcqRel))
        {
            let mut height = height;
            let aspect_bits = self.aspect_ratio.load(Ordering::Relaxed);
            if aspect_bits != 0 {
                let aspect = f32::from_bits(aspect_bits);
                height = (width as f32 / aspect).round().max(1.0) as u32;
            }
            unsafe {
                SetWindowPos(
                    self.hwnd,
                    HWND(std::ptr::null_mut()),
                    0,
                    0,
                    width as i32,
                    height as i32,
                    SWP_NOZORDER,
                );
            }
            self.on_resize();
        }

        self.layout.cursor = self.layout_origin;
        self.canvas.clear(self.theme.background);

        {
            let mut ui = Ui::new(
                &mut self.canvas,
                &self.input,
                &mut self.ui_state,
                &mut self.layout,
                &self.theme,
            );
            (self.on_frame)(&mut ui, &mut self.state);
        }

        self.renderer.upload(self.canvas.size(), self.canvas.pixels());
        let _ = self.renderer.render();

        self.input.mouse_pressed = false;
        self.input.mouse_released = false;
        self.input.wheel_delta = 0.0;
    }
}

/// Spawn a GUI thread that owns the Win32 window and render loop.
pub fn spawn_window_thread<State, Init, Frame>(
    parent_hwnd: isize,
    parent_hinstance: isize,
    title: String,
    size: Size,
    state: State,
    on_init: Init,
    on_frame: Frame,
    resize_request: Arc<AtomicU64>,
    last_size: Arc<AtomicU64>,
    aspect_ratio: Arc<AtomicU32>,
    ui_state: UiState,
    layout: Layout,
    theme: Theme,
) -> Result<WindowHandle, GuiError>
where
    Init: FnMut(&mut Ui<'_>, &mut State) + Send + 'static,
    Frame: FnMut(&mut Ui<'_>, &mut State) + Send + 'static,
    State: Send + 'static,
{
    let (tx, rx) = mpsc::channel();

    thread::Builder::new()
        .name("patchbay-gui".to_string())
        .spawn(move || {
            let result = run_window_loop(
                parent_hwnd,
                title,
                size,
                state,
                on_init,
                on_frame,
                resize_request,
                last_size,
                aspect_ratio,
                ui_state,
                layout,
                theme,
                &tx,
            );
            let _ = tx.send(result);
        })
        .map_err(|_| GuiError::ThreadSpawn)?;

    rx.recv().map_err(|_| GuiError::ThreadSpawn)?
}

fn run_window_loop<State, Init, Frame>(
    parent_hwnd: isize,
    parent_hinstance: isize,
    title: String,
    size: Size,
    mut state: State,
    mut on_init: Init,
    mut on_frame: Frame,
    resize_request: Arc<AtomicU64>,
    last_size: Arc<AtomicU64>,
    aspect_ratio: Arc<AtomicU32>,
    ui_state: UiState,
    layout: Layout,
    theme: Theme,
    ready: &mpsc::Sender<Result<WindowHandle, GuiError>>,
) -> Result<WindowHandle, GuiError>
where
    Init: FnMut(&mut Ui<'_>, &mut State) + Send + 'static,
    Frame: FnMut(&mut Ui<'_>, &mut State) + Send + 'static,
    State: Send + 'static,
{
    let class_name = to_wide("PatchbayGuiWindow");
    let hinstance = unsafe { GetModuleHandleW(PCWSTR::null()) }
        .map_err(|_| GuiError::WindowCreateFailed)?;
    let hinstance = HINSTANCE(hinstance.0);
    let parent_hwnd = HWND(parent_hwnd as *mut _);
    let parent_hinstance = HINSTANCE(parent_hinstance as *mut _);

    unsafe {
        let wnd_class = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc::<State, Init, Frame>),
            hInstance: parent_hinstance,
            lpszClassName: PCWSTR(class_name.as_ptr()),
            hCursor: LoadCursorW(None, windows::Win32::UI::WindowsAndMessaging::IDC_ARROW)
                .unwrap(),
            ..Default::default()
        };
        RegisterClassW(&wnd_class);
    }

    let title_w = to_wide(&title);
    let child_hwnd = unsafe {
        CreateWindowExW(
            Default::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(title_w.as_ptr()),
            WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS | WS_CLIPCHILDREN,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            size.width as i32,
            size.height as i32,
            parent_hwnd,
            HMENU(std::ptr::null_mut()),
            parent_hinstance,
            None,
        )
    }
    .map_err(|_| GuiError::WindowCreateFailed)?;

    let window = SurfaceWindow {
        hwnd: child_hwnd,
        hinstance: parent_hinstance,
    };
    let renderer = Renderer::new(window, size)?;
    let canvas = Canvas::new(size.width, size.height);

    let mut window_state = Box::new(WindowState {
        hwnd: child_hwnd,
        renderer,
        canvas,
        input: InputState::default(),
        ui_state,
        layout,
        layout_origin: layout.cursor,
        theme,
        state,
        on_init,
        on_frame,
        resize_request,
        last_size,
        aspect_ratio,
        initialized: false,
    });

    unsafe {
        SetWindowLongPtrW(child_hwnd, GWLP_USERDATA, &mut *window_state as *mut _ as isize);
        SetTimer(child_hwnd, TIMER_ID, TIMER_INTERVAL_MS, None);
    }

    let handle = WindowHandle { hwnd: child_hwnd };
    let _ = ready.send(Ok(handle.clone()));

    let mut msg = MSG::default();
    loop {
        unsafe {
            while PeekMessageW(&mut msg, HWND(std::ptr::null_mut()), 0, 0, PM_REMOVE).into() {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        if msg.message == WM_NCDESTROY {
            break;
        }
        if msg.message == windows::Win32::UI::WindowsAndMessaging::WM_QUIT {
            break;
        }
        thread::sleep(std::time::Duration::from_millis(1));
    }

    unsafe {
        SetWindowLongPtrW(child_hwnd, GWLP_USERDATA, 0);
    }
    drop(window_state);

    Ok(handle)
}

unsafe extern "system" fn window_proc<State, Init, Frame>(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT
where
    Init: FnMut(&mut Ui<'_>, &mut State) + Send + 'static,
    Frame: FnMut(&mut Ui<'_>, &mut State) + Send + 'static,
    State: Send + 'static,
{
    let ptr = windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(hwnd, GWLP_USERDATA);
    if ptr != 0 {
        let state = &mut *(ptr as *mut WindowState<State, Init, Frame>);
        if state.handle_message(message, wparam, lparam) {
            return LRESULT(0);
        }
    }

    if message == WM_NCDESTROY {
        PostQuitMessage(0);
    }

    DefWindowProcW(hwnd, message, wparam, lparam)
}

fn pack_size(width: u32, height: u32) -> u64 {
    ((width as u64) << 32) | (height as u64)
}

fn unpack_size(value: u64) -> Option<(u32, u32)> {
    if value == 0 {
        return None;
    }
    let width = (value >> 32) as u32;
    let height = (value & 0xFFFF_FFFF) as u32;
    Some((width, height))
}

fn to_wide(text: &str) -> Vec<u16> {
    OsStr::new(text).encode_wide().chain(Some(0)).collect()
}
