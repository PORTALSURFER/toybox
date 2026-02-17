//! CLAP-specific helpers and glue for plugin implementations.
//!
//! This module provides thin wrappers around clack to reduce boilerplate while
//! keeping data flow explicit and realtime-safe.

pub mod automation;
pub mod bundle;
pub mod entry;
pub mod events;
#[cfg(feature = "gui")]
pub mod gui;
pub mod params;
pub mod prelude;
pub mod process;
pub mod transport;
pub mod registration;
pub mod state;
