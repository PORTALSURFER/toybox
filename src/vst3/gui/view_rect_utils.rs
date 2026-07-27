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
/// Host rectangles may carry sentinel-like origins near `i32::MAX`. Shift the
/// origin back only when necessary so `right - left` and `bottom - top` remain
/// the requested dimensions instead of silently collapsing through saturation.
pub fn view_rect_with_origin(
    left: i32,
    top: i32,
    width: i32,
    height: i32,
) -> ViewRect {
    let width = width.max(1);
    let height = height.max(1);
    let left = left.min(i32::MAX.saturating_sub(width));
    let top = top.min(i32::MAX.saturating_sub(height));
    ViewRect {
        left,
        top,
        right: left.saturating_add(width),
        bottom: top.saturating_add(height),
    }
}
