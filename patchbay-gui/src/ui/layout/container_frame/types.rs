/// Shared input specification for root and panel frame containers.
#[derive(Clone, Copy, Debug)]
struct ContainerFrameSpec<'a> {
    /// Stable key used for cached size lookup.
    key: &'a str,
    /// Optional title rendered in the header.
    title: Option<&'a str>,
    /// Padding applied around content.
    padding: i32,
    /// Optional explicit header height override.
    header_height: Option<i32>,
    /// Optional requested size supplied by caller.
    requested_size: Option<Size>,
    /// Frame origin in window coordinates.
    origin: Point,
    /// Frame background color.
    background: Color,
    /// Frame outline color.
    outline: Color,
}

/// Shared runtime values derived from a frame spec.
#[derive(Clone, Copy, Debug)]
struct ContainerFrameResolved<'a> {
    /// Widget id for state cache updates.
    id: WidgetId,
    /// Display title copied from spec for rendering.
    title: Option<&'a str>,
    /// Non-negative content padding.
    padding: i32,
    /// Header height resolved from style or typography defaults.
    header_height: i32,
    /// Requested size from caller.
    requested_size: Option<Size>,
    /// Origin used for drawing and cursor updates.
    origin: Point,
    /// Outer rect used to draw frame shell.
    outer_rect: Rect,
    /// Content origin after padding and header.
    content_origin: Point,
    /// Content rect passed to children.
    content_rect: Rect,
    /// Fill color.
    background: Color,
    /// Stroke color.
    outline: Color,
}

/// Controls how explicit size requests interact with measured content.
#[derive(Clone, Copy, Debug)]
enum ExplicitSizePolicy {
    /// Keep at least the explicit size while allowing larger measured content.
    PreserveExplicitMinimum,
    /// Always keep explicit size when it is provided.
    PreferExplicit,
}

/// Post-measure side effects for a container frame render.
#[derive(Clone, Copy, Debug, Default)]
struct ContainerFrameEffects {
    /// Move layout cursor below the measured frame.
    advance_layout_cursor: bool,
    /// Publish measured size as the root-frame size.
    update_root_frame_size: bool,
}

/// Final frame data returned to panel/root wrappers.
#[derive(Clone, Copy, Debug)]
struct ContainerFrameResult {
    /// The outer bounds of the frame.
    outer_rect: Rect,
    /// The content rectangle available to children.
    content_rect: Rect,
    /// The measured size persisted for future passes.
    measured_size: Size,
}
