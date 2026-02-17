//! Parameter conversion and parsing helpers for the minimal VST3 example.

use toybox::vst3::prelude::ParamRange;

/// Convert gain in linear domain to normalized VST3 value.
pub(crate) fn gain_to_normalized(gain: f64) -> f64 {
    ParamRange::new(0.0, 2.0).plain_to_normalized(gain)
}

/// Convert normalized VST3 value to linear gain.
pub(crate) fn normalized_to_gain(value: f64) -> f64 {
    ParamRange::new(0.0, 2.0).normalized_to_plain(value)
}
