//! Host window management and input plumbing for Patchbay GUI.

use crate::canvas::Size;
use crate::declarative::{UiAction, UiSpec};
use crate::renderer::RendererDevice;
use crate::ui::{Layout, Theme, UiState};
use crate::win32::{
    SpawnCallbacks, SpawnSharedState, SpawnUiConfig, SpawnWindowConfig, SpawnWindowRequest,
    WindowHandle, spawn_window_thread,
};
use raw_window_handle::RawWindowHandle;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

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

/// Policy for handling repeated `open_parented` calls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenParentedMode {
    /// Reuse an existing window if it matches the parent handle.
    ReuseIfOpen,
    /// Always recreate the window and UI state.
    Recreate,
}

/// Request payload for opening a parented GUI window.
///
/// This bundles state and callbacks so call sites do not need wide function
/// signatures when opening Patchbay windows.
pub struct OpenParentedRequest<State, Init, Build, Reduce> {
    /// Window title shown by the host.
    pub title: String,
    /// Initial logical window size.
    pub size: Size,
    /// Initial user-provided UI state.
    pub state: State,
    /// One-time state initialization callback.
    pub on_init: Init,
    /// Per-frame declarative tree builder.
    pub build: Build,
    /// UI action reducer callback.
    pub reduce: Reduce,
    /// Reuse behavior for repeated open calls.
    pub mode: OpenParentedMode,
}

impl<State, Init, Build, Reduce> OpenParentedRequest<State, Init, Build, Reduce> {
    /// Build an open request using [`OpenParentedMode::ReuseIfOpen`].
    pub fn new(
        title: String,
        size: Size,
        state: State,
        on_init: Init,
        build: Build,
        reduce: Reduce,
    ) -> Self {
        Self {
            title,
            size,
            state,
            on_init,
            build,
            reduce,
            mode: OpenParentedMode::ReuseIfOpen,
        }
    }

    /// Override the default reuse mode.
    pub fn with_mode(mut self, mode: OpenParentedMode) -> Self {
        self.mode = mode;
        self
    }
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
    /// The caller supplies initial state plus callbacks for initialization,
    /// per-frame declarative spec building, and action reduction.
    ///
    /// The `size` argument is used as the initial window size.
    pub fn open_parented<State, Init, Build, Reduce>(
        &mut self,
        title: String,
        size: (u32, u32),
        state: State,
        on_init: Init,
        build: Build,
        reduce: Reduce,
    ) -> Result<(), GuiError>
    where
        Init: FnMut(&mut State) + Send + 'static,
        Build: FnMut(&InputState, &State) -> UiSpec + Send + 'static,
        Reduce: FnMut(&mut State, UiAction) + Send + 'static,
        State: Send + 'static,
    {
        self.open_parented_with(OpenParentedRequest::new(
            title,
            Size {
                width: size.0.max(1),
                height: size.1.max(1),
            },
            state,
            on_init,
            build,
            reduce,
        ))
    }

    /// Open a parented Patchbay GUI window with explicit reuse policy.
    ///
    /// When `mode` is [`OpenParentedMode::ReuseIfOpen`], a matching window is
    /// shown, and the new state/callbacks are ignored. When `mode` is
    /// [`OpenParentedMode::Recreate`], any existing window is destroyed and a
    /// new one is created with the provided state. The `size` argument is used
    /// as the initial window size.
    pub fn open_parented_with<State, Init, Build, Reduce>(
        &mut self,
        request: OpenParentedRequest<State, Init, Build, Reduce>,
    ) -> Result<(), GuiError>
    where
        Init: FnMut(&mut State) + Send + 'static,
        Build: FnMut(&InputState, &State) -> UiSpec + Send + 'static,
        Reduce: FnMut(&mut State, UiAction) + Send + 'static,
        State: Send + 'static,
    {
        let OpenParentedRequest {
            title,
            size,
            state,
            on_init,
            build,
            reduce,
            mode,
        } = request;
        let size = Self::normalize_open_size(size);
        self.record_last_size(size);
        let parent = self.resolve_parent_window()?;
        if self.reuse_or_destroy_existing_window(parent.hwnd, mode, size) {
            return Ok(());
        }
        let callbacks = SpawnCallbacks {
            state,
            on_init,
            build,
            reduce,
        };
        self.spawn_parented_window(parent, title, size, callbacks)
    }

    /// Access the OS-level window handle if one exists.
    pub fn handle(&self) -> Option<WindowHandle> {
        self.handle.clone()
    }
}

/// Raw Win32 parent handles extracted from a CLAP host parent handle.
struct ParentWindowHandles {
    /// Parent window handle value.
    hwnd: isize,
    /// Parent module instance handle value.
    hinstance: isize,
}

impl HostWindow {
    /// Clamp requested open size to at least one pixel per dimension.
    fn normalize_open_size(size: Size) -> Size {
        Size {
            width: size.width.max(1),
            height: size.height.max(1),
        }
    }

    /// Persist the latest requested logical size for host polling.
    fn record_last_size(&self, size: Size) {
        self.last_size
            .store(pack_size(size.width, size.height), Ordering::Release);
    }

    /// Resolve and validate Win32 parent handles from the configured raw parent.
    fn resolve_parent_window(&self) -> Result<ParentWindowHandles, GuiError> {
        let parent = self.parent.ok_or(GuiError::NoParent)?;
        match parent {
            RawWindowHandle::Win32(handle) => Ok(ParentWindowHandles {
                hwnd: handle.hwnd as isize,
                hinstance: handle.hinstance as isize,
            }),
            _ => Err(GuiError::UnsupportedHandle),
        }
    }

    /// Reuse an existing matching window or destroy stale window state.
    ///
    /// Returns `true` when a matching window was reused and no new spawn is needed.
    fn reuse_or_destroy_existing_window(
        &mut self,
        parent_hwnd: isize,
        mode: OpenParentedMode,
        size: Size,
    ) -> bool {
        if let Some(handle) = &self.handle {
            if handle.is_valid()
                && handle.parent_matches(parent_hwnd)
                && mode == OpenParentedMode::ReuseIfOpen
            {
                self.resize_request
                    .store(pack_size(size.width, size.height), Ordering::Release);
                self.show();
                return true;
            }
            handle.destroy();
            self.handle = None;
        }
        false
    }

    /// Build shared synchronization state for a newly spawned window.
    fn build_spawn_shared_state(&self) -> SpawnSharedState {
        SpawnSharedState {
            device_cache: self.device_cache.clone(),
            resize_request: self.resize_request.clone(),
            last_size: self.last_size.clone(),
            aspect_ratio: self.aspect_ratio.clone(),
        }
    }

    /// Return default UI configuration used during Win32 window creation.
    fn default_spawn_ui_config() -> SpawnUiConfig {
        SpawnUiConfig {
            ui_state: UiState::default(),
            layout: Layout::default(),
            theme: Theme::default(),
        }
    }

    /// Spawn a new parented window and store its handle.
    fn spawn_parented_window<State, Init, Build, Reduce>(
        &mut self,
        parent: ParentWindowHandles,
        title: String,
        size: Size,
        callbacks: SpawnCallbacks<State, Init, Build, Reduce>,
    ) -> Result<(), GuiError>
    where
        Init: FnMut(&mut State) + Send + 'static,
        Build: FnMut(&InputState, &State) -> UiSpec + Send + 'static,
        Reduce: FnMut(&mut State, UiAction) + Send + 'static,
        State: Send + 'static,
    {
        let request = SpawnWindowRequest {
            window: SpawnWindowConfig {
                parent_hwnd: parent.hwnd,
                parent_hinstance: parent.hinstance,
                title,
                size,
            },
            callbacks,
            shared: self.build_spawn_shared_state(),
            ui: Self::default_spawn_ui_config(),
        };
        let handle = spawn_window_thread(request)?;
        self.handle = Some(handle);
        Ok(())
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
