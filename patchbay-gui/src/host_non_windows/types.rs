//! Host-facing input and placeholder handle/state types for non-Windows.

use raw_window_handle::RawWindowHandle;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64};

use crate::canvas::Size;

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
    /// Key press captured this frame.
    ///
    /// Printable input arrives as Unicode scalar values. Control keys are
    /// normalized to control characters such as backspace (`\u{8}`), return
    /// (`\r`), and escape (`\u{1b}`).
    pub key_pressed: Option<char>,
    /// Files dropped onto the window this frame.
    pub dropped_files: Vec<PathBuf>,
}

/// Opaque non-Windows placeholder for the native window handle.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WindowHandle;

impl WindowHandle {
    /// Return false because no native window is created on this platform.
    pub fn is_valid(&self) -> bool {
        false
    }

    /// Return false because there is no native parent relationship.
    pub fn parent_matches(&self, _parent: isize) -> bool {
        false
    }

    /// No-op on unsupported platforms.
    pub fn destroy(&self) {}

    /// No-op on unsupported platforms.
    pub fn set_visible(&self, _visible: bool) {}
}

/// Handle to an open GUI window.
#[derive(Clone, Debug)]
pub struct HostWindow {
    /// Last parent handle provided by the host.
    pub(super) parent: Option<RawWindowHandle>,
    /// Placeholder native window handle (always `None` on non-Windows).
    pub(super) handle: Option<WindowHandle>,
    /// Packed width/height resize request shared with callers.
    pub(super) resize_request: Arc<AtomicU64>,
    /// Packed width/height last observed size.
    pub(super) last_size: Arc<AtomicU64>,
    /// Packed aspect ratio bits requested by callers.
    pub(super) aspect_ratio: Arc<AtomicU32>,
}

impl Default for HostWindow {
    fn default() -> Self {
        Self {
            parent: None,
            handle: None,
            resize_request: Arc::new(AtomicU64::new(0)),
            last_size: Arc::new(AtomicU64::new(0)),
            aspect_ratio: Arc::new(AtomicU32::new(0)),
        }
    }
}
