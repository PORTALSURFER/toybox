//! Knob vector primitive rendering helpers.

use crate::canvas::Point;
use vello::Scene;
use vello::kurbo::{Affine, BezPath, Circle, Line, Point as KurboPoint, Stroke};
use vello::peniko::Fill;

use super::color_and_angle_helpers::{color_to_vello, normalize_angle};
use super::types::KnobVisual;

/// Emit vector geometry for a knob visual payload.
pub(super) fn draw_knob(scene: &mut Scene, knob: KnobVisual, transform: Affine) {
    draw_knob_body(scene, knob, transform);
    draw_knob_outline_ring(scene, knob, transform);
    draw_knob_active_arc(scene, knob, transform);
    draw_knob_indicator_line(scene, knob, transform);
}

/// Draw the knob circular body fill and outline.
fn draw_knob_body(scene: &mut Scene, knob: KnobVisual, transform: Affine) {
    let center = KurboPoint::new(knob.center.x as f64, knob.center.y as f64);
    let body = Circle::new(center, knob.radius.max(1) as f64);
    scene.fill(
        Fill::NonZero,
        transform,
        color_to_vello(knob.fill),
        None,
        &body,
    );
    scene.stroke(
        &Stroke::new(2.0),
        transform,
        color_to_vello(knob.outline),
        None,
        &body,
    );
}

/// Draw the full knob ring arc.
fn draw_knob_outline_ring(scene: &mut Scene, knob: KnobVisual, transform: Affine) {
    let ring = arc_path(
        knob.center,
        knob.arc_radius.max(1) as f32,
        knob.arc_start,
        knob.arc_end,
    );
    scene.stroke(
        &Stroke::new(knob.arc_thickness.max(1) as f64),
        transform,
        color_to_vello(knob.outline),
        None,
        &ring,
    );
}

/// Draw the active-value arc segment.
fn draw_knob_active_arc(scene: &mut Scene, knob: KnobVisual, transform: Affine) {
    let active = arc_path(
        knob.center,
        knob.arc_radius.max(1) as f32,
        knob.value_angle,
        knob.arc_end,
    );
    scene.stroke(
        &Stroke::new(knob.arc_thickness.max(1) as f64),
        transform,
        color_to_vello(knob.indicator),
        None,
        &active,
    );
}

/// Draw the center-to-indicator line for the current knob value.
fn draw_knob_indicator_line(scene: &mut Scene, knob: KnobVisual, transform: Affine) {
    let tip = indicator_point(knob.center, knob.radius, knob.value_angle);
    let line = Line::new(
        KurboPoint::new(knob.center.x as f64, knob.center.y as f64),
        KurboPoint::new(tip.x as f64, tip.y as f64),
    );
    scene.stroke(
        &Stroke::new(2.0),
        transform,
        color_to_vello(knob.indicator),
        None,
        &line,
    );
}

/// Build a polyline arc path in UI coordinate space.
fn arc_path(center: Point, radius: f32, start_angle: f32, end_angle: f32) -> BezPath {
    let mut start = normalize_angle(start_angle);
    let mut end = normalize_angle(end_angle);
    if (start - end).abs() < f32::EPSILON {
        return BezPath::new();
    }
    if end < start {
        end += std::f32::consts::TAU;
    }
    if start > end {
        std::mem::swap(&mut start, &mut end);
    }
    let span = (end - start).abs();
    let segments = ((span * radius.max(1.0) * 0.2).ceil() as usize).clamp(8, 96);
    let step = span / segments as f32;

    let mut path = BezPath::new();
    for idx in 0..=segments {
        let angle = start + step * idx as f32;
        let x = center.x as f32 + radius * angle.cos();
        let y = center.y as f32 - radius * angle.sin();
        let point = KurboPoint::new(x as f64, y as f64);
        if idx == 0 {
            path.move_to(point);
        } else {
            path.line_to(point);
        }
    }
    path
}

/// Resolve the indicator endpoint for a knob angle and radius.
fn indicator_point(center: Point, radius: i32, angle: f32) -> Point {
    Point {
        x: center.x + (radius as f32 * angle.cos()) as i32,
        y: center.y - (radius as f32 * angle.sin()) as i32,
    }
}
