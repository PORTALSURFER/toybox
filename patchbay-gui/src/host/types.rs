//! Host-facing input and window state types.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64};
use std::sync::{Arc, Mutex};

use raw_window_handle::RawWindowHandle;

use crate::canvas::Size;
#[cfg(feature = "frame-capture")]
use crate::frame_capture::FrameCaptureState;
use crate::renderer::RendererDevice;
use crate::win32::WindowHandle;

/// Input snapshot delivered to UI widgets for a single frame.
#[derive(Clone, Debug)]
pub struct InputState {
    /// Current logical window size in pixels.
    pub window_size: Size,
    /// Current pointer position in pixels.
    pub pointer_pos: crate::canvas::Point,
    /// Whether the pointer is currently inside the client window.
    ///
    /// This remains `true` while dragging with mouse capture so controls can
    /// continue receiving drag updates after leaving window bounds.
    pub pointer_in_window: bool,
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
    /// Whether either platform command key is currently held.
    pub command_down: bool,
    /// Scroll delta for this frame (positive = up).
    pub wheel_delta: f32,
    /// Key press captured this frame.
    ///
    /// Printable input arrives as Unicode scalar values. Control keys are
    /// normalized to control characters such as backspace (`\u{8}`), return
    /// (`\r`), and escape (`\u{1b}`).
    pub key_pressed: Option<char>,
    /// Canonicalized shortcut-style keys currently held down.
    ///
    /// Entries are stored as lowercase ASCII when possible so callers can
    /// query held state case-insensitively via [`Self::shortcut_key_down`].
    pub held_shortcut_keys: Vec<char>,
    /// Files dropped onto the window this frame.
    pub dropped_files: Vec<PathBuf>,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            window_size: Size::default(),
            pointer_pos: crate::canvas::Point::default(),
            pointer_in_window: true,
            mouse_down: false,
            mouse_pressed: false,
            mouse_released: false,
            mouse_double_clicked: false,
            mouse_secondary_down: false,
            mouse_secondary_pressed: false,
            mouse_secondary_released: false,
            shift_down: false,
            alt_down: false,
            command_down: false,
            wheel_delta: 0.0,
            key_pressed: None,
            held_shortcut_keys: Vec::new(),
            dropped_files: Vec::new(),
        }
    }
}

impl InputState {
    /// Return `true` when `key` is currently held in shortcut-normalized form.
    pub fn shortcut_key_down(&self, key: char) -> bool {
        let canonical = canonical_shortcut_char(key);
        self.held_shortcut_keys.contains(&canonical)
    }
}

/// Keyboard modifier flags used for shortcut matching.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShortcutModifiers {
    /// Whether Shift must be held.
    pub shift: bool,
    /// Whether Alt must be held.
    pub alt: bool,
    /// Whether Ctrl must be held.
    pub ctrl: bool,
}

impl ShortcutModifiers {
    /// Build one explicit modifier set.
    pub const fn new(shift: bool, alt: bool, ctrl: bool) -> Self {
        Self { shift, alt, ctrl }
    }

    /// Return packed bit flags suitable for Win32 message payloads.
    pub const fn to_bits(self) -> usize {
        (self.shift as usize) | ((self.alt as usize) << 1) | ((self.ctrl as usize) << 2)
    }

    /// Decode packed bit flags from Win32 message payloads.
    pub const fn from_bits(bits: usize) -> Self {
        Self {
            shift: (bits & 0b001) != 0,
            alt: (bits & 0b010) != 0,
            ctrl: (bits & 0b100) != 0,
        }
    }
}

/// One plugin-registered keyboard shortcut.
///
/// The shortcut dispatches a synthetic `UiAction::ButtonPressed` with
/// `action_key` when `key` and `modifiers` match exactly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShortcutBinding {
    /// Key of the button-style action dispatched into the reducer.
    pub action_key: String,
    /// Trigger character for this shortcut.
    pub key: char,
    /// Required modifier flags.
    pub modifiers: ShortcutModifiers,
}

impl ShortcutBinding {
    /// Create one shortcut binding.
    pub fn new(action_key: impl Into<String>, key: char, modifiers: ShortcutModifiers) -> Self {
        Self {
            action_key: action_key.into(),
            key: canonical_shortcut_char(key),
            modifiers,
        }
    }

    /// Return `true` when this shortcut matches the provided input.
    pub fn matches(&self, key: char, modifiers: ShortcutModifiers) -> bool {
        canonical_shortcut_char(key) == self.key && modifiers == self.modifiers
    }
}

/// Normalize shortcut characters for deterministic matching.
pub(crate) fn canonical_shortcut_char(key: char) -> char {
    key.to_ascii_lowercase()
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
    pub(super) active_text_edit: Arc<AtomicBool>,
    pub(super) shortcut_bindings: Arc<Mutex<Vec<ShortcutBinding>>>,
    #[cfg(feature = "frame-capture")]
    pub(super) frame_capture: Arc<FrameCaptureState>,
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
            active_text_edit: Arc::new(AtomicBool::new(false)),
            shortcut_bindings: Arc::new(Mutex::new(Vec::new())),
            #[cfg(feature = "frame-capture")]
            frame_capture: Arc::new(FrameCaptureState::default()),
        }
    }
}
