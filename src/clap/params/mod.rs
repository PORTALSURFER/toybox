//! CLAP parameter helpers for metadata, value/text conversion, and automation.
//!
//! This module is intentionally split into focused submodules to keep file and
//! function size manageable while preserving a flat public API for callers.

/// Parameter event application logic for applying host events to plugin state.
mod event_apply;
/// Parameter event serialization helpers used by automation and output paths.
mod event_output;
/// Parameter metadata helpers for parameter registration and descriptions.
mod metadata;
/// Toggle text mapping helpers for boolean parameter value I/O.
mod toggle_text;

pub use event_apply::{apply_param_events, apply_param_events_from_unknown};
pub use event_output::{
    ParamEventContext, push_param_gesture_begin, push_param_gesture_end, push_param_mod,
    push_param_value,
};
pub use metadata::{ParamBuilder, ParamSpec};
pub use toggle_text::{bool_to_param, param_to_bool, parse_toggle_text, write_toggle_text};

#[cfg(test)]
mod tests;
