//! EQ helpers shared across plugins.

/// Fill log-spaced band center frequencies between the given min/max range.
///
/// The `bands` argument controls how many active bands should be filled. If it
/// exceeds `out.len()`, it is clamped to the output length. Remaining values
/// are filled with the last valid band center to keep arrays stable.
pub fn fill_log_spaced_frequencies(out: &mut [f32], min_hz: f32, max_hz: f32, bands: usize) {
    if out.is_empty() {
        return;
    }

    let bands = bands.max(1).min(out.len());
    let log_min = min_hz.max(1.0).ln();
    let log_max = max_hz.max(min_hz + 1.0).ln();

    for (index, freq) in out.iter_mut().enumerate().take(bands) {
        let t = if bands > 1 {
            index as f32 / (bands - 1) as f32
        } else {
            0.0
        };
        *freq = (log_min + (log_max - log_min) * t).exp();
    }

    if bands < out.len() {
        let last = out[bands - 1];
        for freq in out.iter_mut().skip(bands) {
            *freq = last;
        }
    }
}

/// Decide whether band center frequencies should be recomputed.
///
/// The comparison uses both absolute and ratio thresholds so small movements
/// near zero or large shifts at high frequencies are handled consistently.
pub fn band_frequencies_need_update(
    last_min: f32,
    last_max: f32,
    next_min: f32,
    next_max: f32,
    epsilon_hz: f32,
    epsilon_ratio: f32,
) -> bool {
    let min_delta = (last_min - next_min).abs();
    let max_delta = (last_max - next_max).abs();
    if min_delta > epsilon_hz || max_delta > epsilon_hz {
        return true;
    }

    let min_ratio = if last_min > 0.0 && next_min > 0.0 {
        (next_min / last_min).max(last_min / next_min)
    } else {
        0.0
    };
    let max_ratio = if last_max > 0.0 && next_max > 0.0 {
        (next_max / last_max).max(last_max / next_max)
    } else {
        0.0
    };

    (min_ratio - 1.0) > epsilon_ratio || (max_ratio - 1.0) > epsilon_ratio
}

#[cfg(test)]
mod tests {
    use super::{band_frequencies_need_update, fill_log_spaced_frequencies};

    #[test]
    fn fills_log_spaced_frequencies() {
        let mut bands = [0.0_f32; 4];
        fill_log_spaced_frequencies(&mut bands, 20.0, 20_000.0, 4);
        assert!(bands[0] >= 20.0);
        assert!(bands[3] <= 20_000.0);
        assert!(bands[1] > bands[0]);
        assert!(bands[2] > bands[1]);
    }

    #[test]
    fn clamps_and_fills_tail() {
        let mut bands = [0.0_f32; 3];
        fill_log_spaced_frequencies(&mut bands, 100.0, 1_000.0, 1);
        assert_eq!(bands[0], bands[1]);
        assert_eq!(bands[1], bands[2]);
    }

    #[test]
    fn band_frequency_update_uses_thresholds() {
        assert!(!band_frequencies_need_update(
            100.0, 1_000.0, 100.1, 1_000.1, 1.0, 0.01
        ));
        assert!(band_frequencies_need_update(
            100.0, 1_000.0, 102.0, 1_000.0, 1.0, 0.01
        ));
        assert!(band_frequencies_need_update(
            100.0, 1_000.0, 100.0, 1_050.0, 1.0, 0.01
        ));
    }
}
