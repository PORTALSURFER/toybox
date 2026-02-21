//! Generic vector shape rendering helpers.

use vello::Scene;
use vello::kurbo::{Affine, BezPath, Circle, Line, Rect as KurboRect, Stroke};
use vello::peniko::Fill;

use super::color_and_angle_helpers::color_to_vello;
use super::types::{
    CircleStrokeVisual, CircleVisual, LineVisual, PolylineVisual, RectStrokeVisual, RectVisual,
};

/// Draw one filled rectangle.
pub(super) fn draw_rect_fill(scene: &mut Scene, rect: RectVisual, transform: Affine) {
    let width = rect.rect.size.width.max(1) as f64;
    let height = rect.rect.size.height.max(1) as f64;
    let shape = KurboRect::new(
        rect.rect.origin.x as f64,
        rect.rect.origin.y as f64,
        rect.rect.origin.x as f64 + width,
        rect.rect.origin.y as f64 + height,
    );
    scene.fill(
        Fill::NonZero,
        transform,
        color_to_vello(rect.color),
        None,
        &shape,
    );
}

/// Draw one stroked rectangle.
pub(super) fn draw_rect_stroke(scene: &mut Scene, rect: RectStrokeVisual, transform: Affine) {
    let width = rect.rect.size.width.max(1) as f64;
    let height = rect.rect.size.height.max(1) as f64;
    let shape = KurboRect::new(
        rect.rect.origin.x as f64,
        rect.rect.origin.y as f64,
        rect.rect.origin.x as f64 + width,
        rect.rect.origin.y as f64 + height,
    );
    scene.stroke(
        &Stroke::new(rect.thickness.max(1.0) as f64),
        transform,
        color_to_vello(rect.color),
        None,
        &shape,
    );
}

/// Draw one stroked line.
pub(super) fn draw_line_stroke(scene: &mut Scene, line: LineVisual, transform: Affine) {
    let shape = Line::new(
        vello::kurbo::Point::new(line.start.x as f64, line.start.y as f64),
        vello::kurbo::Point::new(line.end.x as f64, line.end.y as f64),
    );
    scene.stroke(
        &Stroke::new(line.thickness.max(1.0) as f64),
        transform,
        color_to_vello(line.color),
        None,
        &shape,
    );
}

/// Draw one stroked polyline.
pub(super) fn draw_polyline_stroke(
    scene: &mut Scene,
    polyline: &PolylineVisual,
    transform: Affine,
) {
    if polyline.points.len() < 2 {
        return;
    }
    let mut path = BezPath::new();
    let first = polyline.points[0];
    path.move_to(vello::kurbo::Point::new(first.x as f64, first.y as f64));
    for point in polyline.points.iter().skip(1) {
        path.line_to(vello::kurbo::Point::new(point.x as f64, point.y as f64));
    }
    scene.stroke(
        &Stroke::new(polyline.thickness.max(1.0) as f64),
        transform,
        color_to_vello(polyline.color),
        None,
        &path,
    );
}

/// Draw one filled circle.
pub(super) fn draw_circle_fill(scene: &mut Scene, circle: CircleVisual, transform: Affine) {
    let shape = Circle::new(
        vello::kurbo::Point::new(circle.center.x as f64, circle.center.y as f64),
        circle.radius.max(1.0) as f64,
    );
    scene.fill(
        Fill::NonZero,
        transform,
        color_to_vello(circle.color),
        None,
        &shape,
    );
}

/// Draw one stroked circle.
pub(super) fn draw_circle_stroke(scene: &mut Scene, circle: CircleStrokeVisual, transform: Affine) {
    let shape = Circle::new(
        vello::kurbo::Point::new(circle.center.x as f64, circle.center.y as f64),
        circle.radius.max(1.0) as f64,
    );
    scene.stroke(
        &Stroke::new(circle.thickness.max(1.0) as f64),
        transform,
        color_to_vello(circle.color),
        None,
        &shape,
    );
}
