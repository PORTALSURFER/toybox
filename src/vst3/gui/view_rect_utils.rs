/// Convert a boolean into a VST3 `tresult` success/failure code.
pub const fn bool_to_tresult(value: bool) -> tresult {
    if value { kResultTrue } else { kResultFalse }
}

/// Build a `ViewRect` for plugin views.
pub const fn view_rect(width: i32, height: i32) -> ViewRect {
    ViewRect {
        left: 0,
        top: 0,
        right: width,
        bottom: height,
    }
}

/// Build a constrained rectangle while keeping its positive extent representable.
///
/// VST3 hosts own the rectangle origin. If the requested positive extent cannot
/// be represented without changing that origin, reject the request instead of
/// silently shifting or collapsing the rectangle.
pub fn view_rect_with_origin(
    left: i32,
    top: i32,
    width: i32,
    height: i32,
) -> Option<ViewRect> {
    let width = width.max(1);
    let height = height.max(1);
    let right = left.checked_add(width)?;
    let bottom = top.checked_add(height)?;
    Some(ViewRect {
        left,
        top,
        right,
        bottom,
    })
}
