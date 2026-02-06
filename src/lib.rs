//! Internal CLAP plugin framework library.
//!
//! This crate hosts reusable DSP, GUI, parameter/state, and CLAP integration
//! utilities extracted from existing test plugins. The initial surface area
//! focuses on small, composable helpers that preserve realtime safety.

pub mod clap;
pub mod dsp;
#[cfg(feature = "gui")]
pub mod gui;
#[cfg(feature = "gui")]
mod logging;

pub use clack_common;
/// Re-exported CLAP crates so downstream plugins only depend on `toybox`.
pub use clack_extensions;
pub use clack_plugin;
/// Re-export MTS-ESP for shared tuning support across plugins.
pub use mts_esp;

/// Re-export raw-window-handle for host parent integration code.
#[cfg(feature = "gui")]
pub use raw_window_handle;
