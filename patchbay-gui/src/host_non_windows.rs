//! Non-Windows host window stubs for Patchbay GUI.
//!
//! This module keeps the crate buildable on platforms where the native window
//! backend is unavailable. GUI opening APIs return [`GuiError::UnsupportedHandle`].

use crate::canvas::Size;
use crate::declarative::UiSpec;
use raw_window_handle::RawWindowHandle;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

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
    parent: Option<RawWindowHandle>,
    /// Placeholder native window handle (always `None` on non-Windows).
    handle: Option<WindowHandle>,
    /// Packed width/height resize request shared with callers.
    resize_request: Arc<AtomicU64>,
    /// Packed width/height last observed size.
    last_size: Arc<AtomicU64>,
    /// Packed aspect ratio bits requested by callers.
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
    /// This always returns [`GuiError::UnsupportedHandle`] on non-Windows
    /// platforms. The method exists so clients compile cross-platform.
    pub fn open_parented<State, Init, Frame>(
        &mut self,
        title: String,
        size: (u32, u32),
        state: State,
        on_init: Init,
        on_frame: Frame,
    ) -> Result<(), GuiError>
    where
        Init: FnMut(&mut State) + Send + 'static,
        Frame: FnMut(&InputState, &mut State) -> UiSpec<'static, State> + Send + 'static,
        State: Send + 'static,
    {
        self.open_parented_with(
            title,
            Size {
                width: size.0.max(1),
                height: size.1.max(1),
            },
            state,
            on_init,
            on_frame,
            OpenParentedMode::ReuseIfOpen,
        )
    }

    /// Open a parented Patchbay GUI window with explicit reuse policy.
    ///
    /// This always returns [`GuiError::UnsupportedHandle`] on non-Windows
    /// platforms. The method exists so clients compile cross-platform.
    pub fn open_parented_with<State, Init, Frame>(
        &mut self,
        _title: String,
        size: Size,
        _state: State,
        _on_init: Init,
        _on_frame: Frame,
        _mode: OpenParentedMode,
    ) -> Result<(), GuiError>
    where
        Init: FnMut(&mut State) + Send + 'static,
        Frame: FnMut(&InputState, &mut State) -> UiSpec<'static, State> + Send + 'static,
        State: Send + 'static,
    {
        if self.parent.is_none() {
            return Err(GuiError::NoParent);
        }
        self.last_size
            .store(pack_size(size.width, size.height), Ordering::Release);
        Err(GuiError::UnsupportedHandle)
    }

    /// Access the OS-level window handle if one exists.
    pub fn handle(&self) -> Option<WindowHandle> {
        self.handle.clone()
    }
}

/// Pack a size into a compact atomic payload.
fn pack_size(width: u32, height: u32) -> u64 {
    ((width as u64) << 32) | (height as u64)
}

/// Decode an atomic size payload.
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

    #[test]
    fn open_parented_reports_unsupported_after_parent() {
        let mut host = HostWindow::default();
        host.set_parent(raw_window_handle::RawWindowHandle::AppKit(
            raw_window_handle::AppKitWindowHandle::empty(),
        ));
        let result = host.open_parented(
            "Stub".into(),
            (320, 200),
            (),
            |_state| {},
            |_input, _state| UiSpec {
                root: crate::declarative::RootFrameSpec {
                    key: "root".into(),
                    title: None,
                    padding: 0,
                    content: Box::new(crate::declarative::Node::Spacer(
                        crate::declarative::SpacerSpec {
                            size: Size {
                                width: 1,
                                height: 1,
                            },
                        },
                    )),
                },
            },
        );
        assert!(matches!(result, Err(GuiError::UnsupportedHandle)));
    }
}
