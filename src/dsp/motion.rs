//! Motion and modulation helpers shared across plugins.

/// Warp a normalized position toward low or high values.
///
/// `position` and `warp` are expected in the 0..=1 range. A warp of 0.5 leaves
/// the position unchanged; lower values skew toward low positions and higher
/// values skew toward high positions.
pub fn warp_position(position: f32, warp: f32) -> f32 {
    let warp_signed = (warp - 0.5) * 2.0;
    if warp_signed.abs() < f32::EPSILON {
        return position;
    }
    let exp = 1.0 + warp_signed.abs() * 2.0;
    if warp_signed > 0.0 {
        position.powf(exp)
    } else {
        1.0 - (1.0 - position).powf(exp)
    }
}

/// Convert a normalized position into a direction multiplier.
///
/// When `reverse` is true, the direction is inverted.
pub fn direction_from_position(position: f32, reverse: bool) -> f32 {
    let mut direction = -((position.clamp(0.0, 1.0) - 0.5) * 2.0);
    if reverse {
        direction = -direction;
    }
    direction
}

#[cfg(test)]
mod tests {
    use super::{direction_from_position, warp_position};

    #[test]
    fn warp_position_is_identity_at_center() {
        assert!((warp_position(0.25, 0.5) - 0.25).abs() < 1e-6);
        assert!((warp_position(0.75, 0.5) - 0.75).abs() < 1e-6);
    }

    #[test]
    fn warp_position_skews_endpoints() {
        let low = warp_position(0.25, 0.0);
        let high = warp_position(0.25, 1.0);
        assert!(low > 0.25);
        assert!(high < 0.25);
    }

    #[test]
    fn direction_from_position_handles_reverse() {
        let forward = direction_from_position(0.25, false);
        let reversed = direction_from_position(0.25, true);
        assert_eq!(forward, -reversed);
    }
}
