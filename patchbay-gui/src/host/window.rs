//! HostWindow behavior and Win32 parented-window spawn flow.

use std::sync::atomic::Ordering;

use raw_window_handle::RawWindowHandle;

use crate::canvas::Size;
use crate::declarative::{UiAction, UiSpec};
use crate::ui::{Layout, Theme, UiState};
use crate::win32::{
    SpawnCallbacks, SpawnSharedState, SpawnUiConfig, SpawnWindowConfig, SpawnWindowRequest,
    spawn_window_thread,
};

use super::errors::GuiError;
use super::requests::{OpenParentedCallbacks, OpenParentedMode, OpenParentedRequest};
use super::types::{HostWindow, InputState};
use super::{pack_size, unpack_size};

/// Raw Win32 parent handles extracted from a CLAP host parent handle.
struct ParentWindowHandles {
    /// Parent window handle value.
    hwnd: isize,
    /// Parent module instance handle value.
    hinstance: isize,
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
    /// # Deprecated
    ///
    /// Prefer [`Self::open_parented_with`] with an [`OpenParentedRequest`].
    /// This adapter preserves compatibility while keeping wide callback wiring in one
    /// request value.
    #[deprecated(
        since = "0.1.0",
        note = "Use open_parented_with(OpenParentedRequest::with_callbacks(...))"
    )]
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
        self.open_parented_with(OpenParentedRequest::with_callbacks(
            title,
            Size {
                width: size.0.max(1),
                height: size.1.max(1),
            },
            OpenParentedCallbacks::new(state, on_init, build, reduce),
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
            callbacks,
            mode,
        } = request;
        let size = Self::normalize_open_size(size);
        self.record_last_size(size);
        let parent = self.resolve_parent_window()?;
        if self.reuse_or_destroy_existing_window(parent.hwnd, mode, size) {
            return Ok(());
        }
        let OpenParentedCallbacks {
            state,
            on_init,
            build,
            reduce,
        } = callbacks;
        let callbacks = SpawnCallbacks {
            state,
            on_init,
            build,
            reduce,
        };
        self.spawn_parented_window(parent, title, size, callbacks)
    }

    /// Access the OS-level window handle if one exists.
    pub fn handle(&self) -> Option<crate::win32::WindowHandle> {
        self.handle.clone()
    }

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
