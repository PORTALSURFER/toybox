//! One-pole smoothing helpers used in multiple plugins.

/// Smooth a value toward a target using a decay coefficient.
///
/// `coeff` is the per-sample decay applied to the difference. Values close to
/// 1.0 produce heavy smoothing; 0.0 snaps to the target immediately.
pub fn smooth_value(current: f32, target: f32, coeff: f32) -> f32 {
    target + (current - target) * coeff
}

/// Compute the exponential smoothing coefficient for a time constant.
///
/// The returned value is suitable for use with `smooth_value` on each sample.
/// `time_sec` is clamped to a small minimum to avoid division by zero.
pub fn exp_smoothing_coeff(sample_rate: f32, time_sec: f32) -> f32 {
    let time_sec = time_sec.max(0.0001);
    (-1.0 / (sample_rate.max(1.0) * time_sec)).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smooth_value_respects_coeff_bounds() {
        let current = 0.5;
        let target = 1.0;
        assert_eq!(smooth_value(current, target, 0.0), target);
        assert_eq!(smooth_value(current, target, 1.0), current);
    }

    #[test]
    fn exp_smoothing_coeff_is_between_zero_and_one() {
        let coeff = exp_smoothing_coeff(48_000.0, 0.01);
        assert!(coeff > 0.0 && coeff < 1.0);
    }
}
