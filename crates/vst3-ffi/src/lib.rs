//! Raw VST3 ABI bindings used by toybox.
//!
//! The SDK path is validated from `VST3_SDK_DIR` during build, while ABI
//! types are sourced from the generated `vst3` crate bindings.

pub use vst3::*;
