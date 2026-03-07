/// Compute local plot geometry for one surface rectangle.
fn eq_surface_geometry(rect: Rect, padding_px: i32) -> Option<EqSurfaceGeometry> {
    let padding = padding_px.max(0);
    let max_x = rect.size.width as i32 - 1;
    let max_y = rect.size.height as i32 - 1;
    let left = padding;
    let top = padding;
    let right = (max_x - padding).max(left + 1);
    let bottom = (max_y - padding).max(top + 1);
    let width = (right - left).max(1) as f32;
    let height = (bottom - top).max(1) as f32;
    if width <= 0.0 || height <= 0.0 {
        return None;
    }
    Some(EqSurfaceGeometry {
        left,
        top,
        right,
        bottom,
        width,
        height,
    })
}

/// Return the attractor id currently hit by one local pointer position.
fn eq_hit_attractor(
    model: &EqAttractorSurfaceModel,
    geometry: EqSurfaceGeometry,
    pointer_local: Point,
) -> Option<u64> {
    let hit_radius2 = i64::from(EQ_SURFACE_ATTRACTOR_HIT_RADIUS_PX.pow(2));
    let mut best: Option<(u64, i64)> = None;
    for attractor in &model.attractors {
        let local = eq_normalized_to_local(attractor.x, attractor.y, geometry);
        let dx = i64::from(pointer_local.x - local.x);
        let dy = i64::from(pointer_local.y - local.y);
        let dist2 = dx * dx + dy * dy;
        if dist2 > hit_radius2 {
            continue;
        }
        match best {
            Some((_, best_dist2)) if dist2 >= best_dist2 => {}
            _ => best = Some((attractor.id, dist2)),
        }
    }
    best.map(|(id, _)| id)
}

/// Convert one local pointer coordinate to normalized attractor coordinates.
fn eq_pointer_to_normalized(pointer_local: Point, geometry: EqSurfaceGeometry) -> (f32, f32) {
    let x = ((pointer_local.x - geometry.left) as f32 / geometry.width).clamp(0.0, 1.0);
    let y = ((geometry.bottom - pointer_local.y) as f32 / geometry.height).clamp(0.0, 1.0);
    (x, y)
}

/// Convert normalized coordinates to local surface pixels.
fn eq_normalized_to_local(x: f32, y: f32, geometry: EqSurfaceGeometry) -> Point {
    Point {
        x: geometry.left + (geometry.width * x.clamp(0.0, 1.0)).round() as i32,
        y: geometry.bottom - (geometry.height * y.clamp(0.0, 1.0)).round() as i32,
    }
}

/// Convert normalized coordinates to local surface subpixel coordinates.
fn eq_normalized_to_local_f(x: f32, y: f32, geometry: EqSurfaceGeometry) -> crate::canvas::PointF {
    crate::canvas::PointF {
        x: geometry.left as f32 + geometry.width * x.clamp(0.0, 1.0),
        y: geometry.bottom as f32 - geometry.height * y.clamp(0.0, 1.0),
    }
}

/// Compute a smoothing coefficient from a time constant.
fn eq_smoothing_coeff(time_seconds: f32) -> f32 {
    (-EQ_SURFACE_FRAME_DT_SECONDS / time_seconds.max(0.001)).exp()
}

/// One-pole smoothing helper.
fn eq_smooth_value(current: f32, target: f32, coeff: f32) -> f32 {
    target + (current - target) * coeff.clamp(0.0, 1.0)
}

/// Compute normalized log-frequency interpolation value in `[0, 1]`.
fn eq_freq_to_t(freq_hz: f32, min_hz: f32, max_hz: f32) -> f32 {
    let min = min_hz.max(1.0);
    let max = max_hz.max(min + 1.0e-6);
    ((freq_hz.max(min).min(max).ln() - min.ln()) / (max.ln() - min.ln())).clamp(0.0, 1.0)
}

/// Sample one smooth band value from normalized band space.
fn eq_sample_band_value(values: &[f32], t: f32) -> f32 {
    if values.is_empty() {
        return 0.5;
    }
    if values.len() == 1 {
        return values[0].clamp(0.0, 1.0);
    }

    let scaled = t.clamp(0.0, 1.0) * (values.len() - 1) as f32;
    let index = scaled.floor() as isize;
    let local_t = scaled - index as f32;

    let p0 = eq_mirrored_value(values, index - 1);
    let p1 = eq_mirrored_value(values, index);
    let p2 = eq_mirrored_value(values, index + 1);
    let p3 = eq_mirrored_value(values, index + 2);
    eq_catmull_rom(p0, p1, p2, p3, local_t).clamp(0.0, 1.0)
}

/// Fetch one mirrored band value for edge-safe Catmull-Rom interpolation.
fn eq_mirrored_value(values: &[f32], index: isize) -> f32 {
    let max = (values.len() - 1) as isize;
    let mirrored = if index < 0 {
        (-index).min(max)
    } else if index > max {
        (max - (index - max)).max(0)
    } else {
        index
    } as usize;
    values[mirrored]
}

/// Catmull-Rom interpolation helper.
fn eq_catmull_rom(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    0.5 * ((2.0 * p1)
        + (-p0 + p2) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3)
}

/// Compute one local Gaussian attractor weight.
fn eq_attractor_weight(position: f32, center_x: f32, radius: f32) -> f32 {
    let sigma = radius.max(0.001);
    let distance = position.clamp(0.0, 1.0) - center_x.clamp(0.0, 1.0);
    (-0.5 * (distance / sigma).powi(2)).exp()
}

/// Warp one normalized band position toward an attractor center.
fn eq_gravity_warp_position(position: f32, center_x: f32, strength: f32, radius: f32) -> f32 {
    let position = position.clamp(0.0, 1.0);
    let center = center_x.clamp(0.0, 1.0);
    let strength = strength.max(0.0);
    if strength <= f32::EPSILON {
        return position;
    }
    let raw = eq_gravity_raw_displacement(position, center, strength, radius);
    let edge_lo = eq_gravity_raw_displacement(0.0, center, strength, radius);
    let edge_hi = eq_gravity_raw_displacement(1.0, center, strength, radius);
    let correction = edge_lo + (edge_hi - edge_lo) * position;
    (position + (raw - correction)).clamp(0.0, 1.0)
}

/// Blend multiple attractor pulls into one warped band position.
fn eq_blended_gravity_warp_position(
    position: f32,
    centers: &[f32],
    pulls: &[f32],
    radii: &[f32],
) -> f32 {
    if centers.is_empty() || pulls.is_empty() || radii.is_empty() {
        return position.clamp(0.0, 1.0);
    }
    let count = centers.len().min(pulls.len()).min(radii.len());
    let mut weighted_sum = 0.0;
    let mut weight_sum = 0.0;
    for index in 0..count {
        let pull = pulls[index].max(0.0);
        if pull <= f32::EPSILON {
            continue;
        }
        let radius = radii[index].max(0.001);
        let warped = eq_gravity_warp_position(position, centers[index], pull, radius);
        let local_weight = eq_attractor_weight(position, centers[index], radius) * pull;
        if local_weight <= f32::EPSILON {
            continue;
        }
        weighted_sum += warped * local_weight;
        weight_sum += local_weight;
    }
    if weight_sum <= f32::EPSILON {
        position.clamp(0.0, 1.0)
    } else {
        (weighted_sum / weight_sum).clamp(0.0, 1.0)
    }
}

/// Sample one shared moving wave before any attractor wells deform it.
fn eq_base_wave(position: f32, phase_token: f32, wave_cycles: f32, wave_depth: f32) -> f32 {
    (phase_token + std::f32::consts::TAU * wave_cycles.max(0.0) * position).sin()
        * wave_depth.clamp(0.0, 1.0)
}

/// Sample one shared wave deformed by static attractor wells.
fn eq_wave_sample(
    position: f32,
    phase_token: f32,
    wave_cycles: f32,
    wave_depth: f32,
    centers: &[f32],
    targets: &[f32],
    pulls: &[f32],
    radii: &[f32],
) -> f32 {
    let count = centers
        .len()
        .min(targets.len())
        .min(pulls.len())
        .min(radii.len());
    let position = position.clamp(0.0, 1.0);
    if count == 0 {
        return eq_base_wave(position, phase_token, wave_cycles, wave_depth);
    }

    let mut weights = Vec::with_capacity(count);
    let mut weight_sum = 0.0;
    for index in 0..count {
        let weight =
            eq_attractor_weight(position, centers[index], radii[index]) * pulls[index].max(0.0);
        weights.push(weight);
        weight_sum += weight;
    }

    let warped = eq_blended_gravity_warp_position(
        position,
        &centers[..count],
        &pulls[..count],
        &radii[..count],
    );
    let anchor = eq_weighted_average(&centers[..count], &weights, position);
    let influence = eq_gravity_local_influence(weight_sum);
    let slowed = eq_gravity_slowed_position(warped, anchor, influence);
    let base = eq_base_wave(slowed, phase_token, wave_cycles, wave_depth);
    let target = eq_weighted_average(&targets[..count], &weights, base);

    (base + (target - base) * influence).clamp(-1.0, 1.0)
}

/// Compute raw gravity displacement before endpoint correction.
fn eq_gravity_raw_displacement(position: f32, center_x: f32, strength: f32, radius: f32) -> f32 {
    let sigma = radius.max(0.001);
    let distance = position - center_x;
    let pull = (center_x - position) * strength;
    pull * (-0.5 * (distance / sigma).powi(2)).exp()
}

/// Collapse one local gravity weight sum into a stable `0..=1` influence value.
fn eq_gravity_local_influence(weight_sum: f32) -> f32 {
    (1.0 - (-weight_sum.max(0.0)).exp()).clamp(0.0, 1.0)
}

/// Compress a warped position toward one local gravity anchor.
fn eq_gravity_slowed_position(position: f32, anchor: f32, influence: f32) -> f32 {
    let slowdown = (1.0 - influence.clamp(0.0, 1.0) * 0.65).clamp(0.2, 1.0);
    (anchor + (position - anchor) * slowdown).clamp(0.0, 1.0)
}

/// Compute a weighted scalar average with fallback when all weights are zero.
fn eq_weighted_average(values: &[f32], weights: &[f32], fallback: f32) -> f32 {
    let count = values.len().min(weights.len());
    let mut weighted_sum = 0.0;
    let mut weight_total = 0.0;
    for index in 0..count {
        let weight = weights[index].max(0.0);
        if weight <= f32::EPSILON {
            continue;
        }
        weighted_sum += values[index] * weight;
        weight_total += weight;
    }
    if weight_total <= f32::EPSILON {
        fallback
    } else {
        weighted_sum / weight_total
    }
}

/// Return `color` with alpha multiplied by `alpha` in `0..=255`.
fn eq_scale_alpha(color: Color, alpha: u8) -> Color {
    let scaled = (u16::from(color.a) * u16::from(alpha) + 127) / 255;
    Color::rgba(color.r, color.g, color.b, scaled as u8)
}

#[cfg(test)]
mod eq_surface_tests {
    use super::*;

    #[test]
    fn pointer_mapping_clamps_to_normalized_bounds() {
        let geometry = EqSurfaceGeometry {
            left: 4,
            top: 6,
            right: 104,
            bottom: 206,
            width: 100.0,
            height: 200.0,
        };
        assert_eq!(
            eq_pointer_to_normalized(Point { x: -100, y: 999 }, geometry),
            (0.0, 0.0)
        );
        assert_eq!(
            eq_pointer_to_normalized(Point { x: 999, y: -100 }, geometry),
            (1.0, 1.0)
        );
    }

    #[test]
    fn attractor_hit_chooses_closest_candidate() {
        let model = EqAttractorSurfaceModel::new(vec![
            EqAttractor::new(11, 0.2, 0.2),
            EqAttractor::new(22, 0.8, 0.8),
        ]);
        let geometry = EqSurfaceGeometry {
            left: 0,
            top: 0,
            right: 100,
            bottom: 100,
            width: 100.0,
            height: 100.0,
        };
        let hit = eq_hit_attractor(&model, geometry, Point { x: 22, y: 80 });
        assert_eq!(hit, Some(11));
    }

    #[test]
    fn smoothing_coeff_is_stable_and_bounded() {
        let coeff = eq_smoothing_coeff(0.08);
        assert!(coeff > 0.0 && coeff < 1.0);
        let snapped = eq_smooth_value(0.0, 1.0, coeff);
        assert!(snapped > 0.0 && snapped < 1.0);
    }

    #[test]
    fn sample_band_value_handles_edges() {
        let values = vec![0.1, 0.4, 0.9];
        assert!((eq_sample_band_value(&values, 0.0) - 0.1).abs() < 1.0e-6);
        assert!((eq_sample_band_value(&values, 1.0) - 0.9).abs() < 1.0e-6);
    }

    #[test]
    fn wave_without_attractors_matches_base_wave() {
        let position = 0.3;
        let phase = 0.8;

        assert!(
            (eq_wave_sample(position, phase, 1.25, 0.75, &[], &[], &[], &[])
                - (phase + std::f32::consts::TAU * 1.25 * position).sin() * 0.75)
                .abs()
                < 1.0e-6
        );
    }

    #[test]
    fn attractor_well_pulls_wave_toward_target_y() {
        let sample = eq_wave_sample(0.5, 0.0, 1.0, 1.0, &[0.5], &[0.8], &[2.0], &[0.12]);

        assert!(sample > 0.2, "expected the well to pull the wave upward");
    }
}
