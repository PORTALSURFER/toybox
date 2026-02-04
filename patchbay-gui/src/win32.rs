//! Win32 window creation and message handling.

use crate::canvas::{Canvas, Color, Point, Size};
use crate::host::{GuiError, InputState};
use crate::logging::log_line_safe;
use crate::renderer::{Renderer, RendererDevice};
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
use std::sync::{Arc, Mutex};
use std::time::Instant;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateSolidBrush, DeleteObject, EndPaint, FillRect, HBRUSH, HDC, PAINTSTRUCT,
};
use windows::Win32::System::LibraryLoader::{GetModuleHandleExW, GetModuleHandleW, GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS};
use windows::Win32::UI::Input::KeyboardAndMouse::{ReleaseCapture, SetCapture};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetClientRect, GetParent, LoadCursorW,
    RegisterClassW, SendMessageW, SetTimer, SetWindowLongPtrW, SetWindowPos, ShowWindow,
    CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA, HMENU, SWP_NOZORDER, SW_HIDE, SW_SHOW,
    WM_DESTROY, WM_ERASEBKGND, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL,
    WM_NCDESTROY, WM_PAINT, WM_SIZE, WM_TIMER, WNDCLASSW, WS_CHILD, WS_CLIPSIBLINGS,
    WS_CLIPCHILDREN, WS_VISIBLE,
};

const TIMER_ID: usize = 1;
const TIMER_INTERVAL_MS: u32 = 16;
const PREWARM_FRAMES: u8 = 2;
const MIN_SHOW_DELAY_MS: u128 = 80;
const BACKGROUND_COLOR: COLORREF = COLORREF(18 | (19 << 8) | (22 << 16));

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

    /// Show or hide the window.
    pub fn set_visible(&self, visible: bool) {
        unsafe {
            ShowWindow(self.hwnd, if visible { SW_SHOW } else { SW_HIDE });
        }
    }

    /// Return true if the HWND is still valid.
    pub fn is_valid(&self) -> bool {
        unsafe { windows::Win32::UI::WindowsAndMessaging::IsWindow(Some(self.hwnd)).as_bool() }
    }

    /// Return true if the parent matches the provided HWND.
    pub fn parent_matches(&self, parent: isize) -> bool {
        unsafe { GetParent(self.hwnd).ok() == Some(HWND(parent as *mut _)) }
    }

    /// Destroy the underlying HWND.
    pub fn destroy(&self) {
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

unsafe impl Send for WindowHandle {}
unsafe impl Sync for WindowHandle {}

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
    background_brush: HBRUSH,
    state: State,
    on_init: Init,
    on_frame: Frame,
    resize_request: Arc<AtomicU64>,
    last_size: Arc<AtomicU64>,
    aspect_ratio: Arc<AtomicU32>,
    initialized: bool,
    shown: bool,
    prewarm_frames: u8,
    saw_timer_tick: bool,
    created_at: Instant,
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
                    self.saw_timer_tick = true;
                    self.render_frame();
                    true
                } else {
                    false
                }
            }
            WM_ERASEBKGND => {
                let mut rect = windows::Win32::Foundation::RECT::default();
                unsafe {
                    GetClientRect(self.hwnd, &mut rect);
                    FillRect(HDC(wparam.0 as *mut _), &rect, self.background_brush);
                }
                true
            }
            WM_DESTROY => true,
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
                    None,
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
        let render_ok = self.renderer.render().is_ok();
        if !self.shown {
            let ready_by_time = self.created_at.elapsed().as_millis() >= MIN_SHOW_DELAY_MS;
            if self.saw_timer_tick && render_ok && self.prewarm_frames > 0 {
                self.prewarm_frames = self.prewarm_frames.saturating_sub(1);
                log_line_safe(&format!(
                    "win32: show gate prewarm tick prewarm={} saw_timer={} render_ok={}",
                    self.prewarm_frames,
                    self.saw_timer_tick,
                    render_ok
                ));
            }

            if self.saw_timer_tick && render_ok && self.prewarm_frames == 0 && ready_by_time {
                log_line_safe("win32: show gate passed, showing window");
                unsafe {
                    ShowWindow(self.hwnd, SW_SHOW);
                }
                self.shown = true;
            } else {
                log_line_safe(&format!(
                    "win32: show gate blocked prewarm={} saw_timer={} render_ok={} ready_by_time={}",
                    self.prewarm_frames,
                    self.saw_timer_tick,
                    render_ok,
                    ready_by_time
                ));
            }
        }

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
    device_cache: Arc<Mutex<Option<Arc<RendererDevice>>>>,
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
    log_line_safe("win32: spawn_window_thread begin (using caller thread)");
    create_window_on_thread(
        parent_hwnd,
        parent_hinstance,
        title,
        size,
        state,
        on_init,
        on_frame,
        device_cache,
        resize_request,
        last_size,
        aspect_ratio,
        ui_state,
        layout,
        theme,
    )
}

fn create_window_on_thread<State, Init, Frame>(
    parent_hwnd: isize,
    parent_hinstance: isize,
    title: String,
    size: Size,
    mut state: State,
    mut on_init: Init,
    mut on_frame: Frame,
    device_cache: Arc<Mutex<Option<Arc<RendererDevice>>>>,
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
    log_line_safe("win32: create_window_on_thread begin");
    let class_name = to_wide("PatchbayGuiWindow");
    let parent_hwnd = HWND(parent_hwnd as *mut _);
    let parent_hinstance = HINSTANCE(parent_hinstance as *mut _);
    let module_hinstance = if parent_hinstance.0.is_null() {
        let mut module = windows::Win32::Foundation::HMODULE::default();
        let proc_addr = window_proc::<State, Init, Frame> as *const () as *const u16;
        let got_module = unsafe {
            GetModuleHandleExW(
                GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
                windows::core::PCWSTR(proc_addr),
                &mut module,
            )
        }
        .is_ok();
        if got_module {
            HINSTANCE(module.0)
        } else {
            unsafe { GetModuleHandleW(None).unwrap_or_default().into() }
        }
    } else {
        parent_hinstance
    };
    if parent_hinstance.0.is_null() {
        log_line_safe(&format!(
            "win32: parent hinstance was null, using module hinstance={:?}",
            module_hinstance
        ));
    }
    unsafe {
        if !windows::Win32::UI::WindowsAndMessaging::IsWindow(Some(parent_hwnd)).as_bool() {
            log_line_safe(&format!(
                "win32: invalid parent hwnd={:?}; aborting CreateWindowExW",
                parent_hwnd
            ));
            return Err(GuiError::WindowCreateFailed);
        }
    }

    unsafe {
        let background_brush = unsafe { CreateSolidBrush(BACKGROUND_COLOR) };
        let wnd_class = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc::<State, Init, Frame>),
            hInstance: module_hinstance,
            lpszClassName: PCWSTR(class_name.as_ptr()),
            hCursor: LoadCursorW(None, windows::Win32::UI::WindowsAndMessaging::IDC_ARROW)
                .unwrap(),
            hbrBackground: background_brush,
            ..Default::default()
        };
        RegisterClassW(&wnd_class);
    }
    log_line_safe("win32: RegisterClassW completed");

    let title_w = to_wide(&title);
    log_line_safe(&format!(
        "win32: CreateWindowExW begin title=\"{}\" size={}x{} parent_hwnd={:?} parent_hinstance={:?} module_hinstance={:?}",
        title,
        size.width,
        size.height,
        parent_hwnd,
        parent_hinstance,
        module_hinstance
    ));
    let child_hwnd = unsafe {
        CreateWindowExW(
            Default::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(title_w.as_ptr()),
            WS_CHILD | WS_CLIPSIBLINGS | WS_CLIPCHILDREN,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            size.width as i32,
            size.height as i32,
            Some(parent_hwnd),
            Some(HMENU(std::ptr::null_mut())),
            Some(module_hinstance),
            None,
        )
    }
    .map_err(|err| {
        log_line_safe(&format!("win32: CreateWindowExW error: {err:?}"));
        GuiError::WindowCreateFailed
    })?;
    log_line_safe(&format!("win32: CreateWindowExW ok hwnd={:?}", child_hwnd));
    unsafe {
        ShowWindow(child_hwnd, SW_HIDE);
    }

    let window = SurfaceWindow {
        hwnd: child_hwnd,
        hinstance: module_hinstance,
    };
    log_line_safe("win32: creating renderer");
    let renderer_device = {
        let mut cache = device_cache.lock().map_err(|_| GuiError::DeviceCachePoison)?;
        if let Some(device) = cache.as_ref() {
            Arc::clone(device)
        } else {
            let device = Arc::new(RendererDevice::new()?);
            *cache = Some(Arc::clone(&device));
            device
        }
    };
    let renderer = Renderer::new_with_device(renderer_device, window, size)?;
    log_line_safe("win32: renderer created");
    let canvas = Canvas::new(size.width, size.height);
    let background_brush = unsafe { CreateSolidBrush(colorref_from_theme(theme.background)) };

    let mut window_state = Box::new(WindowState {
        hwnd: child_hwnd,
        renderer,
        canvas,
        input: InputState::default(),
        ui_state,
        layout,
        layout_origin: layout.cursor,
        theme,
        background_brush,
        state,
        on_init,
        on_frame,
        resize_request,
        last_size,
        aspect_ratio,
        initialized: false,
        shown: false,
        prewarm_frames: PREWARM_FRAMES,
        saw_timer_tick: false,
        created_at: Instant::now(),
    });

    unsafe {
        let state_ptr = Box::into_raw(window_state);
        SetWindowLongPtrW(child_hwnd, GWLP_USERDATA, state_ptr as isize);
        SetTimer(Some(child_hwnd), TIMER_ID, TIMER_INTERVAL_MS, None);
        log_line_safe("win32: initial window hidden; waiting for show gate");
        // Render once; on success it will reveal the window.
        let state = &mut *(state_ptr as *mut WindowState<State, Init, Frame>);
        state.render_frame();
    }

    let handle = WindowHandle { hwnd: child_hwnd };
    Ok(handle)
}

impl<State, Init, Frame> Drop for WindowState<State, Init, Frame>
where
    Init: FnMut(&mut Ui<'_>, &mut State) + Send + 'static,
    Frame: FnMut(&mut Ui<'_>, &mut State) + Send + 'static,
    State: Send + 'static,
{
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteObject(self.background_brush.into());
        }
    }
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
    let ptr = unsafe {
        windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(hwnd, GWLP_USERDATA)
    };
    if ptr != 0 {
        let state = unsafe { &mut *(ptr as *mut WindowState<State, Init, Frame>) };
        if state.handle_message(message, wparam, lparam) {
            return LRESULT(0);
        }
    }

    if message == WM_NCDESTROY {
        if ptr != 0 {
            unsafe {
                windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                drop(Box::from_raw(ptr as *mut WindowState<State, Init, Frame>));
            }
        }
    }

    unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
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

fn colorref_from_theme(color: Color) -> COLORREF {
    COLORREF(color.r as u32 | ((color.g as u32) << 8) | ((color.b as u32) << 16))
}
