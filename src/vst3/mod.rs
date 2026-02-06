//! VST3-specific helpers and glue for plugin implementations.
//!
//! This module exposes a low-boilerplate authoring surface built on top of
//! generated VST3 ABI bindings.

pub mod bundle;
pub mod component;
pub mod entry;
pub mod gui;
pub mod params;
pub mod prelude;
pub mod processor;
pub mod registration;
pub mod state;
