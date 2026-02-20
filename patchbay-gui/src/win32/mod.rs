//! Win32 window creation and message handling.

use crate::canvas::{Canvas, Color, Point, Size};
use crate::declarative::{
    LayoutEngineState, UiAction, UiInvalidationScope, UiSpec, plan_root_render,
    render_checked_with_engine,
};
use crate::host::{GuiError, InputState};
use crate::logging::log_line_safe;
use crate::renderer::{PresentationTransform, Renderer, RendererDevice};
use crate::ui::{Layout, Theme, Ui, UiState, WidgetId};
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
    PAINTSTRUCT, ReleaseDC, ScreenToClient,
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
    HTCLIENT, LoadCursorW, MA_ACTIVATE, RegisterClassW, SW_HIDE, SW_SHOW, SWP_NOACTIVATE,
    SWP_NOMOVE, SWP_NOZORDER, SetTimer, SetWindowLongPtrW, SetWindowPos, ShowWindow, WM_CHAR,
    WM_DESTROY, WM_DROPFILES, WM_ERASEBKGND, WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_LBUTTONUP,
    WM_MOUSEACTIVATE, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_NCDESTROY, WM_NCHITTEST, WM_PAINT,
    WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SIZE, WM_TIMER, WNDCLASSW, WS_CHILD, WS_CLIPCHILDREN,
    WS_CLIPSIBLINGS,
};

const TIMER_ID: usize = 1;
const TIMER_INTERVAL_MS: u32 = 16;
const PREWARM_FRAMES: u8 = 2;
const MIN_SHOW_DELAY_MS: u128 = 80;

include!("window_handle_types.rs");
include!("window_state_core.rs");
include!("message_dispatch.rs");
include!("window_state_sizing_input.rs");
include!("render_loop.rs");
include!("resize_helpers.rs");
include!("thread_spawn.rs");
include!("thread_spawn_context.rs");
include!("thread_spawn_window_class.rs");
include!("thread_spawn_renderer.rs");
include!("thread_spawn_state_build.rs");
include!("window_state_drop.rs");
include!("window_proc.rs");
include!("win32_utils.rs");
include!("tests.rs");
