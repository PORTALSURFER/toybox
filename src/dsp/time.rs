//! Time and unit conversion helpers for DSP code.

/// Convert milliseconds to samples for the given sample rate.
///
/// Returns at least 1 sample to avoid zero-length delays.
pub fn ms_to_samples(ms: f32, sample_rate: f32) -> usize {
    ((ms / 1000.0) * sample_rate).round().max(1.0) as usize
}

/// Convert a control-rate frequency to a sample interval.
///
/// Returns at least 1 sample to avoid zero-length intervals.
pub fn hz_to_samples(rate_hz: f32, sample_rate: f32) -> usize {
    if rate_hz <= 0.0 {
        return 1;
    }
    (sample_rate / rate_hz).round().max(1.0) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ms_to_samples_rounds_up_to_one() {
        assert_eq!(ms_to_samples(0.0, 48_000.0), 1);
    }

    #[test]
    fn hz_to_samples_rounds_up_to_one() {
        assert_eq!(hz_to_samples(0.0, 48_000.0), 1);
        assert_eq!(hz_to_samples(1.0, 48_000.0), 48_000);
    }
}
