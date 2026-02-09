use crate::canvas::{Color, Point, Rect, Size};
use crate::ui::{
    DropdownRectRenderRequest, KnobRectRenderRequest, MainPalette, RegionResponse, RootFrameStyle,
    SliderRectRenderRequest, Ui, WidgetId, knob_block_size_for_diameter,
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
