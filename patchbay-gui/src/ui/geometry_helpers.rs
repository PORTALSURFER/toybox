
/// Return the axis-aligned union of two rectangles.
fn rect_union(a: Rect, b: Rect) -> Rect {
    let min_x = i64::from(a.origin.x).min(i64::from(b.origin.x));
    let min_y = i64::from(a.origin.y).min(i64::from(b.origin.y));
    let a_right = i64::from(a.origin.x).saturating_add(i64::from(a.size.width));
    let b_right = i64::from(b.origin.x).saturating_add(i64::from(b.size.width));
    let a_bottom = i64::from(a.origin.y).saturating_add(i64::from(a.size.height));
    let b_bottom = i64::from(b.origin.y).saturating_add(i64::from(b.size.height));
    let max_x = a_right.max(b_right);
    let max_y = a_bottom.max(b_bottom);
    Rect {
        origin: Point {
            x: clamp_i64_to_i32(min_x),
            y: clamp_i64_to_i32(min_y),
        },
        size: Size {
            width: length_i64_to_u32((max_x - min_x).max(0)),
            height: length_i64_to_u32((max_y - min_y).max(0)),
        },
    }
}

/// Return the intersection rectangle if `a` and `b` overlap.
fn rect_intersection(a: Rect, b: Rect) -> Option<Rect> {
    let x0 = i64::from(a.origin.x).max(i64::from(b.origin.x));
    let y0 = i64::from(a.origin.y).max(i64::from(b.origin.y));
    let x1 = i64::from(a.origin.x)
        .saturating_add(i64::from(a.size.width))
        .min(i64::from(b.origin.x).saturating_add(i64::from(b.size.width)));
    let y1 = i64::from(a.origin.y)
        .saturating_add(i64::from(a.size.height))
        .min(i64::from(b.origin.y).saturating_add(i64::from(b.size.height)));
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    Some(Rect {
        origin: Point {
            x: clamp_i64_to_i32(x0),
            y: clamp_i64_to_i32(y0),
        },
        size: Size {
            width: length_i64_to_u32(x1 - x0),
            height: length_i64_to_u32(y1 - y0),
        },
    })
}

/// Clamp an i64 coordinate to an i32 rendering coordinate.
fn clamp_i64_to_i32(value: i64) -> i32 {
    value
        .clamp(i64::from(i32::MIN), i64::from(i32::MAX))
        .try_into()
        .expect("coordinate bounds are guaranteed to fit")
}

/// Convert a non-negative dimension length to a `u32` with upper saturation.
fn length_i64_to_u32(length: i64) -> u32 {
    if length <= 0 {
        return 0;
    }
    let length = u64::try_from(length).unwrap_or(u64::MAX);
    length.min(u64::from(u32::MAX)) as u32
}
