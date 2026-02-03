//! Host window management and input plumbing for Patchbay GUI.

use crate::canvas::Size;
use crate::ui::{Layout, Theme, Ui, UiState};
use crate::win32::{spawn_window_thread, WindowHandle};
use raw_window_handle::RawWindowHandle;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;

/// Input snapshot delivered to UI widgets for a single frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct InputState {
    /// Current pointer position in pixels.
    pub pointer_pos: crate::canvas::Point,
    /// Whether the primary mouse button is held.
    pub mouse_down: bool,
    /// Whether the primary mouse button was pressed this frame.
    pub mouse_pressed: bool,
    /// Whether the primary mouse button was released this frame.
    pub mouse_released: bool,
    /// Scroll delta for this frame (positive = up).
    pub wheel_delta: f32,
}

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
}

/// Handle to an open GUI window.
#[derive(Clone, Debug)]
pub struct HostWindow {
    parent: Option<RawWindowHandle>,
    handle: Option<WindowHandle>,
    resize_request: Arc<AtomicU64>,
    last_size: Arc<AtomicU64>,
    aspect_ratio: Arc<AtomicU32>,
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

impl HostWindow {
    /// Assign the raw parent handle supplied by the CLAP host.
    pub fn set_parent(&mut self, parent: RawWindowHandle) {
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

    /// Set a desired aspect ratio for host-driven resizing.
    pub fn set_aspect_ratio(&mut self, ratio: Option<f32>) {
        let bits = ratio.filter(|value| value.is_finite() && *value > 0.0).unwrap_or(0.0);
        self.aspect_ratio.store(bits.to_bits(), Ordering::Release);
    }

    /// Open a parented Patchbay GUI window.
    ///
    /// The caller supplies initial state plus callbacks for initialization and
    /// per-frame rendering.
    pub fn open_parented<State, Init, Frame>(
        &mut self,
        title: String,
        size: (u32, u32),
        state: State,
        mut on_init: Init,
        mut on_frame: Frame,
    ) -> Result<(), GuiError>
    where
        Init: FnMut(&mut Ui<'_>, &mut State) + Send + 'static,
        Frame: FnMut(&mut Ui<'_>, &mut State) + Send + 'static,
        State: Send + 'static,
    {
        let parent = self.parent.ok_or(GuiError::NoParent)?;
        let (parent_hwnd, parent_hinstance) = match parent {
            RawWindowHandle::Win32(handle) => {
                (handle.hwnd as isize, handle.hinstance as isize)
            }
            _ => return Err(GuiError::UnsupportedHandle),
        };

        let resize_request = self.resize_request.clone();
        let last_size = self.last_size.clone();
        let aspect_ratio = self.aspect_ratio.clone();

        let theme = Theme::default();
        let layout = Layout::default();
        let ui_state = UiState::default();

        let handle = spawn_window_thread(
            parent_hwnd,
            parent_hinstance,
            title,
            Size {
                width: size.0,
                height: size.1,
            },
            state,
            move |ui, state| {
                on_init(ui, state);
            },
            move |ui, state| {
                on_frame(ui, state);
            },
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
