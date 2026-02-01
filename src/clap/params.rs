//! CLAP parameter helpers for metadata, value/text conversion, and automation.

use std::ffi::CStr;
use std::fmt::Write;

use clack_extensions::params::{ParamDisplayWriter, ParamInfo, ParamInfoFlags, ParamInfoWriter};
use clack_plugin::events::event_types::{
    ParamGestureBeginEvent, ParamGestureEndEvent, ParamModEvent, ParamValueEvent,
};
use clack_plugin::events::io::{InputEvents, OutputEvents, TryPushError};
use clack_plugin::events::spaces::CoreEventSpace;
use clack_plugin::events::Pckn;
use clack_plugin::prelude::UnknownEvent;
use clack_plugin::utils::{ClapId, Cookie};

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

/// Builder for CLAP parameter metadata.
///
/// This keeps param definitions compact while still producing a concrete [`ParamSpec`].
pub struct ParamBuilder<'a> {
    id: ClapId,
    flags: ParamInfoFlags,
    name: &'a [u8],
    module: &'a [u8],
    min_value: f64,
    max_value: f64,
    default_value: f64,
}

impl<'a> ParamBuilder<'a> {
    /// Create a new builder with a required id, name, and module label.
    pub fn new(id: ClapId, name: &'a [u8], module: &'a [u8]) -> Self {
        Self {
            id,
            flags: ParamInfoFlags::empty(),
            name,
            module,
            min_value: 0.0,
            max_value: 1.0,
            default_value: 0.0,
        }
    }

    /// Mark the parameter as automatable.
    pub fn automatable(mut self) -> Self {
        self.flags |= ParamInfoFlags::IS_AUTOMATABLE;
        self
    }

    /// Mark the parameter as stepped (integer values only).
    pub fn stepped(mut self) -> Self {
        self.flags |= ParamInfoFlags::IS_STEPPED;
        self
    }

    /// Mark the parameter as an enum.
    pub fn enumerated(mut self) -> Self {
        self.flags |= ParamInfoFlags::IS_ENUM;
        self
    }

    /// Set the parameter's numeric range.
    pub fn range(mut self, min_value: f64, max_value: f64) -> Self {
        self.min_value = min_value;
        self.max_value = max_value;
        self
    }

    /// Set the parameter's default value.
    pub fn default(mut self, default_value: f64) -> Self {
        self.default_value = default_value;
        self
    }

    /// Convert this builder into a concrete [`ParamSpec`].
    pub fn build(self) -> ParamSpec<'a> {
        ParamSpec {
            id: self.id,
            flags: self.flags,
            name: self.name,
            module: self.module,
            min_value: self.min_value,
            max_value: self.max_value,
            default_value: self.default_value,
        }
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

/// Push a CLAP parameter value event into an output event list.
pub fn push_param_value(
    output: &mut OutputEvents<'_>,
    time: u32,
    param_id: ClapId,
    value: f64,
    pckn: Pckn,
    cookie: Cookie,
) -> Result<(), TryPushError> {
    output.try_push(ParamValueEvent::new(time, param_id, pckn, value, cookie))
}

/// Push a CLAP parameter modulation event into an output event list.
pub fn push_param_mod(
    output: &mut OutputEvents<'_>,
    time: u32,
    param_id: ClapId,
    amount: f64,
    pckn: Pckn,
    cookie: Cookie,
) -> Result<(), TryPushError> {
    output.try_push(ParamModEvent::new(time, param_id, pckn, amount, cookie))
}

/// Push a CLAP parameter gesture begin event into an output event list.
pub fn push_param_gesture_begin(
    output: &mut OutputEvents<'_>,
    time: u32,
    param_id: ClapId,
) -> Result<(), TryPushError> {
    output.try_push(ParamGestureBeginEvent::new(time, param_id))
}

/// Push a CLAP parameter gesture end event into an output event list.
pub fn push_param_gesture_end(
    output: &mut OutputEvents<'_>,
    time: u32,
    param_id: ClapId,
) -> Result<(), TryPushError> {
    output.try_push(ParamGestureEndEvent::new(time, param_id))
}

#[cfg(test)]
mod tests {
    use super::{
        push_param_gesture_begin, push_param_gesture_end, push_param_mod, push_param_value,
        ParamBuilder,
    };

    use clack_plugin::events::io::EventBuffer;
    use clack_plugin::events::spaces::CoreEventSpace;
    use clack_plugin::events::Pckn;
    use clack_plugin::utils::{ClapId, Cookie};

    #[test]
    fn push_param_events_writes_output_buffer() {
        let param_id = ClapId::new(5);
        let mut buffer = EventBuffer::new();
        let mut output = buffer.as_output();

        push_param_gesture_begin(&mut output, 0, param_id).unwrap();
        push_param_value(
            &mut output,
            0,
            param_id,
            0.75,
            Pckn::match_all(),
            Cookie::empty(),
        )
        .unwrap();
        push_param_mod(
            &mut output,
            0,
            param_id,
            0.25,
            Pckn::match_all(),
            Cookie::empty(),
        )
        .unwrap();
        push_param_gesture_end(&mut output, 0, param_id).unwrap();

        assert_eq!(buffer.len(), 4);
        let mut saw_value = false;
        let mut saw_mod = false;

        for index in 0..buffer.len() {
            let event = buffer.get(index as u32).unwrap();
            if let Some(core) = event.as_core_event() {
                match core {
                    CoreEventSpace::ParamValue(value) => {
                        saw_value = value.param_id() == Some(param_id);
                    }
                    CoreEventSpace::ParamMod(mod_event) => {
                        saw_mod = mod_event.param_id() == Some(param_id);
                    }
                    _ => {}
                }
            }
        }

        assert!(saw_value);
        assert!(saw_mod);
    }

    #[test]
    fn param_builder_sets_fields() {
        let spec = ParamBuilder::new(ClapId::new(2), b"Rate", b"Rate")
            .automatable()
            .stepped()
            .range(0.0, 10.0)
            .default(1.0)
            .build();

        assert_eq!(spec.id, ClapId::new(2));
        assert_eq!(spec.min_value, 0.0);
        assert_eq!(spec.max_value, 10.0);
        assert_eq!(spec.default_value, 1.0);
        assert!(spec.flags.contains(clack_extensions::params::ParamInfoFlags::IS_AUTOMATABLE));
        assert!(spec.flags.contains(clack_extensions::params::ParamInfoFlags::IS_STEPPED));
    }
}
