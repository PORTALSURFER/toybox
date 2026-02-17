/// Main-axis start offset and inter-item gaps for justified flex placement.
struct FlexMainSpacing {
    /// Leading main-axis offset before the first child.
    leading_offset: i32,
    /// Gap values between adjacent children.
    gaps: Vec<i32>,
}

/// Geometry and alignment state shared across flex child rendering.
struct FlexRenderContext {
    /// Flex axis orientation.
    axis: Axis,
    /// Inner padded container bounds.
    inner: Rect,
    /// Available space on the cross axis.
    available_cross: i32,
    /// Cross-axis alignment mode.
    align: Align,
}

impl FlexRenderContext {
    /// Build a new render context from flex layout primitives.
    fn new(axis: Axis, inner: Rect, align: Align) -> Self {
        Self {
            axis,
            inner,
            available_cross: to_i32_saturating(axis.cross(inner.size)),
            align,
        }
    }
}

/// Convert a `u32` length into a non-wrapping `i32` using saturation.
fn to_i32_saturating(value: u32) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

/// Pre-resolved lengths used while placing flex children.
struct FlexSolvedLengths<'a> {
    /// Intrinsic measured size for each child.
    intrinsic: &'a [Size],
    /// Resolved main-axis length for each child.
    resolved_main: &'a [i32],
    /// Resolved spacing before and between children.
    main_spacing: &'a FlexMainSpacing,
}

impl<'a> FlexSolvedLengths<'a> {
    /// Build solved-length bundle from measured arrays.
    fn new(intrinsic: &'a [Size], resolved_main: &'a [i32], main_spacing: &'a FlexMainSpacing) -> Self {
        Self {
            intrinsic,
            resolved_main,
            main_spacing,
        }
    }
}
