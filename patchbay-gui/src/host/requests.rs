//! Request payloads and policies for parented-window open operations.

use crate::canvas::Size;

/// Policy for handling repeated `open_parented` calls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenParentedMode {
    /// Reuse an existing window if it matches the parent handle.
    ReuseIfOpen,
    /// Always recreate the window and UI state.
    Recreate,
}

/// Grouped callbacks and state for a parented-window open request.
pub struct OpenParentedCallbacks<State, Init, Build, Reduce> {
    /// Initial user-provided UI state.
    pub state: State,
    /// One-time state initialization callback.
    pub on_init: Init,
    /// Per-frame declarative tree builder.
    pub build: Build,
    /// UI action reducer callback.
    pub reduce: Reduce,
}

impl<State, Init, Build, Reduce> OpenParentedCallbacks<State, Init, Build, Reduce> {
    /// Build a grouped callback payload for parented-window open.
    pub fn new(state: State, on_init: Init, build: Build, reduce: Reduce) -> Self {
        Self {
            state,
            on_init,
            build,
            reduce,
        }
    }
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
    /// Grouped callbacks and state used during window initialization.
    pub callbacks: OpenParentedCallbacks<State, Init, Build, Reduce>,
    /// Reuse behavior for repeated open calls.
    pub mode: OpenParentedMode,
}

impl<State, Init, Build, Reduce> OpenParentedRequest<State, Init, Build, Reduce> {
    /// Backward-compatible constructor name for existing call sites.
    pub fn new(
        title: String,
        size: Size,
        state: State,
        on_init: Init,
        build: Build,
        reduce: Reduce,
    ) -> Self {
        Self::with_callbacks(
            title,
            size,
            OpenParentedCallbacks::new(state, on_init, build, reduce),
        )
    }

    /// Build an open request using [`OpenParentedMode::ReuseIfOpen`] with grouped callbacks.
    pub fn with_callbacks(
        title: String,
        size: Size,
        callbacks: OpenParentedCallbacks<State, Init, Build, Reduce>,
    ) -> Self {
        Self {
            title,
            size,
            callbacks,
            mode: OpenParentedMode::ReuseIfOpen,
        }
    }

    /// Override the default reuse mode.
    pub fn with_mode(mut self, mode: OpenParentedMode) -> Self {
        self.mode = mode;
        self
    }
}
