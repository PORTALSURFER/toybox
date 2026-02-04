//! Patchbay GUI helpers for CLAP plugins.

use clack_plugin::plugin::PluginError;
use patchbay_gui::{GuiError, HostWindow};
use raw_window_handle::RawWindowHandle;
use crate::logging::log_line_safe;

/// Re-export Patchbay GUI types for downstream GUI integrations.
pub use patchbay_gui::{
    ButtonResponse, Canvas, Color, DropdownResponse, InputState, KnobResponse, Layout,
    OpenParentedMode, RegionResponse, RootFrameResponse, RootFrameStyle, SliderResponse, Theme,
    ToggleResponse, Ui, WidgetId,
};

/// Wrapper around a Patchbay GUI window for a CLAP editor.
#[derive(Default)]
pub struct GuiHostWindow {
    inner: HostWindow,
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

    /// Set an optional aspect ratio for window resizing.
    pub fn set_aspect_ratio(&mut self, ratio: Option<f32>) {
        log_line_safe(&format!("toybox/gui: set_aspect_ratio ratio={ratio:?}"));
        self.inner.set_aspect_ratio(ratio);
    }

    /// Open a parented Patchbay GUI window.
    ///
    /// The caller supplies the initial state and the GUI update callback. The
    /// helper handles resize requests and stores the last logical size. This
    /// recreates the window each call so new state is applied; use
    /// `open_parented_reuse` to keep an existing window.
    pub fn open_parented<State, Init, Frame>(
        &mut self,
        title: String,
        size: (u32, u32),
        state: State,
        on_init: Init,
        on_frame: Frame,
    ) -> Result<(), PluginError>
    where
        Init: FnMut(&mut patchbay_gui::Ui<'_>, &mut State) + Send + 'static,
        Frame: FnMut(&mut patchbay_gui::Ui<'_>, &mut State) + Send + 'static,
        State: Send + 'static,
    {
        self.open_parented_with(
            title,
            size,
            state,
            on_init,
            on_frame,
            OpenParentedMode::Recreate,
        )
    }

    /// Open a parented window, reusing it if it is already open.
    ///
    /// This mirrors Patchbay's default behavior: if a window is already open
    /// and attached to the same parent, the new state is ignored and the
    /// existing window is shown.
    pub fn open_parented_reuse<State, Init, Frame>(
        &mut self,
        title: String,
        size: (u32, u32),
        state: State,
        on_init: Init,
        on_frame: Frame,
    ) -> Result<(), PluginError>
    where
        Init: FnMut(&mut patchbay_gui::Ui<'_>, &mut State) + Send + 'static,
        Frame: FnMut(&mut patchbay_gui::Ui<'_>, &mut State) + Send + 'static,
        State: Send + 'static,
    {
        self.open_parented_with(
            title,
            size,
            state,
            on_init,
            on_frame,
            OpenParentedMode::ReuseIfOpen,
        )
    }

    /// Open a parented window with an explicit reuse policy.
    pub fn open_parented_with<State, Init, Frame>(
        &mut self,
        title: String,
        size: (u32, u32),
        state: State,
        on_init: Init,
        on_frame: Frame,
        mode: OpenParentedMode,
    ) -> Result<(), PluginError>
    where
        Init: FnMut(&mut patchbay_gui::Ui<'_>, &mut State) + Send + 'static,
        Frame: FnMut(&mut patchbay_gui::Ui<'_>, &mut State) + Send + 'static,
        State: Send + 'static,
    {
        log_line_safe(&format!(
            "toybox/gui: open_parented title=\"{}\" size={}x{} mode={mode:?}",
            title, size.0, size.1
        ));
        self.inner
            .open_parented_with(title, size, state, on_init, on_frame, mode)
            .map_err(map_gui_error)
    }
}

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
