//! Win32 window creation and message handling.

use crate::canvas::{Canvas, Color, Point, Size};
use crate::declarative::{RootScaleMode, UiAction, UiSpec, measure_checked, render_checked};
use crate::host::{GuiError, InputState};
use crate::logging::log_line_safe;
use crate::renderer::{Renderer, RendererDevice};
use crate::ui::{Layout, Theme, Ui, UiState};
use raw_window_handle_06::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle as RawWindowHandle06, Win32WindowHandle, WindowHandle as WindowHandle06,
    WindowsDisplayHandle,
};
use std::ffi::OsStr;
use std::num::NonZeroIsize;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateSolidBrush, DeleteObject, EndPaint, FillRect, GetDC, HBRUSH, HDC,
    PAINTSTRUCT, ReleaseDC,
};
use windows::Win32::System::LibraryLoader::{
    GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GetModuleHandleExW, GetModuleHandleW,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, ReleaseCapture, SetCapture, VK_LBUTTON, VK_MENU, VK_RBUTTON, VK_SHIFT,
};
use windows::Win32::UI::Shell::{DragAcceptFiles, DragFinish, DragQueryFileW, HDROP};
use windows::Win32::UI::WindowsAndMessaging::{
    CS_DBLCLKS, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW,
    DestroyWindow, GWLP_USERDATA, GetClientRect, GetCursorPos, GetParent, GetWindowRect, HMENU,
    HTCLIENT, LoadCursorW, MA_ACTIVATE, RegisterClassW, SW_HIDE, SW_SHOW, SWP_NOZORDER, SetTimer,
    SetWindowLongPtrW, SetWindowPos, ShowWindow, WM_CHAR, WM_DESTROY, WM_DROPFILES, WM_ERASEBKGND,
    WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEACTIVATE, WM_MOUSEMOVE, WM_MOUSEWHEEL,
    WM_NCDESTROY, WM_NCHITTEST, WM_PAINT, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SIZE, WM_TIMER,
    WNDCLASSW, WS_CHILD, WS_CLIPCHILDREN, WS_CLIPSIBLINGS,
};
use windows::core::PCWSTR;

const TIMER_ID: usize = 1;
const TIMER_INTERVAL_MS: u32 = 16;
const PREWARM_FRAMES: u8 = 2;
const MIN_SHOW_DELAY_MS: u128 = 80;
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
            let _ = ShowWindow(self.hwnd, if visible { SW_SHOW } else { SW_HIDE });
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
        let Some(hwnd) = NonZeroIsize::new(self.hwnd.0 as isize) else {
            return Err(HandleError::Unavailable);
        };
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

struct WindowState<State, Init, Build, Reduce>
where
    Init: FnMut(&mut State) + Send + 'static,
    Build: FnMut(&InputState, &State) -> UiSpec + Send + 'static,
    Reduce: FnMut(&mut State, UiAction) + Send + 'static,
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
    build_spec: Build,
    reduce_action: Reduce,
    resize_request: Arc<AtomicU64>,
    last_size: Arc<AtomicU64>,
    aspect_ratio: Arc<AtomicU32>,
    initialized: bool,
    shown: bool,
    prewarm_frames: u8,
    created_at: Instant,
    last_mouse_down: bool,
    last_mouse_secondary_down: bool,
    debug_input: bool,
    frame_counter: u64,
}

impl<State, Init, Build, Reduce> WindowState<State, Init, Build, Reduce>
where
    Init: FnMut(&mut State) + Send + 'static,
    Build: FnMut(&InputState, &State) -> UiSpec + Send + 'static,
    Reduce: FnMut(&mut State, UiAction) + Send + 'static,
    State: Send + 'static,
{
    fn handle_message(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        match message {
            WM_SIZE => {
                self.on_resize();
                Some(LRESULT(0))
            }
            WM_NCHITTEST => {
                // Always treat the plugin surface as client area so it consumes mouse input.
                Some(LRESULT(HTCLIENT as isize))
            }
            WM_MOUSEACTIVATE => {
                // Activate the window and consume the click within the plugin surface.
                Some(LRESULT(MA_ACTIVATE as isize))
            }
            WM_MOUSEMOVE => {
                let x = (lparam.0 & 0xFFFF) as i16 as i32;
                let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
                self.input.pointer_pos = Point { x, y };
                self.render_frame();
                Some(LRESULT(0))
            }
            WM_LBUTTONDOWN => {
                self.input.mouse_down = true;
                self.input.mouse_pressed = true;
                unsafe { SetCapture(self.hwnd) };
                self.render_frame();
                Some(LRESULT(0))
            }
            WM_LBUTTONDBLCLK => {
                self.input.mouse_double_clicked = true;
                self.input.mouse_down = true;
                self.input.mouse_pressed = true;
                unsafe { SetCapture(self.hwnd) };
                self.render_frame();
                Some(LRESULT(0))
            }
            WM_LBUTTONUP => {
                self.input.mouse_down = false;
                self.input.mouse_released = true;
                unsafe {
                    let _ = ReleaseCapture();
                }
                self.render_frame();
                Some(LRESULT(0))
            }
            WM_RBUTTONDOWN => {
                self.input.mouse_secondary_down = true;
                self.input.mouse_secondary_pressed = true;
                unsafe { SetCapture(self.hwnd) };
                self.render_frame();
                Some(LRESULT(0))
            }
            WM_RBUTTONUP => {
                self.input.mouse_secondary_down = false;
                self.input.mouse_secondary_released = true;
                unsafe {
                    let _ = ReleaseCapture();
                }
                self.render_frame();
                Some(LRESULT(0))
            }
            WM_MOUSEWHEEL => {
                let delta = ((wparam.0 >> 16) & 0xFFFF) as i16 as f32 / 120.0;
                self.input.wheel_delta += delta;
                self.render_frame();
                Some(LRESULT(0))
            }
            WM_DROPFILES => {
                let hdrop = HDROP(wparam.0 as *mut _);
                self.input.dropped_files = collect_dropped_files(hdrop);
                unsafe {
                    DragFinish(hdrop);
                }
                self.render_frame();
                Some(LRESULT(0))
            }
            WM_CHAR => {
                let code = (wparam.0 & 0xFFFF) as u16;
                if let Some(ch) = char::from_u32(code as u32) {
                    self.input.key_pressed = Some(ch);
                }
                self.render_frame();
                Some(LRESULT(0))
            }
            WM_PAINT => {
                unsafe {
                    let mut paint = PAINTSTRUCT::default();
                    let hdc = BeginPaint(self.hwnd, &mut paint);
                    FillRect(hdc, &paint.rcPaint, self.background_brush);
                    let _ = EndPaint(self.hwnd, &paint);
                }
                self.render_frame();
                Some(LRESULT(0))
            }
            WM_TIMER => {
                if wparam.0 == TIMER_ID {
                    self.render_frame();
                    Some(LRESULT(0))
                } else {
                    None
                }
            }
            WM_ERASEBKGND => {
                let mut rect = windows::Win32::Foundation::RECT::default();
                let hdc = if wparam.0 == 0 {
                    unsafe { GetDC(Some(self.hwnd)) }
                } else {
                    HDC(wparam.0 as *mut _)
                };
                unsafe {
                    if GetClientRect(self.hwnd, &mut rect).is_err() {
                        log_line_safe("win32: GetClientRect failed in WM_ERASEBKGND");
                    }
                    FillRect(hdc, &rect, self.background_brush);
                }
                if wparam.0 == 0 {
                    unsafe {
                        let _ = ReleaseDC(Some(self.hwnd), hdc);
                    }
                }
                Some(LRESULT(0))
            }
            WM_DESTROY => Some(LRESULT(0)),
            _ => None,
        }
    }

    fn on_resize(&mut self) {
        let mut rect = windows::Win32::Foundation::RECT::default();
        unsafe {
            if GetClientRect(self.hwnd, &mut rect).is_err() {
                log_line_safe("win32: GetClientRect failed in on_resize");
                return;
            }
        }
        let width = (rect.right - rect.left).max(1) as u32;
        let height = (rect.bottom - rect.top).max(1) as u32;
        self.last_size
            .store(pack_size(width, height), Ordering::Release);
        self.input.window_size = Size { width, height };
        self.canvas.resize(width, height);
        self.renderer.resize(Size { width, height });
    }

    fn sync_pointer_pos(&mut self) {
        let mut point = windows::Win32::Foundation::POINT::default();
        if unsafe { GetCursorPos(&mut point) }.is_err() {
            return;
        }
        let mut window_rect = windows::Win32::Foundation::RECT::default();
        if unsafe { GetWindowRect(self.hwnd, &mut window_rect) }.is_err() {
            return;
        }
        let local_x = point.x - window_rect.left;
        let local_y = point.y - window_rect.top;

        let mut client_rect = windows::Win32::Foundation::RECT::default();
        if unsafe { GetClientRect(self.hwnd, &mut client_rect) }.is_err() {
            self.input.pointer_pos = Point {
                x: local_x,
                y: local_y,
            };
            return;
        }
        let client_width = (client_rect.right - client_rect.left).max(1) as i32;
        let client_height = (client_rect.bottom - client_rect.top).max(1) as i32;
        let canvas_size = self.canvas.size();
        let scaled_x = (local_x as i64 * canvas_size.width as i64 / client_width as i64) as i32;
        let scaled_y = (local_y as i64 * canvas_size.height as i64 / client_height as i64) as i32;

        self.input.pointer_pos = Point {
            x: scaled_x,
            y: scaled_y,
        };
    }

    fn sync_mouse_buttons(&mut self) {
        let primary_now = unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) } < 0;
        let secondary_now = unsafe { GetAsyncKeyState(VK_RBUTTON.0 as i32) } < 0;
        let shift_now = unsafe { GetAsyncKeyState(VK_SHIFT.0 as i32) } < 0;
        let alt_now = unsafe { GetAsyncKeyState(VK_MENU.0 as i32) } < 0;

        self.input.mouse_pressed = primary_now && !self.last_mouse_down;
        self.input.mouse_released = !primary_now && self.last_mouse_down;
        self.input.mouse_down = primary_now;

        self.input.mouse_secondary_pressed = secondary_now && !self.last_mouse_secondary_down;
        self.input.mouse_secondary_released = !secondary_now && self.last_mouse_secondary_down;
        self.input.mouse_secondary_down = secondary_now;
        self.input.shift_down = shift_now;
        self.input.alt_down = alt_now;

        self.last_mouse_down = primary_now;
        self.last_mouse_secondary_down = secondary_now;
    }

    fn render_frame(&mut self) {
        self.frame_counter = self.frame_counter.wrapping_add(1);
        if !self.initialized {
            (self.on_init)(&mut self.state);
            self.initialized = true;
        }

        if let Some((width, height)) = unpack_size(self.resize_request.swap(0, Ordering::AcqRel)) {
            let mut width = width;
            let mut height = height;
            let aspect_bits = self.aspect_ratio.load(Ordering::Relaxed);
            if aspect_bits != 0 {
                let aspect = f32::from_bits(aspect_bits);
                (width, height) = enforce_aspect_min(width, height, aspect);
            }
            unsafe {
                if let Err(err) = SetWindowPos(
                    self.hwnd,
                    None,
                    0,
                    0,
                    width as i32,
                    height as i32,
                    SWP_NOZORDER,
                ) {
                    log_line_safe(&format!("win32: SetWindowPos failed: {err:?}"));
                }
            }
            self.on_resize();
        }

        self.layout.cursor = self.layout_origin;
        self.canvas.clear(self.theme.background);
        self.sync_pointer_pos();
        self.sync_mouse_buttons();

        {
            self.ui_state.begin_frame();
            let mut spec = (self.build_spec)(&self.input, &self.state);
            let mut restored_window_size = None;
            if spec.root.scale_mode == RootScaleMode::UniformFit {
                let design_size = spec.root.design_size.unwrap_or_else(|| {
                    measure_checked(&spec).unwrap_or(Size {
                        width: self.canvas.size().width.max(1),
                        height: self.canvas.size().height.max(1),
                    })
                });
                let design_size = Size {
                    width: design_size.width.max(1),
                    height: design_size.height.max(1),
                };
                if self.canvas.size() != design_size {
                    self.canvas.resize(design_size.width, design_size.height);
                    self.sync_pointer_pos();
                }
                restored_window_size = Some(self.input.window_size);
                self.input.window_size = design_size;
                spec = (self.build_spec)(&self.input, &self.state);
            }
            let mut ui = Ui::new(
                &mut self.canvas,
                &self.input,
                &mut self.ui_state,
                &mut self.layout,
                &self.theme,
            );
            ui.set_vector_text_enabled(self.renderer.vector_text_available());
            ui.reset_input_consumption();
            ui.clear_overlays();
            match render_checked(&spec, &mut ui, Point { x: 0, y: 0 }) {
                Ok(result) => {
                    for action in result.actions {
                        (self.reduce_action)(&mut self.state, action);
                    }
                }
                Err(err) => {
                    log_line_safe(&format!(
                        "win32: declarative render validation error: {err}"
                    ));
                }
            }
            ui.draw_overlays();
            self.renderer.set_vector_commands(ui.take_vector_commands());
            if let Some(window_size) = restored_window_size {
                self.input.window_size = window_size;
            }
        }
        let _ = self.ui_state.take_root_frame_size();
        if self.debug_input {
            let text = format!(
                "frame={} ptr=({}, {}) md={} mr={} rd={}",
                self.frame_counter,
                self.input.pointer_pos.x,
                self.input.pointer_pos.y,
                self.input.mouse_down as u8,
                self.input.mouse_released as u8,
                self.input.mouse_secondary_down as u8
            );
            self.canvas
                .draw_text(Point { x: 6, y: 6 }, &text, self.theme.text, 1);
        }

        self.renderer
            .upload(self.canvas.size(), self.canvas.pixels());
        let render_ok = self.renderer.render().is_ok();
        if !self.shown && render_ok {
            if self.prewarm_frames > 0 {
                self.prewarm_frames = self.prewarm_frames.saturating_sub(1);
            }
            let elapsed_ms = self.created_at.elapsed().as_millis();
            if self.prewarm_frames == 0 && elapsed_ms >= MIN_SHOW_DELAY_MS {
                log_line_safe("win32: render ok, showing window");
                unsafe {
                    let _ = ShowWindow(self.hwnd, SW_SHOW);
                }
                self.shown = true;
                let _ = self.renderer.render();
            }
        }

        self.input.mouse_pressed = false;
        self.input.mouse_released = false;
        self.input.mouse_double_clicked = false;
        self.input.mouse_secondary_pressed = false;
        self.input.mouse_secondary_released = false;
        self.input.wheel_delta = 0.0;
        self.input.key_pressed = None;
        self.input.dropped_files.clear();
    }
}

fn enforce_aspect_min(width: u32, height: u32, aspect: f32) -> (u32, u32) {
    if !aspect.is_finite() || aspect <= 0.0 {
        return (width.max(1), height.max(1));
    }
    let width_from_height = (height as f32 * aspect).ceil().max(1.0) as u32;
    let height_from_width = (width as f32 / aspect).ceil().max(1.0) as u32;
    if width_from_height >= width {
        (width_from_height, height.max(1))
    } else {
        (width.max(1), height_from_width)
    }
}

/// Spawn a GUI thread that owns the Win32 window and render loop.
pub fn spawn_window_thread<State, Init, Build, Reduce>(
    parent_hwnd: isize,
    parent_hinstance: isize,
    title: String,
    size: Size,
    state: State,
    on_init: Init,
    build: Build,
    reduce: Reduce,
    device_cache: Arc<Mutex<Option<Arc<RendererDevice>>>>,
    resize_request: Arc<AtomicU64>,
    last_size: Arc<AtomicU64>,
    aspect_ratio: Arc<AtomicU32>,
    ui_state: UiState,
    layout: Layout,
    theme: Theme,
) -> Result<WindowHandle, GuiError>
where
    Init: FnMut(&mut State) + Send + 'static,
    Build: FnMut(&InputState, &State) -> UiSpec + Send + 'static,
    Reduce: FnMut(&mut State, UiAction) + Send + 'static,
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
        build,
        reduce,
        device_cache,
        resize_request,
        last_size,
        aspect_ratio,
        ui_state,
        layout,
        theme,
    )
}

fn create_window_on_thread<State, Init, Build, Reduce>(
    parent_hwnd: isize,
    parent_hinstance: isize,
    title: String,
    size: Size,
    state: State,
    on_init: Init,
    build: Build,
    reduce: Reduce,
    device_cache: Arc<Mutex<Option<Arc<RendererDevice>>>>,
    resize_request: Arc<AtomicU64>,
    last_size: Arc<AtomicU64>,
    aspect_ratio: Arc<AtomicU32>,
    ui_state: UiState,
    layout: Layout,
    theme: Theme,
) -> Result<WindowHandle, GuiError>
where
    Init: FnMut(&mut State) + Send + 'static,
    Build: FnMut(&InputState, &State) -> UiSpec + Send + 'static,
    Reduce: FnMut(&mut State, UiAction) + Send + 'static,
    State: Send + 'static,
{
    log_line_safe("win32: create_window_on_thread begin");
    let class_name = to_wide("PatchbayGuiWindow");
    let parent_hwnd = HWND(parent_hwnd as *mut _);
    let parent_hinstance = HINSTANCE(parent_hinstance as *mut _);
    let module_hinstance = if parent_hinstance.0.is_null() {
        let mut module = windows::Win32::Foundation::HMODULE::default();
        let proc_addr = window_proc::<State, Init, Build, Reduce> as *const () as *const u16;
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

    let cursor = unsafe { LoadCursorW(None, windows::Win32::UI::WindowsAndMessaging::IDC_ARROW) }
        .map_err(|err| {
        log_line_safe(&format!("win32: LoadCursorW error: {err:?}"));
        GuiError::WindowCreateFailed
    })?;
    unsafe {
        let wnd_class = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW | CS_DBLCLKS,
            lpfnWndProc: Some(window_proc::<State, Init, Build, Reduce>),
            hInstance: module_hinstance,
            lpszClassName: PCWSTR(class_name.as_ptr()),
            hCursor: cursor,
            hbrBackground: HBRUSH(std::ptr::null_mut()),
            ..Default::default()
        };
        RegisterClassW(&wnd_class);
    }
    log_line_safe("win32: RegisterClassW completed");

    let title_w = to_wide(&title);
    log_line_safe(&format!(
        "win32: CreateWindowExW begin title=\"{}\" size={}x{} parent_hwnd={:?} parent_hinstance={:?} module_hinstance={:?}",
        title, size.width, size.height, parent_hwnd, parent_hinstance, module_hinstance
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
        let _ = ShowWindow(child_hwnd, SW_HIDE);
    }

    let window = SurfaceWindow {
        hwnd: child_hwnd,
        hinstance: module_hinstance,
    };
    log_line_safe("win32: creating renderer");
    let renderer_device = {
        let mut cache = device_cache
            .lock()
            .map_err(|_| GuiError::DeviceCachePoison)?;
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

    let window_state = Box::new(WindowState {
        hwnd: child_hwnd,
        renderer,
        canvas,
        input: InputState {
            window_size: size,
            ..InputState::default()
        },
        ui_state,
        layout,
        layout_origin: layout.cursor,
        theme,
        background_brush,
        state,
        on_init,
        build_spec: build,
        reduce_action: reduce,
        resize_request,
        last_size,
        aspect_ratio,
        initialized: false,
        shown: false,
        prewarm_frames: PREWARM_FRAMES,
        created_at: Instant::now(),
        last_mouse_down: false,
        last_mouse_secondary_down: false,
        debug_input: std::env::var_os("PATCHBAY_DEBUG_INPUT").is_some(),
        frame_counter: 0,
    });

    unsafe {
        let state_ptr = Box::into_raw(window_state);
        SetWindowLongPtrW(child_hwnd, GWLP_USERDATA, state_ptr as isize);
        SetTimer(Some(child_hwnd), TIMER_ID, TIMER_INTERVAL_MS, None);
        DragAcceptFiles(child_hwnd, true);
        log_line_safe("win32: initial window hidden; waiting for show gate");
        let state = &mut *(state_ptr as *mut WindowState<State, Init, Build, Reduce>);
        // Synchronize to the actual client rect before the first frame.
        // Some hosts may constrain the child view at create-time without
        // emitting WM_SIZE immediately, which otherwise causes a one-frame (or
        // persistent) size mismatch and clipped content.
        state.on_resize();
        // Render once; on success it will reveal the window.
        state.render_frame();
    }

    let handle = WindowHandle { hwnd: child_hwnd };
    Ok(handle)
}

impl<State, Init, Build, Reduce> Drop for WindowState<State, Init, Build, Reduce>
where
    Init: FnMut(&mut State) + Send + 'static,
    Build: FnMut(&InputState, &State) -> UiSpec + Send + 'static,
    Reduce: FnMut(&mut State, UiAction) + Send + 'static,
    State: Send + 'static,
{
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteObject(self.background_brush.into());
        }
    }
}

unsafe extern "system" fn window_proc<State, Init, Build, Reduce>(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT
where
    Init: FnMut(&mut State) + Send + 'static,
    Build: FnMut(&InputState, &State) -> UiSpec + Send + 'static,
    Reduce: FnMut(&mut State, UiAction) + Send + 'static,
    State: Send + 'static,
{
    let ptr =
        unsafe { windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(hwnd, GWLP_USERDATA) };
    if ptr != 0 {
        let state = unsafe { &mut *(ptr as *mut WindowState<State, Init, Build, Reduce>) };
        if let Some(result) = state.handle_message(message, wparam, lparam) {
            return result;
        }
    }

    if message == WM_NCDESTROY {
        if ptr != 0 {
            unsafe {
                windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                drop(Box::from_raw(
                    ptr as *mut WindowState<State, Init, Build, Reduce>,
                ));
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

fn collect_dropped_files(hdrop: HDROP) -> Vec<PathBuf> {
    let count = unsafe { DragQueryFileW(hdrop, 0xFFFF_FFFF, None) };
    let mut paths = Vec::new();
    for index in 0..count {
        let len = unsafe { DragQueryFileW(hdrop, index, None) };
        if len == 0 {
            continue;
        }
        let mut buffer = vec![0u16; (len + 1) as usize];
        let written = unsafe { DragQueryFileW(hdrop, index, Some(&mut buffer)) };
        if written == 0 {
            continue;
        }
        if let Some(path) = wide_to_path(&buffer[..written as usize]) {
            paths.push(path);
        }
    }
    paths
}

fn wide_to_path(buffer: &[u16]) -> Option<PathBuf> {
    let string = String::from_utf16(buffer).ok()?;
    if string.is_empty() {
        None
    } else {
        Some(PathBuf::from(string))
    }
}

#[cfg(test)]
mod tests {
    use super::enforce_aspect_min;

    #[test]
    fn aspect_enforces_min_dimensions() {
        let (w, h) = enforce_aspect_min(400, 300, 1.5);
        assert!(w >= 400);
        assert!(h >= 300);
        assert!((w as f32 / h as f32 - 1.5).abs() < 0.01);
    }

    #[test]
    fn aspect_noop_for_invalid_ratio() {
        assert_eq!(enforce_aspect_min(100, 80, 0.0), (100, 80));
    }
}
