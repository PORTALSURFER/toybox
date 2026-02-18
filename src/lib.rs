//! Internal CLAP and VST3 plugin framework library.
//!
//! This crate hosts reusable DSP, GUI, parameter/state, and host-integration
//! utilities extracted from existing test plugins. The initial surface area
//! focuses on small, composable helpers that preserve realtime safety for both
//! CLAP and VST3 integrations.

// Keep doc expectations visible at the crate boundary. The workspace lints also
// enforce this, but having it here makes the standard obvious to contributors.
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod clap;
pub mod dsp;
#[cfg(feature = "gui")]
pub mod gui;
#[cfg(feature = "gui")]
mod logging;
mod state;
#[cfg(feature = "vst3")]
pub mod vst3;

pub use clack_common;
/// Re-exported CLAP crates so downstream plugins only depend on `toybox`.
pub use clack_extensions;
pub use clack_plugin;
/// Re-export MTS-ESP for shared tuning support across plugins.
pub use mts_esp;
/// Re-export raw generated VST3 ABI bindings.
#[cfg(feature = "vst3")]
pub use toybox_vst3_ffi;

/// Re-export raw-window-handle for host parent integration code.
#[cfg(feature = "gui")]
pub use raw_window_handle;
