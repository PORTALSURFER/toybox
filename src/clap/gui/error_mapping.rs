//! Mapping between Patchbay GUI errors and stable CLAP plugin errors.

use crate::logging::log_line_safe;
use clack_plugin::plugin::PluginError;
use patchbay_gui::GuiError;

/// Convert a Patchbay GUI error into a stable host-facing plugin error message.
pub(super) fn map_gui_error(err: GuiError) -> PluginError {
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
