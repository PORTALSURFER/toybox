//! Shared constants for the minimal VST3 example plugin.

use toybox::vst3::prelude::ParamID;

/// Human-readable plugin name shown in hosts.
pub(crate) const PLUGIN_NAME: &str = "Toybox Minimal Gain";
/// VST3 parameter id for gain.
pub(crate) const PARAM_GAIN_ID: ParamID = 0;
/// State payload magic (`TVST`).
pub(crate) const STATE_MAGIC: u32 = u32::from_le_bytes(*b"TVST");
/// State payload version.
pub(crate) const STATE_VERSION: u32 = 1;
