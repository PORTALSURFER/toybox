/// Request payload for rendering a curve editor in a fixed rectangle.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CurveEditorRectRenderRequest {
    /// Stable widget id.
    pub(crate) id: WidgetId,
    /// Rectangle bounds for clipped rendering and interaction.
    pub(crate) rect: Rect,
    /// Rendering style payload.
    pub(crate) style: crate::declarative::CurveEditorStyle,
    /// Interaction configuration payload.
    pub(crate) interaction: crate::declarative::CurveInteractionOptions,
    /// Optional normalized playhead x position.
    pub(crate) playhead_x: Option<f32>,
}

impl CurveEditorRectRenderRequest {
    /// Build a new curve-editor render request.
    pub(crate) fn new(
        id: WidgetId,
        rect: Rect,
        style: crate::declarative::CurveEditorStyle,
        interaction: crate::declarative::CurveInteractionOptions,
        playhead_x: Option<f32>,
    ) -> Self {
        Self {
            id,
            rect,
            style,
            interaction,
            playhead_x: playhead_x.map(|value| value.clamp(0.0, 1.0)),
        }
    }
}

/// Response metadata from curve-editor widgets.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct CurveEditorResponse {
    /// The curve model changed this frame.
    pub changed: bool,
}

/// Runtime drag mode for curve-editor interactions.
#[derive(Clone, Debug, PartialEq)]
enum CurveEditorDragMode {
    /// Dragging one point.
    MovePoint {
        /// Index of the originally pressed point.
        origin_index: usize,
        /// Model snapshot at drag start.
        origin_model: crate::declarative::CurveModel,
        /// Pointer position at drag start.
        start_pointer: Point,
        /// True once movement passed drag-start threshold.
        dragging: bool,
    },
    /// Dragging one segment as a two-point translation.
    MoveSegment {
        /// Segment index.
        index: usize,
        /// Pointer position at drag start.
        start_pointer: Point,
        /// Start left x.
        start_left_x: f32,
        /// Start right x.
        start_right_x: f32,
        /// Start left y.
        start_left_y: f32,
        /// Start right y.
        start_right_y: f32,
        /// True once movement passed drag-start threshold.
        dragging: bool,
    },
    /// Dragging one segment tension with modifier.
    AdjustSegmentTension {
        /// Segment index.
        index: usize,
        /// Pointer position at drag start.
        start_pointer: Point,
        /// Start tension value.
        start_tension: f32,
        /// True once movement passed drag-start threshold.
        dragging: bool,
    },
}

/// Per-widget runtime state for one curve editor instance.
#[derive(Clone, Debug, Default)]
struct CurveEditorRuntimeState {
    /// Currently selected point index.
    selected_point: Option<usize>,
    /// Active drag state.
    drag_mode: Option<CurveEditorDragMode>,
}

/// Snapshot of per-frame curve-editor visual state.
#[derive(Clone, Copy, Debug, Default)]
struct CurveEditorVisualState {
    /// Selected point index.
    selected_point: Option<usize>,
    /// Hovered point index.
    hovered_point: Option<usize>,
    /// Hovered segment index.
    hovered_segment: Option<usize>,
    /// Optional preview point for insertion.
    preview_point: Option<crate::declarative::CurvePoint>,
}
