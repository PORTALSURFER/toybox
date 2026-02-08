//! Minimal VST3 gain plugin showcasing toybox VST3 helpers.

#![deny(clippy::missing_docs_in_private_items, missing_docs, warnings)]

mod constants;
mod controller;
mod factory;
mod params;
mod processor;
mod state_io;
mod view;

use crate::factory::Factory;

toybox::vst3_plugin_entry!(Factory);
