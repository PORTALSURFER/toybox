//! Internal CLAP plugin framework library.
//!
//! This crate hosts reusable DSP, GUI, parameter/state, and CLAP integration
//! utilities extracted from existing test plugins. The initial surface area
//! focuses on small, composable helpers that preserve realtime safety.

pub mod dsp;
pub mod clap;
#[cfg(feature = "gui")]
pub mod gui;

/// Re-exported CLAP crates so downstream plugins only depend on `toybox`.
pub use clack_extensions;
pub use clack_plugin;
pub use clack_common;
/// Re-export MTS-ESP for shared tuning support across plugins.
pub use mts_esp;

/// Re-export GUI dependencies behind the `gui` feature flag.
#[cfg(feature = "gui")]
pub use baseview;
#[cfg(feature = "gui")]
pub use egui_baseview;
#[cfg(feature = "gui")]
pub use raw_window_handle;
