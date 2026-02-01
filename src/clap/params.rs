//! CLAP parameter helpers for metadata, value/text conversion, and automation.

use std::ffi::CStr;
use std::fmt::Write;

use clack_extensions::params::{ParamDisplayWriter, ParamInfo, ParamInfoFlags, ParamInfoWriter};
use clack_plugin::events::spaces::CoreEventSpace;
use clack_plugin::prelude::UnknownEvent;
use clack_plugin::utils::ClapId;
use clack_plugin::events::io::InputEvents;

/// Describes a CLAP parameter's metadata for registration with the host.
pub struct ParamSpec<'a> {
    /// Stable CLAP parameter identifier.
    pub id: ClapId,
    /// CLAP parameter flags (automation, stepped, etc.).
    pub flags: ParamInfoFlags,
    /// Parameter display name.
    pub name: &'a [u8],
    /// Parameter module/group name.
    pub module: &'a [u8],
    /// Minimum value.
    pub min_value: f64,
    /// Maximum value.
    pub max_value: f64,
    /// Default value.
    pub default_value: f64,
}

impl ParamSpec<'_> {
    /// Write this spec into the CLAP parameter info writer.
    pub fn write(&self, writer: &mut ParamInfoWriter) {
        writer.set(&ParamInfo {
            id: self.id,
            flags: self.flags,
            cookie: Default::default(),
            name: self.name,
            module: self.module,
            min_value: self.min_value,
            max_value: self.max_value,
            default_value: self.default_value,
        });
    }
}

/// Apply incoming automation events to a handler callback.
pub fn apply_param_events<F>(input: &InputEvents<'_>, mut apply: F)
where
    F: FnMut(ClapId, f64),
{
    for event in input {
        if let Some(CoreEventSpace::ParamValue(param)) = event.as_core_event() {
            if let Some(param_id) = param.param_id() {
                apply(param_id, param.value());
            }
        }
    }
}

/// Convenience helper to apply automation from a list of unknown events.
pub fn apply_param_events_from_unknown<F>(events: &[&UnknownEvent], mut apply: F)
where
    F: FnMut(ClapId, f64),
{
    for event in events {
        if let Some(CoreEventSpace::ParamValue(param)) = event.as_core_event() {
            if let Some(param_id) = param.param_id() {
                apply(param_id, param.value());
            }
        }
    }
}

/// Convert a boolean to a CLAP parameter value.
pub fn bool_to_param(value: bool) -> f64 {
    if value { 1.0 } else { 0.0 }
}

/// Convert a CLAP parameter value into a boolean.
pub fn param_to_bool(value: f64) -> bool {
    value >= 0.5
}

/// Write a simple on/off display label.
pub fn write_toggle_text(
    writer: &mut ParamDisplayWriter,
    value: f64,
    on_label: &str,
    off_label: &str,
) -> std::fmt::Result {
    if param_to_bool(value) {
        write!(writer, "{on_label}")
    } else {
        write!(writer, "{off_label}")
    }
}

/// Parse a toggle text value into a CLAP parameter value.
pub fn parse_toggle_text(text: &CStr, on_label: &str, off_label: &str) -> Option<f64> {
    let raw_text = text.to_str().ok()?.trim();
    if raw_text.eq_ignore_ascii_case(on_label) {
        Some(1.0)
    } else if raw_text.eq_ignore_ascii_case(off_label) {
        Some(0.0)
    } else {
        None
    }
}
