//! Time and unit conversion helpers for DSP code.

/// Convert milliseconds to samples for the given sample rate.
///
/// Returns at least 1 sample to avoid zero-length delays.
pub fn ms_to_samples(ms: f32, sample_rate: f32) -> usize {
    ((ms / 1000.0) * sample_rate).round().max(1.0) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ms_to_samples_rounds_up_to_one() {
        assert_eq!(ms_to_samples(0.0, 48_000.0), 1);
    }
}
