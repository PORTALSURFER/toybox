//! Types that define CLAP GUI open requests and resize policy.

use patchbay_gui::OpenParentedMode;

/// Host-resize policy for CLAP Patchbay GUI windows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HostResizePolicy {
    /// Accept host resize requests and forward them to Patchbay.
    #[default]
    Enabled,
    /// Ignore host resize requests and report a fixed-size window.
    Disabled,
}

/// Request payload for opening a CLAP parented Patchbay window.
///
/// This avoids wide function signatures while preserving explicit ownership of
/// title, size, state, and callbacks at call sites.
pub struct GuiOpenRequest<State, Init, Build, Reduce> {
    /// Window title shown by the host.
    pub title: String,
    /// Initial logical size in pixels.
    pub size: (u32, u32),
    /// Initial user-provided UI state.
    pub state: State,
    /// One-time state initialization callback.
    pub on_init: Init,
    /// Per-frame declarative UI builder callback.
    pub build: Build,
    /// UI action reducer callback.
    pub reduce: Reduce,
    /// Reuse behavior for repeated open calls.
    pub mode: OpenParentedMode,
}

impl<State, Init, Build, Reduce> GuiOpenRequest<State, Init, Build, Reduce> {
    /// Build a request using [`OpenParentedMode::Recreate`].
    pub fn new(
        title: String,
        size: (u32, u32),
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
            mode: OpenParentedMode::Recreate,
        }
    }

    /// Override the default reuse mode.
    pub fn with_mode(mut self, mode: OpenParentedMode) -> Self {
        self.mode = mode;
        self
    }
}
