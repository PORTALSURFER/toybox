/// Minimum supported curve segment tension.
pub const CURVE_SEGMENT_TENSION_MIN: f32 = -1.0;
/// Maximum supported curve segment tension.
pub const CURVE_SEGMENT_TENSION_MAX: f32 = 1.0;

/// X-distance epsilon used while deduplicating near-identical points.
const CURVE_POINT_X_EPSILON: f32 = 1.0e-4;

/// One normalized control point for a curve editor model.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CurvePoint {
    /// Normalized x position in `[0, 1]`.
    pub x: f32,
    /// Normalized y value in `[0, 1]`.
    pub y: f32,
}

impl CurvePoint {
    /// Build one curve point with explicit normalized coordinates.
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// One segment descriptor connecting adjacent curve points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CurveSegment {
    /// Segment curvature amount in `[-1, 1]`.
    pub tension: f32,
}

impl CurveSegment {
    /// Build one segment descriptor.
    pub const fn new(tension: f32) -> Self {
        Self { tension }
    }
}

/// Editable point/segment curve model consumed by curve-editor widgets.
#[derive(Clone, Debug, PartialEq)]
pub struct CurveModel {
    /// Ordered points from left (`x = 0`) to right (`x = 1`).
    pub points: Vec<CurvePoint>,
    /// Segment settings where `segments[i]` connects `points[i] -> points[i + 1]`.
    pub segments: Vec<CurveSegment>,
    /// Optional unwrapped source used for exact cyclic phase translation.
    #[doc(hidden)]
    pub phase_source: Option<Box<CurveModel>>,
    /// Phase offset applied to `phase_source`, in normalized cycles.
    #[doc(hidden)]
    pub phase_offset: f32,
}

impl CurveModel {
    /// Build one curve model and normalize it for safe interaction/rendering.
    pub fn new(points: Vec<CurvePoint>, segments: Vec<CurveSegment>) -> Self {
        Self {
            points,
            segments,
            phase_source: None,
            phase_offset: 0.0,
        }
        .normalized()
    }

    /// Return a normalized copy with repaired topology and clamped values.
    pub fn normalized(mut self) -> Self {
        self.normalize_in_place();
        self
    }

    /// Normalize this model in place.
    pub fn normalize_in_place(&mut self) {
        self.points = normalize_points(&self.points);
        self.segments = normalize_segments(&self.segments, self.points.len().saturating_sub(1));
        if let Some(source) = self.phase_source.as_mut() {
            source.normalize_in_place();
        }
        self.phase_offset = if self.phase_source.is_some() {
            self.phase_offset.rem_euclid(1.0)
        } else {
            0.0
        };
    }

    /// Drop exact phase-translation metadata before an ordinary edit.
    pub fn clear_phase_metadata(&mut self) {
        self.phase_source = None;
        self.phase_offset = 0.0;
    }

    /// Sample the curve at normalized `x` in `[0, 1]`.
    pub fn sample(&self, x: f32) -> f32 {
        if let Some(source) = self.phase_source.as_deref() {
            let relative = (x - self.phase_offset).rem_euclid(1.0);
            let relative = if relative <= CURVE_POINT_X_EPSILON
                || 1.0 - relative <= CURVE_POINT_X_EPSILON
            {
                0.0
            } else {
                relative
            };
            return source.sample(relative);
        }
        if self.points.len() < 2 {
            return 1.0;
        }
        let clamped_x = x.clamp(0.0, 1.0);
        if clamped_x <= self.points[0].x {
            return self.points[0].y.clamp(0.0, 1.0);
        }
        if clamped_x >= self.points[self.points.len() - 1].x {
            return self.points[self.points.len() - 1].y.clamp(0.0, 1.0);
        }

        let mut segment_index = 0usize;
        while segment_index + 1 < self.points.len() && clamped_x > self.points[segment_index + 1].x
        {
            segment_index += 1;
        }
        sample_segment(self, segment_index, clamped_x)
    }
}

/// Sample one segment from a normalized model.
fn sample_segment(model: &CurveModel, segment_index: usize, x: f32) -> f32 {
    let left = model.points[segment_index];
    let right = model.points[(segment_index + 1).min(model.points.len().saturating_sub(1))];
    let span = (right.x - left.x).max(1.0e-6);
    let local = ((x - left.x) / span).clamp(0.0, 1.0);
    let shaped = shape_with_tension(
        local,
        model
            .segments
            .get(segment_index)
            .copied()
            .unwrap_or(CurveSegment { tension: 0.0 })
            .tension,
    );
    lerp(left.y, right.y, shaped).clamp(0.0, 1.0)
}

/// Apply segment tension shaping to one local interpolation value.
fn shape_with_tension(value: f32, tension: f32) -> f32 {
    let v = value.clamp(0.0, 1.0);
    let t = tension.clamp(CURVE_SEGMENT_TENSION_MIN, CURVE_SEGMENT_TENSION_MAX);
    if t >= 0.0 {
        v.powf(1.0 + t * 3.0)
    } else {
        1.0 - (1.0 - v).powf(1.0 + (-t) * 3.0)
    }
}

/// Normalize points into a valid, sorted, finite topology.
fn normalize_points(points: &[CurvePoint]) -> Vec<CurvePoint> {
    let mut normalized: Vec<CurvePoint> = points
        .iter()
        .copied()
        .filter(|point| point.x.is_finite() && point.y.is_finite())
        .map(|point| CurvePoint {
            x: point.x.clamp(0.0, 1.0),
            y: point.y.clamp(0.0, 1.0),
        })
        .collect();

    if normalized.is_empty() {
        return vec![CurvePoint { x: 0.0, y: 1.0 }, CurvePoint { x: 1.0, y: 1.0 }];
    }

    normalized.sort_by(|left, right| left.x.total_cmp(&right.x));

    let mut deduped: Vec<CurvePoint> = Vec::with_capacity(normalized.len());
    for point in normalized {
        if let Some(last) = deduped.last_mut()
            && (point.x - last.x).abs() < CURVE_POINT_X_EPSILON
        {
            last.y = point.y;
            continue;
        }
        deduped.push(point);
    }

    if deduped.len() == 1 {
        let only = deduped[0];
        deduped.push(CurvePoint {
            x: if only.x < 0.5 { 1.0 } else { 0.0 },
            y: only.y,
        });
        deduped.sort_by(|left, right| left.x.total_cmp(&right.x));
    }

    deduped[0].x = 0.0;
    let last = deduped.len().saturating_sub(1);
    deduped[last].x = 1.0;
    for index in 1..=last {
        let min_x = deduped[index - 1].x + CURVE_POINT_X_EPSILON;
        if deduped[index].x < min_x {
            deduped[index].x = min_x;
        }
    }
    deduped[last].x = 1.0;
    for index in (0..last).rev() {
        let max_x = deduped[index + 1].x - CURVE_POINT_X_EPSILON;
        if deduped[index].x > max_x {
            deduped[index].x = max_x;
        }
    }
    deduped[0].x = 0.0;
    deduped
}

/// Normalize segment vector length and clamp all tension values.
fn normalize_segments(segments: &[CurveSegment], target_len: usize) -> Vec<CurveSegment> {
    let mut normalized = Vec::with_capacity(target_len);
    for index in 0..target_len {
        let tension = segments
            .get(index)
            .copied()
            .unwrap_or(CurveSegment { tension: 0.0 })
            .tension
            .clamp(CURVE_SEGMENT_TENSION_MIN, CURVE_SEGMENT_TENSION_MAX);
        normalized.push(CurveSegment { tension });
    }
    normalized
}

/// Linear interpolation helper.
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
