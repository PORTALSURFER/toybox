
use crate::canvas::{Color, Point, Rect, Size};
use crate::ui::{
    MainPalette, RegionResponse, RootFrameStyle, Ui, WidgetId, knob_block_size_for_diameter,
};

/// Validation errors produced by declarative UI helpers.
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum DeclarativeError {
    /// An interactive node was declared with an empty key.
    #[error("declarative node `{node_kind}` requires a non-empty key")]
    EmptyNodeKey {
        /// Concrete node variant that failed validation.
        node_kind: &'static str,
    },
    /// Two interactive nodes share the same key.
    #[error("duplicate declarative key `{key}`")]
    DuplicateNodeKey {
        /// Duplicated key value.
        key: String,
    },
    /// The grid template does not define any columns.
    #[error("grid template must define at least one column track")]
    EmptyGridColumns,
    /// A value range is malformed for a parameterized control.
    #[error("declarative node `{node_kind}` key `{key}` must have min < max and finite bounds")]
    InvalidValueRange {
        /// Concrete node variant that failed validation.
        node_kind: &'static str,
        /// Stable key associated with the control.
        key: String,
    },
    /// A dropdown selected index is out of bounds.
    #[error(
        "declarative node `Dropdown` key `{key}` selected index {selected} is out of bounds for {options_len} options"
    )]
    InvalidDropdownSelection {
        /// Stable dropdown key.
        key: String,
        /// Requested selected index.
        selected: usize,
        /// Number of options provided.
        options_len: usize,
    },
    /// A control was given a zero-sized explicit control box.
    #[error(
        "declarative node `{node_kind}` key `{key}` control_size must be non-zero (got {width}x{height})"
    )]
    InvalidControlSize {
        /// Concrete node variant that failed validation.
        node_kind: &'static str,
        /// Stable key associated with the control.
        key: String,
        /// Invalid width value.
        width: u32,
        /// Invalid height value.
        height: u32,
    },
    /// A control has a non-finite or out-of-range current value.
    #[error(
        "declarative node `{node_kind}` key `{key}` value {value} must be finite and inside [{min}, {max}]"
    )]
    InvalidControlValue {
        /// Concrete node variant that failed validation.
        node_kind: &'static str,
        /// Stable key associated with the control.
        key: String,
        /// Invalid value.
        value: f32,
        /// Lower range bound.
        min: f32,
        /// Upper range bound.
        max: f32,
    },
    /// Root content must be a container node.
    #[error("root content must be a container node (got `{node_kind}`)")]
    InvalidRootContent {
        /// Concrete node variant at root content position.
        node_kind: &'static str,
    },
    /// Section containers only accept container children.
    #[error("section child must be a container node (got `{node_kind}`)")]
    InvalidSectionChild {
        /// Concrete section-child node kind.
        node_kind: &'static str,
    },
    /// Section tracks must use fraction/fill sizing only.
    #[error("section tracks must use Fraction or Fill definitions")]
    InvalidSectionTrack,
    /// Section percentage definitions are malformed.
    #[error(
        "invalid section fractions: total percent {total_percent}, fill_count {fill_count} (require total <= 100 and total == 100 when fill_count == 0)"
    )]
    InvalidSectionFractions {
        /// Sum of fraction percentages in the section.
        total_percent: u16,
        /// Number of fill tracks in the section.
        fill_count: usize,
    },
}

/// Typed interaction actions emitted by declarative rendering.
#[derive(Clone, Debug, PartialEq)]
pub enum UiAction {
    /// Knob value update.
    KnobChanged {
        /// Stable widget key.
        key: String,
        /// New widget value.
        value: f32,
    },
    /// Slider value update.
    SliderChanged {
        /// Stable widget key.
        key: String,
        /// New widget value.
        value: f32,
    },
    /// Toggle value update.
    ToggleChanged {
        /// Stable widget key.
        key: String,
        /// New widget value.
        value: bool,
    },
    /// Button click event.
    ButtonPressed {
        /// Stable widget key.
        key: String,
    },
    /// Dropdown selection event.
    DropdownSelected {
        /// Stable widget key.
        key: String,
        /// Selected option index.
        index: usize,
    },
    /// Region hover state update.
    RegionHover {
        /// Stable widget key.
        key: String,
        /// True when the pointer is currently inside the region.
        hovered: bool,
        /// Pointer position relative to the region bounds.
        local_pointer: Point,
    },
    /// Region interaction event.
    RegionInteracted {
        /// Stable widget key.
        key: String,
        /// Interaction kind.
        kind: RegionInteractionKind,
        /// Pointer position relative to the interacted region.
        local_pointer: Point,
        /// Unclamped pointer position relative to the region origin.
        raw_local_pointer: Point,
        /// Whether Alt was held during this interaction frame.
        alt_down: bool,
    },
}

/// Declarative drawing command for region rendering.
#[derive(Clone, Debug, PartialEq)]
pub enum DrawCommand {
    /// Fill a rectangle at a region-relative position.
    FillRect {
        /// Region-relative rectangle.
        rect: Rect,
        /// Fill color.
        color: Color,
    },
    /// Stroke a rectangle at a region-relative position.
    StrokeRect {
        /// Region-relative rectangle.
        rect: Rect,
        /// Stroke thickness in pixels.
        thickness: u32,
        /// Stroke color.
        color: Color,
    },
    /// Fill a circle at a region-relative center.
    FillCircle {
        /// Region-relative center point.
        center: Point,
        /// Circle radius in pixels.
        radius: i32,
        /// Fill color.
        color: Color,
    },
    /// Stroke a circle at a region-relative center.
    StrokeCircle {
        /// Region-relative center point.
        center: Point,
        /// Circle radius in pixels.
        radius: i32,
        /// Stroke thickness in pixels.
        thickness: i32,
        /// Stroke color.
        color: Color,
    },
    /// Draw a line between two region-relative points.
    Line {
        /// Region-relative start point.
        start: Point,
        /// Region-relative end point.
        end: Point,
        /// Line color.
        color: Color,
    },
    /// Draw text at a region-relative origin.
    Text {
        /// Region-relative text origin.
        origin: Point,
        /// Text content.
        text: String,
        /// Text color.
        color: Color,
        /// Bitmap text scale.
        scale: u32,
    },
}

/// Specific region interaction type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionInteractionKind {
    /// Primary press began.
    Pressed,
    /// Primary press ended.
    Released,
    /// Drag in progress.
    Dragged,
    /// Secondary click occurred.
    SecondaryClicked,
    /// Double click occurred.
    DoubleClicked,
}

/// Result of rendering a declarative UI tree.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderResult {
    /// Measured root size used for host auto-resize.
    pub measured_size: Size,
    /// Actions emitted during widget interaction handling.
    pub actions: Vec<UiAction>,
    /// Resolved uniform render scale applied for this frame.
    pub resolved_scale: f32,
    /// Resolved content rectangle used for rendering root content.
    pub content_rect: Rect,
}

impl Default for RenderResult {
    fn default() -> Self {
        Self {
            measured_size: Size {
                width: 0,
                height: 0,
            },
            actions: Vec::new(),
            resolved_scale: 1.0,
            content_rect: Rect {
                origin: Point { x: 0, y: 0 },
                size: Size {
                    width: 0,
                    height: 0,
                },
            },
        }
    }
}

/// Root-level transform that maps design-space coordinates to the host surface.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RootTransform {
    /// X-axis scale from design space to surface space.
    pub scale_x: f32,
    /// Y-axis scale from design space to surface space.
    pub scale_y: f32,
    /// X-axis surface offset in pixels after scaling.
    pub offset_x: f32,
    /// Y-axis surface offset in pixels after scaling.
    pub offset_y: f32,
    /// Design-space content rectangle before transform.
    pub content_rect_design: Rect,
    /// Surface-space content rectangle after transform.
    pub content_rect_surface: Rect,
}

impl RootTransform {
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    /// Map a point from surface space back into design space.
    pub(crate) fn surface_to_design(&self, point: Point) -> Point {
        let inv_x = if self.scale_x.abs() <= f32::EPSILON {
            1.0
        } else {
            1.0 / self.scale_x
        };
        let inv_y = if self.scale_y.abs() <= f32::EPSILON {
            1.0
        } else {
            1.0 / self.scale_y
        };
        Point {
            x: ((point.x as f32 - self.offset_x) * inv_x).round() as i32,
            y: ((point.y as f32 - self.offset_y) * inv_y).round() as i32,
        }
    }
}
