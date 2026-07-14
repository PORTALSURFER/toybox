/// Endpoint coupling policy for curve-editor point motion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndpointMode {
    /// Endpoint y values are independent.
    Independent,
    /// Left/right endpoint y values are kept identical.
    CoupledY,
}

/// Visual emphasis mode for active/hover curve highlights.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CurveHighlightMode {
    /// Render a clean bright-circle highlight marker.
    BrightCircle,
}

/// Keyboard modifier that can gate a curve-editor interaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CurveEditorModifier {
    /// The platform command key (Command on macOS, Control on Windows).
    Command,
}

impl CurveEditorModifier {
    /// The Shift key for the opt-in curve-point horizontal constraint.
    ///
    /// This associated token preserves exhaustive matches over
    /// [`CurveEditorModifier`] while keeping the decorator call site concise.
    #[allow(non_upper_case_globals)]
    pub const Shift: CurvePointHorizontalConstraintModifier =
        CurvePointHorizontalConstraintModifier { _private: () };
}

/// Modifier token accepted by the curve-point horizontal-constraint decorator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CurvePointHorizontalConstraintModifier {
    /// Prevent external construction so the supported token remains explicit.
    _private: (),
}

/// Opt-in configuration for moving a curve segment as one translated pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CurveSegmentMoveOptions {
    /// Modifier that must remain held throughout the segment drag.
    pub modifier: CurveEditorModifier,
    /// Curve stroke and marker color used for the gated move target.
    pub highlight: Color,
}

impl CurveSegmentMoveOptions {
    /// Build an opt-in segment-move configuration.
    pub const fn new(modifier: CurveEditorModifier, highlight: Color) -> Self {
        Self {
            modifier,
            highlight,
        }
    }
}

impl Default for CurveSegmentMoveOptions {
    fn default() -> Self {
        Self::new(
            CurveEditorModifier::Command,
            Color::rgb(255, 190, 92),
        )
    }
}

/// Interaction parameters for one curve-editor widget.
#[derive(Clone, Debug, PartialEq)]
pub struct CurveInteractionOptions {
    /// Maximum number of points allowed in the editable model.
    pub max_points: usize,
    /// Minimum x spacing between adjacent points in normalized units.
    pub min_point_spacing_x: f32,
    /// Pointer movement required before drag mode activates.
    pub drag_start_threshold_px: i32,
    /// Extra crossing distance required before push-through point deletion.
    pub push_through_threshold_px: i32,
    /// Endpoint coupling policy.
    pub endpoint_mode: EndpointMode,
    /// Whether interior points can be deleted by double click.
    pub double_click_delete_interior: bool,
    /// Snap behavior for curve-point interactions.
    pub snap: CurveSnapConfig,
}

impl Default for CurveInteractionOptions {
    fn default() -> Self {
        Self {
            max_points: 64,
            min_point_spacing_x: 1.0e-4,
            drag_start_threshold_px: 3,
            push_through_threshold_px: 2,
            endpoint_mode: EndpointMode::Independent,
            double_click_delete_interior: true,
            snap: CurveSnapConfig::default(),
        }
    }
}

/// Rendering-time grid overlay settings for one curve editor.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CurveGridConfig {
    /// Normalized x positions for brighter vertical guide lines.
    pub emphasized_verticals: Vec<f32>,
}

/// Snap behavior for curve-editor pointer interactions.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CurveSnapConfig {
    /// Whether snapping is currently enabled.
    pub enabled: bool,
    /// Normalized x positions used for vertical snapping.
    pub vertical_positions: Vec<f32>,
    /// Normalized y positions used for horizontal snapping.
    pub horizontal_positions: Vec<f32>,
}

/// Full color/style payload for curve-editor rendering.
#[derive(Clone, Debug, PartialEq)]
pub struct CurveEditorStyle {
    /// Region background color.
    pub background: Color,
    /// Region border color.
    pub border: Color,
    /// Vertical grid line color.
    pub grid_vertical: Color,
    /// Brighter vertical guide color for emphasized beat/grid lines.
    pub grid_vertical_emphasis: Color,
    /// Horizontal grid line color.
    pub grid_horizontal: Color,
    /// Base curve stroke color.
    pub line: Color,
    /// Curve stroke color when one segment is highlighted.
    pub line_highlight: Color,
    /// Node fill color.
    pub node_fill: Color,
    /// Node stroke color.
    pub node_stroke: Color,
    /// Hovered-node fill color.
    pub node_hover_fill: Color,
    /// Hovered-node stroke color.
    pub node_hover_stroke: Color,
    /// Selected-node fill color.
    pub node_selected_fill: Color,
    /// Selected-node stroke color.
    pub node_selected_stroke: Color,
    /// Preview-point fill color.
    pub preview_fill: Color,
    /// Preview-point stroke color.
    pub preview_stroke: Color,
    /// Playhead core color.
    pub playhead_core: Color,
    /// Playhead stroke color.
    pub playhead_stroke: Color,
    /// Highlight mode for active/hover emphasis.
    pub highlight_mode: CurveHighlightMode,
}

impl Default for CurveEditorStyle {
    fn default() -> Self {
        Self {
            background: Color::rgb(20, 22, 22),
            border: Color::rgb(80, 85, 80),
            grid_vertical: Color::rgb(39, 43, 40),
            grid_vertical_emphasis: Color::rgb(69, 76, 71),
            grid_horizontal: Color::rgb(53, 58, 53),
            line: Color::rgb(140, 230, 220),
            line_highlight: Color::rgb(199, 250, 242),
            node_fill: Color::rgb(170, 180, 170),
            node_stroke: Color::rgb(110, 120, 110),
            node_hover_fill: Color::rgb(220, 236, 220),
            node_hover_stroke: Color::rgb(125, 140, 125),
            node_selected_fill: Color::rgb(240, 250, 240),
            node_selected_stroke: Color::rgb(130, 145, 130),
            preview_fill: Color::rgba(170, 240, 232, 96),
            preview_stroke: Color::rgb(160, 230, 222),
            playhead_core: Color::rgb(220, 230, 220),
            playhead_stroke: Color::rgb(124, 136, 124),
            highlight_mode: CurveHighlightMode::BrightCircle,
        }
    }
}

/// Curve-editor widget specification.
#[derive(Clone, Debug, PartialEq)]
pub struct CurveEditorSpec {
    /// Stable widget key.
    pub key: String,
    /// Current curve model value.
    pub model: CurveModel,
    /// Visual style payload.
    pub style: CurveEditorStyle,
    /// Grid overlay payload.
    pub grid: CurveGridConfig,
    /// Interaction behavior payload.
    pub interaction: CurveInteractionOptions,
    /// Optional normalized playhead x position.
    pub playhead_x: Option<f32>,
    /// Layout constraints.
    pub layout: LayoutBox,
}

impl CurveEditorSpec {
    /// Build a curve-editor spec from a stable key and model.
    pub fn new(key: impl Into<String>, model: CurveModel) -> Self {
        Self {
            key: key.into(),
            model,
            style: CurveEditorStyle::default(),
            grid: CurveGridConfig::default(),
            interaction: CurveInteractionOptions::default(),
            playhead_x: None,
            layout: LayoutBox::auto(),
        }
    }

    /// Override style payload.
    pub fn style(mut self, style: CurveEditorStyle) -> Self {
        self.style = style;
        self
    }

    /// Override grid overlay payload.
    pub fn grid(mut self, grid: CurveGridConfig) -> Self {
        self.grid = grid;
        self
    }

    /// Override interaction options.
    pub fn interaction(mut self, interaction: CurveInteractionOptions) -> Self {
        self.interaction = interaction;
        self
    }

    /// Override optional playhead x position.
    pub fn playhead_x(mut self, playhead_x: Option<f32>) -> Self {
        self.playhead_x = playhead_x.map(|value| value.clamp(0.0, 1.0));
        self
    }

    /// Override layout constraints.
    pub fn layout(mut self, layout: LayoutBox) -> Self {
        self.layout = layout;
        self
    }
}
