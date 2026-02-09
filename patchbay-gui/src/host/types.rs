//! Host-facing input and window state types.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, AtomicU64};
use std::sync::{Arc, Mutex};

use raw_window_handle::RawWindowHandle;

use crate::canvas::Size;
use crate::renderer::RendererDevice;
use crate::win32::WindowHandle;

/// Input snapshot delivered to UI widgets for a single frame.
#[derive(Clone, Debug, Default)]
pub struct InputState {
    /// Current logical window size in pixels.
    pub window_size: Size,
    /// Current pointer position in pixels.
    pub pointer_pos: crate::canvas::Point,
    /// Whether the primary mouse button is held.
    pub mouse_down: bool,
    /// Whether the primary mouse button was pressed this frame.
    pub mouse_pressed: bool,
    /// Whether the primary mouse button was released this frame.
    pub mouse_released: bool,
    /// Whether the primary mouse button was double-clicked this frame.
    pub mouse_double_clicked: bool,
    /// Whether the secondary mouse button is held.
    pub mouse_secondary_down: bool,
    /// Whether the secondary mouse button was pressed this frame.
    pub mouse_secondary_pressed: bool,
    /// Whether the secondary mouse button was released this frame.
    pub mouse_secondary_released: bool,
    /// Whether either Shift key is currently held.
    pub shift_down: bool,
    /// Whether either Alt key is currently held.
    pub alt_down: bool,
    /// Scroll delta for this frame (positive = up).
    pub wheel_delta: f32,
    /// Key press captured this frame (ASCII).
    pub key_pressed: Option<char>,
    /// Files dropped onto the window this frame.
    pub dropped_files: Vec<PathBuf>,
}

/// Handle to an open GUI window.
#[derive(Clone, Debug)]
pub struct HostWindow {
    pub(super) parent: Option<RawWindowHandle>,
    pub(super) parent_hwnd: Option<isize>,
    pub(super) handle: Option<WindowHandle>,
    pub(super) device_cache: Arc<Mutex<Option<Arc<RendererDevice>>>>,
    pub(super) resize_request: Arc<AtomicU64>,
    pub(super) last_size: Arc<AtomicU64>,
    pub(super) aspect_ratio: Arc<AtomicU32>,
}

impl Default for HostWindow {
    fn default() -> Self {
        Self {
            parent: None,
            parent_hwnd: None,
            handle: None,
            device_cache: Arc::new(Mutex::new(None)),
            resize_request: Arc::new(AtomicU64::new(0)),
            last_size: Arc::new(AtomicU64::new(0)),
            aspect_ratio: Arc::new(AtomicU32::new(0)),
        }
    }
}
