//! HostWindow behavior for non-Windows stubs.

use std::sync::atomic::Ordering;
#[cfg(feature = "frame-capture")]
use std::time::Duration;

use raw_window_handle::RawWindowHandle;

use crate::canvas::Size;
use crate::declarative::{UiAction, UiSpec};
#[cfg(feature = "frame-capture")]
use crate::frame_capture::CapturedWindowFrame;

use super::errors::GuiError;
use super::requests::{OpenParentedCallbacks, OpenParentedRequest};
use super::types::{HostWindow, InputState, ShortcutBinding, ShortcutModifiers, WindowHandle};
use super::{pack_size, unpack_size};

impl HostWindow {
    /// Return `true` when any editable text box is active this frame.
    pub fn text_edit_active(&self) -> bool {
        self.active_text_edit.load(Ordering::Acquire)
    }

    /// Replace focused-window shortcut bindings.
    pub fn set_shortcuts(&self, shortcuts: Vec<ShortcutBinding>) {
        if let Ok(mut current) = self.shortcut_bindings.lock() {
            *current = shortcuts;
        }
    }

    /// Resolve one registered shortcut action key from input.
    pub fn shortcut_action_for_input(
        &self,
        ch: char,
        modifiers: ShortcutModifiers,
    ) -> Option<String> {
        let Ok(current) = self.shortcut_bindings.lock() else {
            return None;
        };
        current
            .iter()
            .find(|binding| binding.matches(ch, modifiers))
            .map(|binding| binding.action_key.clone())
    }

    /// Assign the raw parent handle supplied by the CLAP host.
    pub fn set_parent(&mut self, parent: RawWindowHandle) {
        self.parent = Some(parent);
    }

    /// Return the most recent logical size reported by the window.
    pub fn last_size(&self) -> Option<(u32, u32)> {
        unpack_size(self.last_size.load(Ordering::Acquire))
    }

    /// Request a logical resize from the GUI thread.
    ///
    /// The requested size is recorded immediately so callers can observe host
    /// size negotiation progress even before the resize request is consumed.
    pub fn request_resize(&self, width: u32, height: u32) {
        let size = Size {
            width: width.max(1),
            height: height.max(1),
        };
        self.last_size
            .store(pack_size(size.width, size.height), Ordering::Release);
        self.resize_request
            .store(pack_size(size.width, size.height), Ordering::Release);
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
    /// This always returns [`GuiError::UnsupportedHandle`] on non-Windows
    /// platforms. The method exists so clients compile cross-platform.
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
    /// This always returns [`GuiError::UnsupportedHandle`] on non-Windows
    /// platforms. The method exists so clients compile cross-platform.
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
            title: _title,
            size,
            callbacks: _callbacks,
            mode: _mode,
        } = request;
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

    /// Capture the next rendered hosted-window frame.
    ///
    /// Non-Windows builds do not host native windows and therefore do not
    /// support live frame capture.
    #[cfg(feature = "frame-capture")]
    pub fn capture_next_frame(&self, _timeout: Duration) -> Result<CapturedWindowFrame, GuiError> {
        Err(GuiError::UnsupportedHandle)
    }

    /// Stub text-input injection for non-Windows builds.
    pub fn post_text_char(&self, _ch: char) -> bool {
        false
    }

    /// Stub injected text-input path for non-Windows builds.
    pub fn post_injected_text_char(&self, _ch: char, _modifiers: ShortcutModifiers) -> bool {
        false
    }

    /// Stub injected key-up path for non-Windows builds.
    pub fn post_injected_key_up(&self, _ch: char, _modifiers: ShortcutModifiers) -> bool {
        false
    }
}
