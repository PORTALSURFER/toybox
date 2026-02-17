//! Parameter helpers for VST3 controller and processor implementations.

use std::slice;
use std::str::FromStr;

use toybox_vst3_ffi::ComRef;
use toybox_vst3_ffi::Steinberg::Vst::{
    IParamValueQueueTrait, IParameterChanges, IParameterChangesTrait, ParamID, ParamValue, TChar,
};
use toybox_vst3_ffi::Steinberg::kResultTrue;

/// Numeric parameter range used for normalized VST3 value conversions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParamRange {
    /// Minimum plain-value boundary.
    pub min: f64,
    /// Maximum plain-value boundary.
    pub max: f64,
}

impl ParamRange {
    /// Create a new range.
    ///
    /// # Panics
    ///
    /// Panics if `max <= min`.
    pub fn new(min: f64, max: f64) -> Self {
        assert!(max > min, "parameter range must satisfy max > min");
        Self { min, max }
    }

    /// Clamp a plain value to the range.
    pub fn clamp_plain(self, value: f64) -> f64 {
        value.clamp(self.min, self.max)
    }

    /// Convert plain value to normalized `[0.0, 1.0]` VST3 space.
    pub fn plain_to_normalized(self, value: f64) -> f64 {
        ((self.clamp_plain(value) - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
    }

    /// Convert normalized `[0.0, 1.0]` value back to plain parameter units.
    pub fn normalized_to_plain(self, normalized: f64) -> f64 {
        self.min + normalized.clamp(0.0, 1.0) * (self.max - self.min)
    }
}

/// Iterate every queued parameter automation point in a process block.
///
/// The callback receives `(param_id, sample_offset, normalized_value)`.
///
/// # Safety
///
/// `changes` must be a valid `IParameterChanges` pointer for the duration of
/// this call.
pub unsafe fn for_each_param_point(
    changes: *mut IParameterChanges,
    mut callback: impl FnMut(ParamID, i32, ParamValue),
) {
    let Some(changes) = (unsafe { ComRef::from_raw(changes) }) else {
        return;
    };

    let queue_count = unsafe { changes.getParameterCount() };
    for queue_index in 0..queue_count {
        let Some(queue) = (unsafe { ComRef::from_raw(changes.getParameterData(queue_index)) })
        else {
            continue;
        };

        let param_id = unsafe { queue.getParameterId() };
        let point_count = unsafe { queue.getPointCount() };
        for point_index in 0..point_count {
            let mut sample_offset = 0;
            let mut value = 0.0;
            if unsafe { queue.getPoint(point_index, &mut sample_offset, &mut value) } == kResultTrue
            {
                callback(param_id, sample_offset, value);
            }
        }
    }
}

/// Fetch the newest normalized point for one parameter in a process block.
///
/// Returns `(sample_offset, normalized_value)` for the latest point if found.
///
/// # Safety
///
/// `changes` must be a valid `IParameterChanges` pointer for the duration of
/// this call.
pub unsafe fn latest_param_point(
    changes: *mut IParameterChanges,
    target_param_id: ParamID,
) -> Option<(i32, ParamValue)> {
    let changes = (unsafe { ComRef::from_raw(changes) })?;

    let queue_count = unsafe { changes.getParameterCount() };
    for queue_index in 0..queue_count {
        let Some(queue) = (unsafe { ComRef::from_raw(changes.getParameterData(queue_index)) })
        else {
            continue;
        };

        if unsafe { queue.getParameterId() } != target_param_id {
            continue;
        }

        let point_count = unsafe { queue.getPointCount() };
        if point_count <= 0 {
            return None;
        }

        let mut sample_offset = 0;
        let mut value = 0.0;
        if unsafe { queue.getPoint(point_count - 1, &mut sample_offset, &mut value) } == kResultTrue
        {
            return Some((sample_offset, value));
        }
    }

    None
}

/// Parse a NUL-terminated Windows UTF-16 CLAP/VST3 text buffer into `f64`.
///
/// The implementation accepts optional suffixes such as `%`, `"dB"`, and `"x"`.
/// ignores outer whitespace.
///
/// # Safety
///
/// Callers must guarantee `text` points to a valid NUL-terminated UTF-16 buffer.
pub unsafe fn parse_tchar_f64(text: *mut TChar) -> Option<f64> {
    if text.is_null() {
        return None;
    }

    let length = unsafe { crate::vst3::gui::tchar_len(text as *const TChar) };
    let utf16 = unsafe { slice::from_raw_parts(text.cast::<u16>(), length) };
    let parsed = String::from_utf16(utf16).ok()?;
    let normalized = parsed
        .trim()
        .trim_end_matches('x')
        .trim_end_matches('%')
        .trim_end_matches("dB")
        .trim();
    f64::from_str(normalized).ok()
}

#[cfg(test)]
mod tests {
    use super::{parse_tchar_f64, ParamRange};
    use toybox_vst3_ffi::Steinberg::Vst::TChar;

    #[test]
    fn parse_tchar_f64_trims_and_parses_suffixes() {
        let mut percent: Vec<TChar> = b"42%".iter().map(|byte| u16::from(*byte) as TChar)
            .chain(Some(0))
            .collect();

        let parsed = unsafe { parse_tchar_f64(percent.as_mut_ptr()) };
        assert_eq!(parsed, Some(42.0));

        let mut decibel: Vec<TChar> = b" -6 dB "
            .iter()
            .map(|byte| u16::from(*byte) as TChar)
            .chain(Some(0))
            .collect();
        let parsed = unsafe { parse_tchar_f64(decibel.as_mut_ptr()) };
        assert_eq!(parsed, Some(-6.0));

        let mut linear: Vec<TChar> = b" 1.00x "
            .iter()
            .map(|byte| u16::from(*byte) as TChar)
            .chain(Some(0))
            .collect();
        let parsed = unsafe { parse_tchar_f64(linear.as_mut_ptr()) };
        assert_eq!(parsed, Some(1.0));
    }

    #[test]
    fn parse_tchar_f64_handles_invalid_and_empty_text() {
        let mut invalid: Vec<TChar> = b"abc".iter().map(|byte| u16::from(*byte) as TChar)
            .chain(Some(0))
            .collect();

        assert_eq!(unsafe { parse_tchar_f64(invalid.as_mut_ptr()) }, None);
        assert_eq!(unsafe { parse_tchar_f64(core::ptr::null_mut()) }, None);
    }

    #[test]
    fn plain_to_normalized_clamps_to_bounds() {
        let range = ParamRange::new(-12.0, 12.0);
        assert_eq!(range.plain_to_normalized(-24.0), 0.0);
        assert_eq!(range.plain_to_normalized(24.0), 1.0);
    }

    #[test]
    fn conversion_round_trip_is_stable_for_center_point() {
        let range = ParamRange::new(0.0, 2.0);
        let normalized = range.plain_to_normalized(1.0);
        let plain = range.normalized_to_plain(normalized);
        assert!((plain - 1.0).abs() < 1e-12);
    }
}
