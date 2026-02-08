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
