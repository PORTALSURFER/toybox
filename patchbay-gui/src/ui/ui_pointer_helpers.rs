#[cfg(test)]
fn knob_indicator_point(center: Point, radius: i32, angle: f32) -> Point {
    Point {
        x: center.x + (angle.cos() * (radius as f32 * 0.7)) as i32,
        y: center.y - (angle.sin() * (radius as f32 * 0.7)) as i32,
    }
}

/// Convert a surface pointer position into rectangle-local coordinates clamped
/// to the rectangle bounds.
fn local_pointer_in_rect(pointer: Point, rect: Rect) -> Point {
    let max_x = rect.size.width.saturating_sub(1) as i32;
    let max_y = rect.size.height.saturating_sub(1) as i32;
    Point {
        x: (pointer.x - rect.origin.x).clamp(0, max_x.max(0)),
        y: (pointer.y - rect.origin.y).clamp(0, max_y.max(0)),
    }
}

/// Convert a surface pointer position into rectangle-local coordinates without
/// clamping to section bounds.
fn raw_local_pointer_in_rect(pointer: Point, rect: Rect) -> Point {
    Point {
        x: pointer.x - rect.origin.x,
        y: pointer.y - rect.origin.y,
    }
}
