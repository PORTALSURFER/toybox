/// Request payload for rendering a curve editor in a fixed rectangle.
#[derive(Clone, Debug)]
pub(crate) struct CurveEditorRectRenderRequest {
    /// Stable widget id.
    pub(crate) id: WidgetId,
    /// Rectangle bounds for clipped rendering and interaction.
    pub(crate) rect: Rect,
    /// Rendering style payload.
    pub(crate) style: crate::declarative::CurveEditorStyle,
    /// Grid overlay payload.
    pub(crate) grid: crate::declarative::CurveGridConfig,
    /// Interaction configuration payload.
    pub(crate) interaction: crate::declarative::CurveInteractionOptions,
    /// Optional modifier-gated grouped segment-move configuration.
    pub(crate) segment_move: Option<crate::declarative::CurveSegmentMoveOptions>,
    /// Optional modifier that constrains point dragging horizontally.
    pub(crate) point_horizontal_constraint:
        Option<crate::declarative::CurvePointHorizontalConstraintModifier>,
    /// Optional normalized playhead x position.
    pub(crate) playhead_x: Option<f32>,
}

impl CurveEditorRectRenderRequest {
    /// Build a new curve-editor render request.
    pub(crate) fn new(
        id: WidgetId,
        rect: Rect,
        style: crate::declarative::CurveEditorStyle,
        grid: crate::declarative::CurveGridConfig,
        interaction: crate::declarative::CurveInteractionOptions,
        playhead_x: Option<f32>,
    ) -> Self {
        Self {
            id,
            rect,
            style,
            grid,
            interaction,
            segment_move: None,
            point_horizontal_constraint: None,
            playhead_x: playhead_x.map(|value| value.clamp(0.0, 1.0)),
        }
    }

    /// Opt into modifier-gated grouped segment movement for this request.
    pub(crate) fn segment_move(
        mut self,
        segment_move: crate::declarative::CurveSegmentMoveOptions,
    ) -> Self {
        self.segment_move = Some(segment_move);
        self
    }

    /// Opt into modifier-gated horizontal movement for curve-point drags.
    pub(crate) fn point_horizontal_constraint(
        mut self,
        modifier: crate::declarative::CurvePointHorizontalConstraintModifier,
    ) -> Self {
        self.point_horizontal_constraint = Some(modifier);
        self
    }
}

/// Response metadata from curve-editor widgets.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct CurveEditorResponse {
    /// The curve model changed this frame.
    pub changed: bool,
}

/// Opaque declarative interaction decorators resolved for one curve editor.
#[derive(Clone, Copy, Debug, Default)]
struct CurveEditorInteractionDecorators {
    /// Optional modifier-gated grouped segment-move configuration.
    segment_move: Option<crate::declarative::CurveSegmentMoveOptions>,
    /// Optional modifier that constrains point dragging horizontally.
    point_horizontal_constraint:
        Option<crate::declarative::CurvePointHorizontalConstraintModifier>,
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
        /// Whether the horizontal constraint was active on the prior frame.
        horizontal_constraint_active: bool,
        /// Stable normalized y captured when the horizontal constraint engaged.
        horizontal_constraint_anchor_y: Option<f32>,
        /// Normalized pointer-to-point y offset established when the constraint released.
        vertical_pointer_offset_y: f32,
        /// Whether the vertical pointer path has been rebased during this gesture.
        vertical_pointer_rebased: bool,
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
    /// Modifier-gated segment-move target or active segment index.
    segment_move_segment: Option<usize>,
    /// Dedicated highlight color for modifier-gated segment movement.
    segment_move_highlight: Option<crate::canvas::Color>,
    /// Optional preview point for insertion.
    preview_point: Option<crate::declarative::CurvePoint>,
}
