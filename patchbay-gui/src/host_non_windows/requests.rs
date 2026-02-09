//! Request payloads and open-mode policy for non-Windows host stubs.

use crate::canvas::Size;

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
