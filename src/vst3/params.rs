//! Parameter helpers for VST3 controller and processor implementations.

use toybox_vst3_ffi::ComRef;
use toybox_vst3_ffi::Steinberg::Vst::{
    IParamValueQueueTrait, IParameterChanges, IParameterChangesTrait, ParamID, ParamValue,
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
    let Some(changes) = (unsafe { ComRef::from_raw(changes) }) else {
        return None;
    };

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

#[cfg(test)]
mod tests {
    use super::ParamRange;

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
