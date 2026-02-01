//! Gain and level conversion helpers.

/// Convert a decibel value to linear gain.
pub fn db_to_linear(db: f32) -> f32 {
    (10.0_f32).powf(db / 20.0)
}

/// Convert a linear gain value to decibels.
///
/// `floor_db` is used when the linear value is too small to represent.
pub fn linear_to_db(linear: f32, floor_db: f32) -> f32 {
    if linear <= 0.0 {
        floor_db
    } else {
        20.0 * linear.log10()
    }
}

#[cfg(test)]
mod tests {
    use super::{db_to_linear, linear_to_db};

    #[test]
    fn db_round_trip_is_reasonable() {
        let db = -6.0;
        let linear = db_to_linear(db);
        let back = linear_to_db(linear, -120.0);
        assert!((back - db).abs() < 1e-3);
    }
}
