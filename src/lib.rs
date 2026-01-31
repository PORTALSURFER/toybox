//! Internal CLAP plugin framework library.
//!
//! This crate hosts reusable DSP, GUI, parameter/state, and CLAP integration
//! utilities extracted from existing test plugins. The initial surface area
//! focuses on small, composable helpers that preserve realtime safety.

pub mod dsp;
