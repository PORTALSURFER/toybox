//! Contour point conversion and polyline emission utilities.

use super::super::sampling::clamp_sample;
use super::super::{Color, LaneBounds, Point, SurfaceCommand};

/// Convert sampled values into x/y contour points.
pub(super) fn samples_to_points(
    samples: &[f32],
    x_max: i32,
    lane: LaneBounds,
    center_y: i32,
    scale_y: f32,
    out: &mut Vec<Point>,
) {
    out.clear();
    if samples.is_empty() {
        return;
    }

    let points = samples.len();
    out.reserve(points);
    for (point_index, sample) in samples.iter().enumerate() {
        let x = point_x(point_index, points, x_max);
        let y = sample_to_lane_y(*sample, center_y, scale_y, lane);
        out.push(Point { x, y });
    }
}

/// Emit a polyline as a deterministic sequence of line commands.
pub(super) fn emit_polyline(commands: &mut Vec<SurfaceCommand>, points: &[Point], color: Color) {
    if points.len() < 2 || color.a == 0 {
        return;
    }

    for segment in points.windows(2) {
        if let [start, end] = segment {
            if start.x == end.x && start.y == end.y {
                continue;
            }
            commands.push(SurfaceCommand::Line {
                start: *start,
                end: *end,
                color,
            });
        }
    }
}

/// Emit one batched polyline command.
pub(super) fn emit_polyline_batched(
    commands: &mut Vec<SurfaceCommand>,
    points: &[Point],
    color: Color,
    thickness: f32,
) {
    if points.len() < 2 || color.a == 0 {
        return;
    }
    let compact = compact_polyline_points(points);
    if compact.len() < 2 {
        return;
    }
    commands.push(SurfaceCommand::Polyline {
        points: compact,
        thickness: thickness.max(1.0),
        color,
    });
}

/// Return sampled point count for one stride over a polyline.
pub(super) fn sampled_polyline_point_count(points: usize, stride: usize) -> usize {
    if points <= 1 {
        return points;
    }
    let stride = stride.max(1);
    ((points - 1) / stride) + 1
}

/// Map one point index to a rounded pixel-space x coordinate.
pub(super) fn point_x(point_index: usize, points: usize, x_max: i32) -> i32 {
    if points <= 1 || x_max <= 0 {
        return 0;
    }
    if points == (x_max + 1) as usize {
        return point_index as i32;
    }
    let denominator = (points - 1) as i64;
    let numerator = point_index as i64 * x_max as i64;
    ((numerator + denominator / 2) / denominator) as i32
}

/// Convert one normalized sample value into lane Y coordinates.
pub(super) fn sample_to_lane_y(sample: f32, center_y: i32, scale_y: f32, lane: LaneBounds) -> i32 {
    ((center_y as f32 - clamp_sample(sample) * scale_y).round() as i32).clamp(lane.top, lane.bottom)
}

/// Remove consecutive duplicate points while preserving deterministic order.
fn compact_polyline_points(points: &[Point]) -> Vec<Point> {
    let mut compact = Vec::with_capacity(points.len());
    for point in points {
        if compact.last().copied() != Some(*point) {
            compact.push(*point);
        }
    }
    compact
}
