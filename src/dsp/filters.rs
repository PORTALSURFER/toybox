//! Filters and filter helpers used in multiple plugins.

use std::f32::consts::PI;

/// A simple one-pole low-pass filter used for smoothing and damping.
#[derive(Debug, Clone, Copy)]
pub struct OnePole {
    coefficient: f32,
    state: f32,
}

impl OnePole {
    /// Create a new filter with the given coefficient.
    ///
    /// `coefficient` is clamped to 0.0..=1.0, where 0.0 is no smoothing and
    /// 1.0 is heavy smoothing.
    pub fn new(coefficient: f32) -> Self {
        Self {
            coefficient: coefficient.clamp(0.0, 1.0),
            state: 0.0,
        }
    }

    /// Update the filter coefficient (0.0 = no smoothing, 1.0 = heavy smoothing).
    pub fn set_coefficient(&mut self, coefficient: f32) {
        self.coefficient = coefficient.clamp(0.0, 1.0);
    }

    /// Process a single sample through the filter.
    pub fn process(&mut self, input: f32) -> f32 {
        self.state += (input - self.state) * (1.0 - self.coefficient);
        self.state
    }
}

/// Single biquad filter state for one channel.
#[derive(Clone, Copy, Default)]
pub struct BiquadState {
    /// Delay element z1.
    z1: f32,
    /// Delay element z2.
    z2: f32,
}

/// Normalized biquad coefficients for a peaking EQ section.
#[derive(Clone, Copy, Default)]
pub struct BiquadCoeffs {
    /// Feedforward coefficient b0.
    pub b0: f32,
    /// Feedforward coefficient b1.
    pub b1: f32,
    /// Feedforward coefficient b2.
    pub b2: f32,
    /// Feedback coefficient a1.
    pub a1: f32,
    /// Feedback coefficient a2.
    pub a2: f32,
}

/// Compute peaking EQ coefficients using the RBJ cookbook formulas.
///
/// `freq_hz` is clamped to Nyquist, and `q` is clamped to a minimum of 0.01.
pub fn peaking_eq_coeffs(sample_rate: f32, freq_hz: f32, q: f32, gain_db: f32) -> BiquadCoeffs {
    let omega = 2.0 * PI * (freq_hz / sample_rate.max(1.0)).clamp(0.0, 0.499);
    let sin = omega.sin();
    let cos = omega.cos();
    let alpha = sin / (2.0 * q.max(0.01));
    let a = 10.0_f32.powf(gain_db / 40.0);

    let b0 = 1.0 + alpha * a;
    let b1 = -2.0 * cos;
    let b2 = 1.0 - alpha * a;
    let a0 = 1.0 + alpha / a;
    let a1 = -2.0 * cos;
    let a2 = 1.0 - alpha / a;

    let inv_a0 = 1.0 / a0;
    BiquadCoeffs {
        b0: b0 * inv_a0,
        b1: b1 * inv_a0,
        b2: b2 * inv_a0,
        a1: a1 * inv_a0,
        a2: a2 * inv_a0,
    }
}

/// Process one sample through a biquad filter.
pub fn process_biquad(sample: f32, coeffs: &BiquadCoeffs, state: &mut BiquadState) -> f32 {
    let out = coeffs.b0 * sample + state.z1;
    state.z1 = coeffs.b1 * sample - coeffs.a1 * out + state.z2;
    state.z2 = coeffs.b2 * sample - coeffs.a2 * out;
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_pole_smooths_step() {
        let mut filter = OnePole::new(0.9);
        let _ = filter.process(0.0);
        let out = filter.process(1.0);
        assert!(out > 0.0 && out < 1.0);
    }

    #[test]
    fn peaking_eq_coeffs_is_finite() {
        let coeffs = peaking_eq_coeffs(48_000.0, 1_000.0, 0.7, 3.0);
        assert!(coeffs.b0.is_finite());
        assert!(coeffs.a2.is_finite());
    }
}
