//! Parameter conversion and parsing helpers for the minimal VST3 example.

use std::slice;
use std::str::FromStr;

use toybox::vst3::prelude::{ParamRange, TChar, tchar_len};

/// Convert gain in linear domain to normalized VST3 value.
pub(crate) fn gain_to_normalized(gain: f64) -> f64 {
    ParamRange::new(0.0, 2.0).plain_to_normalized(gain)
}

/// Convert normalized VST3 value to linear gain.
pub(crate) fn normalized_to_gain(value: f64) -> f64 {
    ParamRange::new(0.0, 2.0).normalized_to_plain(value)
}

/// Parse a UTF-16 VST3 string into `f64`.
pub(crate) unsafe fn parse_tchar_f64(text: *mut TChar) -> Option<f64> {
    if text.is_null() {
        return None;
    }

    // SAFETY: caller guarantees a valid null-terminated TChar pointer.
    let length = unsafe { tchar_len(text as *const TChar) };
    // SAFETY: caller guarantees this pointer references `length` UTF-16 units.
    let utf16 = unsafe { slice::from_raw_parts(text.cast::<u16>(), length) };
    let parsed = String::from_utf16(utf16).ok()?;
    let trimmed = parsed.trim().trim_end_matches('x');
    f64::from_str(trimmed).ok()
}
