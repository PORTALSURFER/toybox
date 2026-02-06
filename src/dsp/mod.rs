//! Reusable DSP utilities extracted from in-repo CLAP plugins.
//!
//! Modules here are intentionally small and composable. They provide basic
//! building blocks (filters, delay lines, smoothing, windowing) that can be
//! reused across plugins without introducing heavy abstractions.

pub mod delay;
pub mod eq;
pub mod filters;
pub mod level;
pub mod motion;
pub mod smoothing;
pub mod time;
pub mod window;

pub use delay::{DelayLine, FeedbackComb, FractionalDelayLine, StereoComb};
pub use filters::{BiquadCoeffs, BiquadState, OnePole, peaking_eq_coeffs, process_biquad};
pub use smoothing::{exp_smoothing_coeff, smooth_value};
pub use time::{hz_to_samples, ms_to_samples};
pub use window::{hann_window, overlap_normalization, overlap_normalization_checked};
