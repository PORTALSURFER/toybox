//! CLAP-specific helpers and glue for plugin implementations.
//!
//! This module provides thin wrappers around clack to reduce boilerplate while
//! keeping data flow explicit and realtime-safe.

pub mod bundle;
pub mod entry;
pub mod events;
pub mod params;
pub mod process;
pub mod registration;
#[cfg(feature = "gui")]
pub mod gui;
