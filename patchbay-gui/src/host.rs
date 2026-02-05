//! Host window management and input plumbing for Patchbay GUI.

use crate::canvas::Size;
use crate::declarative::UiSpec;
use crate::renderer::RendererDevice;
use crate::ui::{Layout, Theme, UiState};
use crate::win32::{spawn_window_thread, WindowHandle};
use raw_window_handle::RawWindowHandle;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// Input snapshot delivered to UI widgets for a single frame.
#[derive(Clone, Debug, Default)]
pub struct InputState {
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

/// Policy for handling repeated `open_parented` calls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenParentedMode {
    /// Reuse an existing window if it matches the parent handle.
    ReuseIfOpen,
    /// Always recreate the window and UI state.
    Recreate,
}

const DEFAULT_WINDOW_SIZE: Size = Size {
    width: 640,
    height: 360,
};

/// Errors returned by the Patchbay GUI system.
#[derive(thiserror::Error, Debug)]
pub enum GuiError {
    /// The host did not provide a parent window.
    #[error("no parent window was provided")]
    NoParent,
    /// The raw window handle is not supported on this platform.
    #[error("unsupported window handle for this platform")]
    UnsupportedHandle,
    /// Failed to create the native window.
    #[error("failed to create Win32 window")]
    WindowCreateFailed,
    /// Failed to locate a compatible GPU adapter.
    #[error("no compatible GPU adapter found")]
    AdapterNotFound,
    /// Surface creation failed.
    #[error("failed to create wgpu surface")]
    Surface(#[source] wgpu::CreateSurfaceError),
    /// Device creation failed.
    #[error("failed to create wgpu device")]
    Device(#[source] wgpu::RequestDeviceError),
    /// Surface has no supported formats.
    #[error("wgpu surface reports no supported formats")]
    SurfaceFormat,
    /// Failed to acquire the next swapchain frame.
    #[error("failed to acquire next swapchain texture")]
    SurfaceAcquire(#[source] wgpu::SurfaceError),
    /// GUI thread failed to start.
    #[error("failed to spawn GUI thread")]
    ThreadSpawn,
    /// Device cache mutex was poisoned.
    #[error("renderer device cache mutex poisoned")]
    DeviceCachePoison,
}

/// Handle to an open GUI window.
#[derive(Clone, Debug)]
pub struct HostWindow {
    parent: Option<RawWindowHandle>,
    parent_hwnd: Option<isize>,
    handle: Option<WindowHandle>,
    device_cache: Arc<Mutex<Option<Arc<RendererDevice>>>>,
    resize_request: Arc<AtomicU64>,
    last_size: Arc<AtomicU64>,
    aspect_ratio: Arc<AtomicU32>,
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

impl HostWindow {
    /// Assign the raw parent handle supplied by the CLAP host.
    pub fn set_parent(&mut self, parent: RawWindowHandle) {
        let new_parent_hwnd = match parent {
            RawWindowHandle::Win32(handle) => Some(handle.hwnd as isize),
            _ => None,
        };
        if self.parent_hwnd != new_parent_hwnd {
            if let Some(handle) = &self.handle {
                handle.destroy();
            }
            self.handle = None;
            self.parent_hwnd = new_parent_hwnd;
        }
        self.parent = Some(parent);
    }

    /// Return the most recent logical size reported by the window.
    pub fn last_size(&self) -> Option<(u32, u32)> {
        unpack_size(self.last_size.load(Ordering::Acquire))
    }

    /// Request a logical resize from the GUI thread.
    pub fn request_resize(&self, width: u32, height: u32) {
        self.resize_request
            .store(pack_size(width, height), Ordering::Release);
    }

    /// Return true if a native window has been created.
    pub fn is_open(&self) -> bool {
        self.handle.is_some()
    }

    /// Show the native window if it exists.
    pub fn show(&self) {
        if let Some(handle) = &self.handle {
            handle.set_visible(true);
        }
    }

    /// Hide the native window if it exists.
    pub fn hide(&self) {
        if let Some(handle) = &self.handle {
            handle.set_visible(false);
        }
    }

    /// Set a desired aspect ratio for host-driven resizing.
    pub fn set_aspect_ratio(&mut self, ratio: Option<f32>) {
        let bits = ratio
            .filter(|value| value.is_finite() && *value > 0.0)
            .unwrap_or(0.0);
        self.aspect_ratio.store(bits.to_bits(), Ordering::Release);
    }

    /// Open a parented Patchbay GUI window.
    ///
    /// The caller supplies initial state plus callbacks for initialization and
    /// per-frame declarative rendering. The `size` argument is ignored and the
    /// window uses the internal default size before auto-resizing to the root
    /// frame measurement. If a matching window is already open, this reuses it
    /// and ignores the new state.
    pub fn open_parented<State, Init, Frame>(
        &mut self,
        title: String,
        _size: (u32, u32),
        state: State,
        mut on_init: Init,
        mut on_frame: Frame,
    ) -> Result<(), GuiError>
    where
        Init: FnMut(&mut State) + Send + 'static,
        Frame: FnMut(&InputState, &mut State) -> UiSpec<'static, State> + Send + 'static,
        State: Send + 'static,
    {
        self.open_parented_with(
            title,
            DEFAULT_WINDOW_SIZE,
            state,
            on_init,
            on_frame,
            OpenParentedMode::ReuseIfOpen,
        )
    }

    /// Open a parented Patchbay GUI window with explicit reuse policy.
    ///
    /// When `mode` is [`OpenParentedMode::ReuseIfOpen`], a matching window is
    /// shown, and the new state/callbacks are ignored. When `mode` is
    /// [`OpenParentedMode::Recreate`], any existing window is destroyed and a
    /// new one is created with the provided state. The `size` argument is used
    /// as the initial window size; the declarative root frame will still drive
    /// auto-resizing.
    pub fn open_parented_with<State, Init, Frame>(
        &mut self,
        title: String,
        size: Size,
        state: State,
        mut on_init: Init,
        mut on_frame: Frame,
        mode: OpenParentedMode,
    ) -> Result<(), GuiError>
    where
        Init: FnMut(&mut State) + Send + 'static,
        Frame: FnMut(&InputState, &mut State) -> UiSpec<'static, State> + Send + 'static,
        State: Send + 'static,
    {
        let parent = self.parent.ok_or(GuiError::NoParent)?;
        let (parent_hwnd, parent_hinstance) = match parent {
            RawWindowHandle::Win32(handle) => (handle.hwnd as isize, handle.hinstance as isize),
            _ => return Err(GuiError::UnsupportedHandle),
        };
        if let Some(handle) = &self.handle {
            if handle.is_valid() && handle.parent_matches(parent_hwnd) {
                if mode == OpenParentedMode::ReuseIfOpen {
                    self.show();
                    return Ok(());
                }
            }
            handle.destroy();
            self.handle = None;
        }

        let resize_request = self.resize_request.clone();
        let last_size = self.last_size.clone();
        let aspect_ratio = self.aspect_ratio.clone();
        let device_cache = self.device_cache.clone();

        let theme = Theme::default();
        let layout = Layout::default();
        let ui_state = UiState::default();

        let handle = spawn_window_thread(
            parent_hwnd,
            parent_hinstance,
            title,
            size,
            state,
            move |state| {
                on_init(state);
            },
            move |input, state| on_frame(input, state),
            device_cache,
            resize_request,
            last_size,
            aspect_ratio,
            ui_state,
            layout,
            theme,
        )?;

        self.handle = Some(handle);
        Ok(())
    }

    /// Access the OS-level window handle if one exists.
    pub fn handle(&self) -> Option<WindowHandle> {
        self.handle.clone()
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_unpack_roundtrip() {
        let packed = pack_size(640, 480);
        assert_eq!(unpack_size(packed), Some((640, 480)));
    }
}
