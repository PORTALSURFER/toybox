
/// Return the axis-aligned union of two rectangles.
fn rect_union(a: Rect, b: Rect) -> Rect {
    let min_x = a.origin.x.min(b.origin.x);
    let min_y = a.origin.y.min(b.origin.y);
    let max_x = (a.origin.x + a.size.width as i32).max(b.origin.x + b.size.width as i32);
    let max_y = (a.origin.y + a.size.height as i32).max(b.origin.y + b.size.height as i32);
    Rect {
        origin: Point { x: min_x, y: min_y },
        size: Size {
            width: (max_x - min_x).max(0) as u32,
            height: (max_y - min_y).max(0) as u32,
        },
    }
}

/// Return the intersection rectangle if `a` and `b` overlap.
fn rect_intersection(a: Rect, b: Rect) -> Option<Rect> {
    let x0 = a.origin.x.max(b.origin.x);
    let y0 = a.origin.y.max(b.origin.y);
    let x1 = (a.origin.x + a.size.width as i32).min(b.origin.x + b.size.width as i32);
    let y1 = (a.origin.y + a.size.height as i32).min(b.origin.y + b.size.height as i32);
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    Some(Rect {
        origin: Point { x: x0, y: y0 },
        size: Size {
            width: (x1 - x0) as u32,
            height: (y1 - y0) as u32,
        },
    })
}
