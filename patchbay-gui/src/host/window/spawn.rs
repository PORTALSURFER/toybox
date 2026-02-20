//! Native window spawn helpers for [`HostWindow`].

use crate::host::errors::GuiError;
use crate::host::types::HostWindow;
use crate::ui::{Layout, Theme, UiState};
use crate::win32::{
    SpawnCallbacks, SpawnSharedState, SpawnUiConfig, SpawnWindowConfig, SpawnWindowRequest,
    spawn_window_thread,
};

use super::ParentWindowHandles;

impl HostWindow {
    /// Build shared synchronization state for a newly spawned window.
    fn build_spawn_shared_state(&self) -> SpawnSharedState {
        SpawnSharedState {
            device_cache: self.device_cache.clone(),
            resize_request: self.resize_request.clone(),
            last_size: self.last_size.clone(),
            aspect_ratio: self.aspect_ratio.clone(),
            active_text_edit: self.active_text_edit.clone(),
            shortcut_bindings: self.shortcut_bindings.clone(),
            #[cfg(feature = "frame-capture")]
            frame_capture: self.frame_capture.clone(),
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
    pub(super) fn spawn_parented_window<State, Init, Build, Reduce>(
        &mut self,
        parent: ParentWindowHandles,
        title: String,
        size: crate::canvas::Size,
        callbacks: SpawnCallbacks<State, Init, Build, Reduce>,
    ) -> Result<(), GuiError>
    where
        Init: FnMut(&mut State) + Send + 'static,
        Build: FnMut(&crate::host::types::InputState, &State) -> crate::declarative::UiSpec
            + Send
            + 'static,
        Reduce: FnMut(&mut State, crate::declarative::UiAction) + Send + 'static,
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
