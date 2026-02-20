//! CLAP-facing host window wrapper for Patchbay GUI integration.

use crate::logging::log_line_safe;
use clack_extensions::gui::GuiSize;
use clack_plugin::plugin::PluginError;
use patchbay_gui::{
    HostWindow, OpenParentedCallbacks, OpenParentedRequest as PatchbayOpenParentedRequest,
    ShortcutBinding, ShortcutModifiers, Size, UiAction, UiSpec,
};
use raw_window_handle::RawWindowHandle;

use super::error_mapping::map_gui_error;
use super::{GuiOpenCallbacks, GuiOpenRequest, HostResizePolicy};

include!("host_window/core.rs");
include!("host_window/resize_policy.rs");
include!("host_window/open_parented.rs");
