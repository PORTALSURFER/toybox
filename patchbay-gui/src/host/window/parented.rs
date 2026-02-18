//! Parented-window open and reuse flow for [`HostWindow`].

use raw_window_handle::RawWindowHandle;

use crate::canvas::Size;
use crate::declarative::{UiAction, UiSpec};
use crate::host::errors::GuiError;
use crate::host::pack_size;
use crate::host::requests::{OpenParentedCallbacks, OpenParentedMode, OpenParentedRequest};
use crate::host::types::{HostWindow, InputState};
use crate::win32::SpawnCallbacks;

use super::ParentWindowHandles;

impl HostWindow {
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
                self.resize_request.store(
                    pack_size(size.width, size.height),
                    std::sync::atomic::Ordering::Release,
                );
                self.show();
                return true;
            }
            handle.destroy();
            self.handle = None;
        }
        false
    }
}
