//! Windowing and overlap-add helpers for spectral processing.

use std::f32::consts::PI;

/// Generate a Hann window of the requested size.
pub fn hann_window(size: usize) -> Vec<f32> {
    let denom = (size as f32 - 1.0).max(1.0);
    (0..size)
        .map(|i| 0.5 - 0.5 * (2.0 * PI * i as f32 / denom).cos())
        .collect()
}

/// Compute overlap-add normalization factors for a window and hop size.
///
/// Returns a per-sample multiplier that compensates for overlap energy.
pub fn overlap_normalization(window: &[f32], hop_size: usize) -> Vec<f32> {
    let size = window.len();
    let mut sums = vec![0.0; size];
    let mut offset = 0;
    while offset < size {
        for (index, value) in window.iter().enumerate() {
            sums[(index + offset) % size] += value * value;
        }
        offset += hop_size;
    }
    sums.into_iter()
        .map(|value| if value > 0.0 { 1.0 / value } else { 1.0 })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hann_window_has_expected_length() {
        let window = hann_window(16);
        assert_eq!(window.len(), 16);
        assert!(window.first().unwrap().abs() < 1e-6);
    }

    #[test]
    fn overlap_normalization_returns_non_zero() {
        let window = hann_window(16);
        let norm = overlap_normalization(&window, 4);
        assert_eq!(norm.len(), 16);
        assert!(norm.iter().all(|value| *value > 0.0));
    }
}
