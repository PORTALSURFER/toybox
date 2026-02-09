use std::ffi::CStr;
use std::fmt::Write;

use clack_extensions::params::ParamDisplayWriter;

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
