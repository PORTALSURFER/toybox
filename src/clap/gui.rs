//! Strict declarative Patchbay GUI helpers for CLAP plugins.

mod error_mapping;
mod host_window;
mod macros;
mod requester;
mod types;

/// Re-export Patchbay GUI types for downstream declarative GUI integrations.
pub use patchbay_gui::{Color, InputState, OpenParentedMode, ThemeTokens};

pub use host_window::GuiHostWindow;
pub use requester::{HostParamRequester, host_param_requester};
pub use types::{GuiOpenCallbacks, GuiOpenRequest, HostResizePolicy};
