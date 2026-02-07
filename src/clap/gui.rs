//! Strict declarative Patchbay GUI helpers for CLAP plugins.

use crate::logging::log_line_safe;
use clack_extensions::gui::GuiSize;
use clack_plugin::plugin::PluginError;
use patchbay_gui::{GuiError, HostWindow, Size, UiAction, UiSpec};
use raw_window_handle::RawWindowHandle;

/// Re-export Patchbay GUI types for downstream declarative GUI integrations.
pub use patchbay_gui::{Color, InputState, OpenParentedMode, ThemeTokens};

/// Host-resize policy for CLAP Patchbay GUI windows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HostResizePolicy {
    /// Accept host resize requests and forward them to Patchbay.
    #[default]
    Enabled,
    /// Ignore host resize requests and report a fixed-size window.
    Disabled,
}

/// Wrapper around a Patchbay GUI window for a CLAP editor.
#[derive(Default)]
pub struct GuiHostWindow {
    /// Underlying host window adapter from `patchbay-gui`.
    inner: HostWindow,
    /// Policy controlling host-driven resize behavior.
    host_resize_policy: HostResizePolicy,
}

impl GuiHostWindow {
    /// Set the raw parent handle provided by the host.
    pub fn set_parent(&mut self, parent: RawWindowHandle) {
        log_line_safe("toybox/gui: set_parent");
        self.inner.set_parent(parent);
    }

    /// Return the most recently observed logical size, if any.
    pub fn last_size(&self) -> Option<(u32, u32)> {
        self.inner.last_size()
    }

    /// Return true if a native window has been created.
    pub fn is_open(&self) -> bool {
        self.inner.is_open()
    }

    /// Show the native window if it exists.
    pub fn show(&self) {
        log_line_safe("toybox/gui: show");
        self.inner.show();
    }

    /// Hide the native window if it exists.
    pub fn hide(&self) {
        log_line_safe("toybox/gui: hide");
        self.inner.hide();
    }

    /// Request a logical resize from the GUI thread.
    pub fn request_resize(&self, width: u32, height: u32) {
        log_line_safe(&format!(
            "toybox/gui: request_resize width={} height={}",
            width, height
        ));
        self.inner.request_resize(width, height);
    }

    /// Set host-resize behavior for this window.
    pub fn set_host_resize_policy(&mut self, policy: HostResizePolicy) {
        log_line_safe(&format!(
            "toybox/gui: set_host_resize_policy policy={policy:?}"
        ));
        self.host_resize_policy = policy;
    }

    /// Disable host-driven resize handling for this window.
    pub fn disable_host_resize(&mut self) {
        self.set_host_resize_policy(HostResizePolicy::Disabled);
    }

    /// Return the canonical host-resize policy for Patchbay CLAP windows.
    ///
    /// Patchbay GUIs are designed to accept host-driven resize requests.
    pub const fn host_resize_enabled(&self) -> bool {
        matches!(self.host_resize_policy, HostResizePolicy::Enabled)
    }

    /// Normalize a host-provided GUI size to Patchbay's non-zero constraints.
    ///
    /// CLAP hosts may report zero during transient resize negotiation; Patchbay
    /// always clamps to at least `1x1`.
    pub fn normalize_host_size(&self, size: GuiSize) -> GuiSize {
        GuiSize {
            width: size.width.max(1),
            height: size.height.max(1),
        }
    }

    /// Resolve the host-adjusted size according to the current resize policy.
    ///
    /// Returns `None` when host-driven resizing is disabled.
    pub fn adjust_host_size(&self, size: GuiSize) -> Option<GuiSize> {
        self.host_resize_enabled()
            .then(|| self.normalize_host_size(size))
    }

    /// Apply a host-driven GUI resize request using Patchbay's policy.
    ///
    /// This keeps resize ownership in Toybox so plugin implementations do not
    /// need per-plugin resize forwarding logic.
    pub fn apply_host_size(&self, size: GuiSize) {
        if !self.host_resize_enabled() {
            log_line_safe("toybox/gui: apply_host_size ignored (resize disabled)");
            return;
        }
        let normalized = self.normalize_host_size(size);
        self.request_resize(normalized.width, normalized.height);
    }

    /// Set an optional aspect ratio for window resizing.
    pub fn set_aspect_ratio(&mut self, ratio: Option<f32>) {
        log_line_safe(&format!("toybox/gui: set_aspect_ratio ratio={ratio:?}"));
        self.inner.set_aspect_ratio(ratio);
    }

    /// Open a parented Patchbay GUI window.
    ///
    /// The caller supplies the initial state, a declarative UI builder, and an
    /// action reducer. The helper handles resize requests and stores the last
    /// logical size.
    pub fn open_parented<State, Init, Build, Reduce>(
        &mut self,
        title: String,
        size: (u32, u32),
        state: State,
        on_init: Init,
        build: Build,
        reduce: Reduce,
    ) -> Result<(), PluginError>
    where
        Init: FnMut(&mut State) + Send + 'static,
        Build: FnMut(&patchbay_gui::InputState, &State) -> UiSpec + Send + 'static,
        Reduce: FnMut(&mut State, UiAction) + Send + 'static,
        State: Send + 'static,
    {
        self.open_parented_with(
            title,
            size,
            state,
            on_init,
            build,
            reduce,
            OpenParentedMode::Recreate,
        )
    }

    /// Open a parented window, reusing it if it is already open.
    ///
    /// This mirrors Patchbay's default behavior: if a window is already open
    /// and attached to the same parent, the new state is ignored and the
    /// existing window is shown.
    pub fn open_parented_reuse<State, Init, Build, Reduce>(
        &mut self,
        title: String,
        size: (u32, u32),
        state: State,
        on_init: Init,
        build: Build,
        reduce: Reduce,
    ) -> Result<(), PluginError>
    where
        Init: FnMut(&mut State) + Send + 'static,
        Build: FnMut(&patchbay_gui::InputState, &State) -> UiSpec + Send + 'static,
        Reduce: FnMut(&mut State, UiAction) + Send + 'static,
        State: Send + 'static,
    {
        self.open_parented_with(
            title,
            size,
            state,
            on_init,
            build,
            reduce,
            OpenParentedMode::ReuseIfOpen,
        )
    }

    /// Open a parented window with an explicit reuse policy.
    ///
    /// The `size` argument is used as the initial window size.
    #[allow(clippy::too_many_arguments)]
    pub fn open_parented_with<State, Init, Build, Reduce>(
        &mut self,
        title: String,
        size: (u32, u32),
        state: State,
        on_init: Init,
        build: Build,
        reduce: Reduce,
        mode: OpenParentedMode,
    ) -> Result<(), PluginError>
    where
        Init: FnMut(&mut State) + Send + 'static,
        Build: FnMut(&patchbay_gui::InputState, &State) -> UiSpec + Send + 'static,
        Reduce: FnMut(&mut State, UiAction) + Send + 'static,
        State: Send + 'static,
    {
        log_line_safe(&format!(
            "toybox/gui: open_parented title=\"{}\" requested_size={}x{} mode={mode:?}",
            title, size.0, size.1
        ));
        self.inner
            .open_parented_with(
                title,
                Size {
                    width: size.0.max(1),
                    height: size.1.max(1),
                },
                state,
                on_init,
                build,
                reduce,
                mode,
            )
            .map_err(map_gui_error)
    }
}

/// Convert a Patchbay GUI error into a stable host-facing plugin error message.
fn map_gui_error(err: GuiError) -> PluginError {
    log_line_safe(&format!("toybox/gui: open_parented error: {err:?}"));
    PluginError::Message(match err {
        GuiError::NoParent => "No parent window provided",
        GuiError::UnsupportedHandle => "Unsupported host window handle",
        GuiError::WindowCreateFailed => "Failed to create GUI window",
        GuiError::AdapterNotFound => "No compatible GPU adapter found",
        GuiError::Surface(_) => "Failed to create GPU surface",
        GuiError::Device(_) => "Failed to create GPU device",
        GuiError::SurfaceFormat => "No compatible swapchain format",
        GuiError::SurfaceAcquire(_) => "Failed to acquire swapchain texture",
        GuiError::ThreadSpawn => "Failed to spawn GUI thread",
        GuiError::DeviceCachePoison => "GUI device cache was poisoned",
    })
}

/// Inject default CLAP resize callbacks backed by Toybox Patchbay policy.
///
/// This macro standardizes host-resize behavior across plugins: resizable by
/// default, with an opt-out via [`GuiHostWindow::set_host_resize_policy`].
#[cfg(feature = "gui")]
#[macro_export]
macro_rules! patchbay_clap_resize_callbacks {
    ($field:ident) => {
        fn can_resize(&mut self) -> bool {
            self.$field.host_resize_enabled()
        }

        fn adjust_size(
            &mut self,
            size: $crate::clack_extensions::gui::GuiSize,
        ) -> Option<$crate::clack_extensions::gui::GuiSize> {
            self.$field.adjust_host_size(size)
        }

        fn set_size(
            &mut self,
            size: $crate::clack_extensions::gui::GuiSize,
        ) -> Result<(), $crate::clack_plugin::plugin::PluginError> {
            self.$field.apply_host_size(size);
            Ok(())
        }
    };
}
