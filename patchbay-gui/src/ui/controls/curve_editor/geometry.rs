/// Base vertical drag scale used for segment tension interaction.
const CURVE_TENSION_PIXEL_SCALE: f32 = 160.0;

/// Return one normalized point from rectangle-local pointer coordinates.
fn curve_point_from_local(local: Point, rect: Rect) -> crate::declarative::CurvePoint {
    let width = (rect.size.width.max(1) as f32 - 1.0).max(1.0);
    let height = (rect.size.height.max(1) as f32 - 1.0).max(1.0);
    let x = (local.x as f32 / width).clamp(0.0, 1.0);
    let y = (1.0 - (local.y as f32 / height)).clamp(0.0, 1.0);
    crate::declarative::CurvePoint { x, y }
}

/// Return one rectangle-local point from normalized curve coordinates.
fn local_from_curve_point(point: crate::declarative::CurvePoint, rect: Rect) -> Point {
    let width = rect.size.width.max(1) as f32 - 1.0;
    let height = rect.size.height.max(1) as f32 - 1.0;
    let x = (point.x.clamp(0.0, 1.0) * width).round() as i32;
    let y = ((1.0 - point.y.clamp(0.0, 1.0)) * height).round() as i32;
    Point { x, y }
}

/// Return the squared distance between two points.
fn distance_squared(left: Point, right: Point) -> i64 {
    let dx = i64::from(left.x - right.x);
    let dy = i64::from(left.y - right.y);
    dx * dx + dy * dy
}

/// Return one sampled y value for a normalized x position.
fn sample_curve_model(model: &crate::declarative::CurveModel, x: f32) -> f32 {
    model.sample(x)
}

/// Return the closest point hit within one radius threshold.
fn find_point_hit_within(
    model: &crate::declarative::CurveModel,
    local_pointer: Point,
    radius: i32,
    rect: Rect,
) -> Option<usize> {
    let mut best: Option<(usize, i64)> = None;
    let radius_squared = i64::from(radius.max(0) * radius.max(0));
    for (index, point) in model.points.iter().copied().enumerate() {
        let center = local_from_curve_point(point, rect);
        let distance = distance_squared(center, local_pointer);
        if distance <= radius_squared {
            match best {
                Some((_, best_distance)) if distance >= best_distance => {}
                _ => best = Some((index, distance)),
            }
        }
    }
    best.map(|(index, _)| index)
}

/// Return the nearest segment hit within one radius threshold.
fn find_segment_hit_within(
    model: &crate::declarative::CurveModel,
    local_pointer: Point,
    radius: i32,
    rect: Rect,
) -> Option<usize> {
    let mut best: Option<(usize, f32)> = None;
    let radius_squared = (radius.max(0) * radius.max(0)) as f32;
    for segment_index in 0..model.segments.len() {
        let distance = segment_polyline_distance_squared(model, segment_index, local_pointer, rect);
        if distance <= radius_squared {
            match best {
                Some((_, best_distance)) if distance >= best_distance => {}
                _ => best = Some((segment_index, distance)),
            }
        }
    }
    best.map(|(index, _)| index)
}

/// Return the distance from pointer to one segment polyline approximation.
fn segment_polyline_distance_squared(
    model: &crate::declarative::CurveModel,
    segment_index: usize,
    local_pointer: Point,
    rect: Rect,
) -> f32 {
    if segment_index >= model.segments.len() || segment_index + 1 >= model.points.len() {
        return f32::MAX;
    }
    let left = model.points[segment_index];
    let right = model.points[segment_index + 1];
    let left_local = local_from_curve_point(left, rect);
    let right_local = local_from_curve_point(right, rect);
    let segment_width = (right_local.x - left_local.x).abs().max(2);
    let steps = segment_width.clamp(2, 128) as usize;
    let mut best = f32::MAX;
    let mut previous = Point {
        x: left_local.x,
        y: local_from_curve_point(
            crate::declarative::CurvePoint {
                x: left.x,
                y: sample_curve_model(model, left.x),
            },
            rect,
        )
        .y,
    };
    for step in 1..=steps {
        let t = step as f32 / steps as f32;
        let x = left.x + (right.x - left.x) * t;
        let y = sample_curve_model(model, x);
        let point = local_from_curve_point(crate::declarative::CurvePoint { x, y }, rect);
        let distance = point_to_segment_distance_squared(local_pointer, previous, point);
        if distance < best {
            best = distance;
        }
        previous = point;
    }
    best
}

/// Return distance squared from one point to one segment.
fn point_to_segment_distance_squared(point: Point, start: Point, end: Point) -> f32 {
    let px = point.x as f32;
    let py = point.y as f32;
    let x1 = start.x as f32;
    let y1 = start.y as f32;
    let x2 = end.x as f32;
    let y2 = end.y as f32;
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len2 = dx * dx + dy * dy;
    if len2 <= f32::EPSILON {
        let ddx = px - x1;
        let ddy = py - y1;
        return ddx * ddx + ddy * ddy;
    }
    let t = (((px - x1) * dx + (py - y1) * dy) / len2).clamp(0.0, 1.0);
    let proj_x = x1 + t * dx;
    let proj_y = y1 + t * dy;
    let ddx = px - proj_x;
    let ddy = py - proj_y;
    ddx * ddx + ddy * ddy
}

/// Return one preview point snapped to the current curve under pointer x.
fn preview_point_on_curve(
    model: &crate::declarative::CurveModel,
    local_pointer: Point,
    rect: Rect,
) -> Option<crate::declarative::CurvePoint> {
    if model.points.len() < 2 {
        return None;
    }
    let pointer = curve_point_from_local(local_pointer, rect);
    Some(crate::declarative::CurvePoint {
        x: pointer.x,
        y: sample_curve_model(model, pointer.x),
    })
}

/// Insert one point in sorted order while preserving segment topology.
fn insert_point(
    model: &mut crate::declarative::CurveModel,
    point: crate::declarative::CurvePoint,
    max_points: usize,
    min_spacing_x: f32,
) -> usize {
    if model.points.len() >= max_points.max(2) {
        return find_nearest_point_index(model, point);
    }
    let mut insert_at = model.points.partition_point(|existing| existing.x < point.x);
    insert_at = insert_at.clamp(1, model.points.len().saturating_sub(1));
    let min_x = model.points[insert_at - 1].x + min_spacing_x;
    let max_x = model.points[insert_at].x - min_spacing_x;
    if min_x >= max_x {
        return insert_at.saturating_sub(1);
    }
    let clamped = crate::declarative::CurvePoint {
        x: point.x.clamp(min_x, max_x),
        y: point.y.clamp(0.0, 1.0),
    };
    model.points.insert(insert_at, clamped);
    let inherited = model
        .segments
        .get(insert_at.saturating_sub(1))
        .copied()
        .unwrap_or(crate::declarative::CurveSegment { tension: 0.0 });
    model.segments.insert(insert_at.saturating_sub(1), inherited);
    insert_at
}

/// Return the nearest point index to one normalized point.
fn find_nearest_point_index(
    model: &crate::declarative::CurveModel,
    point: crate::declarative::CurvePoint,
) -> usize {
    model
        .points
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| {
            let left_dist = (left.x - point.x).abs() + (left.y - point.y).abs();
            let right_dist = (right.x - point.x).abs() + (right.y - point.y).abs();
            left_dist.total_cmp(&right_dist)
        })
        .map(|(index, _)| index)
        .unwrap_or(0)
}

/// Remove one interior point and collapse adjacent segment metadata.
fn remove_interior_point(model: &mut crate::declarative::CurveModel, index: usize) {
    if index == 0 || index >= model.points.len().saturating_sub(1) {
        return;
    }
    model.points.remove(index);
    if !model.segments.is_empty() {
        let remove_segment = index
            .saturating_sub(1)
            .min(model.segments.len().saturating_sub(1));
        model.segments.remove(remove_segment);
    }
}

/// Apply endpoint coupling mode after a model mutation.
fn enforce_endpoint_mode(
    model: &mut crate::declarative::CurveModel,
    mode: crate::declarative::EndpointMode,
) {
    if matches!(mode, crate::declarative::EndpointMode::CoupledY) && model.points.len() >= 2 {
        let last = model.points.len().saturating_sub(1);
        model.points[last].y = model.points[0].y;
    }
}

/// Recompute move-point drag output from one drag-origin snapshot.
fn recompute_move_point_from_origin(
    origin_model: &crate::declarative::CurveModel,
    origin_index: usize,
    target: crate::declarative::CurvePoint,
    min_spacing_x: f32,
    push_through_threshold_px: i32,
    rect: Rect,
    endpoint_mode: crate::declarative::EndpointMode,
) -> (crate::declarative::CurveModel, usize) {
    let mut recomputed = origin_model.clone();
    let moved_index = move_point_with_push_through(
        &mut recomputed,
        origin_index,
        target,
        min_spacing_x,
        push_through_threshold_px,
        rect,
        endpoint_mode,
    );
    (recomputed, moved_index)
}

/// Move one point with push-through deletion semantics.
fn move_point_with_push_through(
    model: &mut crate::declarative::CurveModel,
    index: usize,
    target: crate::declarative::CurvePoint,
    min_spacing_x: f32,
    push_through_threshold_px: i32,
    rect: Rect,
    endpoint_mode: crate::declarative::EndpointMode,
) -> usize {
    if index >= model.points.len() {
        return index;
    }
    let y = target.y.clamp(0.0, 1.0);
    let last = model.points.len().saturating_sub(1);
    if index == 0 {
        model.points[0].y = y;
        enforce_endpoint_mode(model, endpoint_mode);
        return 0;
    }
    if index == last {
        model.points[last].y = y;
        enforce_endpoint_mode(model, endpoint_mode);
        return last;
    }

    let mut moved_index = index;
    let threshold_x = push_through_threshold_px.max(0) as f32 / (rect.size.width.max(2) - 1) as f32;
    while moved_index + 1 < model.points.len().saturating_sub(1)
        && target.x > model.points[moved_index + 1].x + threshold_x
    {
        remove_interior_point(model, moved_index + 1);
    }
    while moved_index > 1 && target.x < model.points[moved_index - 1].x - threshold_x {
        remove_interior_point(model, moved_index - 1);
        moved_index = moved_index.saturating_sub(1);
    }

    let min_x = model.points[moved_index - 1].x + min_spacing_x;
    let max_x = model.points[moved_index + 1].x - min_spacing_x;
    model.points[moved_index].x = target.x.clamp(min_x, max_x);
    model.points[moved_index].y = y;
    enforce_endpoint_mode(model, endpoint_mode);
    moved_index
}

/// Move one segment by translating both endpoint points.
fn move_segment_translated(
    model: &mut crate::declarative::CurveModel,
    segment_index: usize,
    start_left: (f32, f32),
    start_right: (f32, f32),
    delta: (f32, f32),
    min_spacing_x: f32,
    endpoint_mode: crate::declarative::EndpointMode,
) {
    let (start_left_x, start_left_y) = start_left;
    let (start_right_x, start_right_y) = start_right;
    let (delta_x, delta_y) = delta;
    if model.points.len() < 2 || segment_index >= model.points.len().saturating_sub(1) {
        return;
    }
    let right_index = segment_index + 1;
    let mut applied_dx = delta_x;
    if segment_index == 0 || right_index == model.points.len().saturating_sub(1) {
        applied_dx = 0.0;
    } else {
        let min_dx = model.points[segment_index - 1].x + min_spacing_x - start_left_x;
        let max_dx = model.points[right_index + 1].x - min_spacing_x - start_right_x;
        applied_dx = applied_dx.clamp(min_dx, max_dx);
    }
    model.points[segment_index].x = (start_left_x + applied_dx).clamp(0.0, 1.0);
    model.points[right_index].x = (start_right_x + applied_dx).clamp(0.0, 1.0);
    model.points[segment_index].y = (start_left_y + delta_y).clamp(0.0, 1.0);
    model.points[right_index].y = (start_right_y + delta_y).clamp(0.0, 1.0);
    enforce_endpoint_mode(model, endpoint_mode);
}

/// Return signed tension delta from one vertical drag.
fn tension_delta_from_drag(
    model: &crate::declarative::CurveModel,
    segment_index: usize,
    start_pointer: Point,
    raw_local_pointer: Point,
    rect: Rect,
) -> f32 {
    let drag_units = (start_pointer.y - raw_local_pointer.y) as f32
        / (CURVE_TENSION_PIXEL_SCALE * curve_scale_for_rect(rect));
    drag_units * segment_upward_tension_sign(model, segment_index)
}

/// Return a scale factor based on current curve-editor rectangle size.
fn curve_scale_for_rect(rect: Rect) -> f32 {
    let width_scale = rect.size.width.max(1) as f32 / 420.0;
    let height_scale = rect.size.height.max(1) as f32 / 258.0;
    width_scale.min(height_scale).clamp(0.1, 8.0)
}

/// Return sign for "upward bend" mapping relative to segment slope.
fn segment_upward_tension_sign(model: &crate::declarative::CurveModel, segment_index: usize) -> f32 {
    let left = model.points.get(segment_index).copied();
    let right = model.points.get(segment_index + 1).copied();
    if let (Some(left), Some(right)) = (left, right) {
        if right.y >= left.y {
            -1.0
        } else {
            1.0
        }
    } else {
        1.0
    }
}
