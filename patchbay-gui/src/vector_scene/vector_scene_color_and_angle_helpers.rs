//! Shared color and angle helpers for vector rendering.

use crate::canvas::Color;
use vello::peniko::Color as VelloColor;

/// Normalize an angle in radians to the `[0, TAU)` domain.
pub(super) fn normalize_angle(angle: f32) -> f32 {
    let mut normalized = angle % std::f32::consts::TAU;
    if normalized < 0.0 {
        normalized += std::f32::consts::TAU;
    }
    normalized
}

/// Convert a canvas color into a Vello color.
pub(super) fn color_to_vello(color: Color) -> VelloColor {
    VelloColor::from_rgba8(color.r, color.g, color.b, color.a)
}
