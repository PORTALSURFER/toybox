//! Strict declarative layout primitives for Patchbay GUI widgets.
//!
//! This module defines a pure-data UI specification and a renderer that emits
//! typed actions. UI state mutation is intentionally kept outside of the tree
//! via an explicit reducer step.

use crate::canvas::{Color, Point, Rect, Size};
use crate::ui::{RegionResponse, RootFrameStyle, Ui, WidgetId};

/// Validation errors produced by declarative UI helpers.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
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
    /// Region interaction event.
    RegionInteracted {
        /// Stable widget key.
        key: String,
        /// Interaction kind.
        kind: RegionInteractionKind,
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
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderResult {
    /// Measured root size used for host auto-resize.
    pub measured_size: Size,
    /// Actions emitted during widget interaction handling.
    pub actions: Vec<UiAction>,
}

/// Declarative UI tree describing a window.
#[derive(Clone, Debug)]
pub struct UiSpec {
    /// Root frame definition.
    pub root: RootFrameSpec,
}

impl UiSpec {
    /// Build a UI spec from a root frame.
    pub fn new(root: RootFrameSpec) -> Self {
        Self { root }
    }
}

/// Root frame definition for a declarative window.
#[derive(Clone, Debug)]
pub struct RootFrameSpec {
    /// Stable frame key.
    pub key: String,
    /// Optional title displayed in the header.
    pub title: Option<String>,
    /// Padding inside the frame.
    pub padding: i32,
    /// Root sizing constraints.
    pub layout: LayoutBox,
    /// Optional design-token overrides for this frame.
    pub tokens: Option<ThemeTokens>,
    /// Root content node.
    pub content: Box<Node>,
}

impl RootFrameSpec {
    /// Create a root frame with a key and content node.
    pub fn new(key: impl Into<String>, content: Node) -> Self {
        Self {
            key: key.into(),
            title: None,
            padding: 12,
            layout: LayoutBox::auto(),
            tokens: None,
            content: Box::new(content),
        }
    }

    /// Set optional title text.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Override frame padding.
    pub fn padding(mut self, padding: i32) -> Self {
        self.padding = padding;
        self
    }

    /// Override root layout constraints.
    pub fn layout(mut self, layout: LayoutBox) -> Self {
        self.layout = layout;
        self
    }

    /// Override token set for this root.
    pub fn tokens(mut self, tokens: ThemeTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }
}

/// Layout nodes for the declarative UI tree.
#[derive(Clone, Debug)]
pub enum Node {
    /// Panel container.
    Panel(PanelSpec),
    /// Horizontal flex container.
    Row(FlexSpec),
    /// Vertical flex container.
    Column(FlexSpec),
    /// Grid container.
    Grid(GridSpec),
    /// Absolute positioning container.
    Absolute(AbsoluteSpec),
    /// Label node.
    Label(LabelSpec),
    /// Spacer node.
    Spacer(SpacerSpec),
    /// Knob control.
    Knob(KnobSpec),
    /// Slider control.
    Slider(SliderSpec),
    /// Toggle control.
    Toggle(ToggleSpec),
    /// Button control.
    Button(ButtonSpec),
    /// Dropdown control.
    Dropdown(DropdownSpec),
    /// Interactive region.
    Region(RegionSpec),
    /// Indicator node.
    Indicator(IndicatorSpec),
}

impl Node {
    /// Create a row container.
    pub fn row(children: Vec<Node>) -> Self {
        Self::Row(FlexSpec::row(children))
    }

    /// Create a column container.
    pub fn column(children: Vec<Node>) -> Self {
        Self::Column(FlexSpec::column(children))
    }
}

/// Create a row container node.
pub fn row(children: Vec<Node>) -> Node {
    Node::row(children)
}

/// Create a column container node.
pub fn column(children: Vec<Node>) -> Node {
    Node::column(children)
}

/// Create a grid container node.
pub fn grid(template: GridTemplate, children: Vec<Node>) -> Node {
    Node::Grid(GridSpec::new(template, children))
}

/// Create a panel container node.
pub fn panel(key: impl Into<String>, content: Node) -> Node {
    Node::Panel(PanelSpec::new(key, content))
}

/// Create a text label node.
pub fn label(text: impl Into<String>) -> Node {
    Node::Label(LabelSpec::new(text))
}

/// Create a fixed-size spacer node.
pub fn spacer(size: Size) -> Node {
    Node::Spacer(SpacerSpec::new(size))
}

/// Create a knob control node.
pub fn knob(
    key: impl Into<String>,
    label: impl Into<String>,
    value: f32,
    range: (f32, f32),
) -> Node {
    Node::Knob(KnobSpec::new(key, label, value, range))
}

/// Create a slider control node.
pub fn slider(
    key: impl Into<String>,
    label: impl Into<String>,
    value: f32,
    range: (f32, f32),
) -> Node {
    Node::Slider(SliderSpec::new(key, label, value, range))
}

/// Create a toggle control node.
pub fn toggle(key: impl Into<String>, label: impl Into<String>, value: bool) -> Node {
    Node::Toggle(ToggleSpec::new(key, label, value))
}

/// Create a button control node.
pub fn button(key: impl Into<String>, label: impl Into<String>) -> Node {
    Node::Button(ButtonSpec::new(key, label))
}

/// Create a dropdown control node.
pub fn dropdown(
    key: impl Into<String>,
    label: impl Into<String>,
    options: Vec<String>,
    selected: usize,
) -> Node {
    Node::Dropdown(DropdownSpec::new(key, label, options, selected))
}

/// Create an interactive region node.
pub fn region(key: impl Into<String>, size: Size) -> Node {
    Node::Region(RegionSpec::new(key, size))
}

/// Create an indicator node.
pub fn indicator(size: Size, active: bool) -> Node {
    Node::Indicator(IndicatorSpec::new(size, active))
}

/// Length value for constrained layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Length {
    /// Use measured content size.
    Auto,
    /// Fixed pixels.
    Px(u32),
    /// Fill available space with optional relative weight.
    Fill(u16),
}

impl Length {
    /// Return the fill weight.
    fn fill_weight(self) -> u32 {
        match self {
            Self::Fill(weight) => weight.max(1) as u32,
            _ => 0,
        }
    }
}

/// Box constraints shared by all node types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayoutBox {
    /// Width sizing mode.
    pub width: Length,
    /// Height sizing mode.
    pub height: Length,
    /// Optional minimum width.
    pub min_width: Option<u32>,
    /// Optional minimum height.
    pub min_height: Option<u32>,
    /// Optional maximum width.
    pub max_width: Option<u32>,
    /// Optional maximum height.
    pub max_height: Option<u32>,
}

impl LayoutBox {
    /// Create unconstrained auto sizing.
    pub const fn auto() -> Self {
        Self {
            width: Length::Auto,
            height: Length::Auto,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
        }
    }

    /// Create a box that fills available space.
    pub const fn fill() -> Self {
        Self {
            width: Length::Fill(1),
            height: Length::Fill(1),
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
        }
    }

    /// Create a fixed-size box.
    pub const fn fixed(width: u32, height: u32) -> Self {
        Self {
            width: Length::Px(width),
            height: Length::Px(height),
            min_width: Some(width),
            min_height: Some(height),
            max_width: Some(width),
            max_height: Some(height),
        }
    }

    /// Set width behavior.
    pub const fn with_width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Set height behavior.
    pub const fn with_height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Set width to fill available space.
    pub const fn fill_width(mut self) -> Self {
        self.width = Length::Fill(1);
        self
    }

    /// Set height to fill available space.
    pub const fn fill_height(mut self) -> Self {
        self.height = Length::Fill(1);
        self
    }

    /// Set a fixed width while preserving current height behavior.
    pub const fn fixed_width(mut self, width: u32) -> Self {
        self.width = Length::Px(width);
        self.min_width = Some(width);
        self.max_width = Some(width);
        self
    }

    /// Set a fixed height while preserving current width behavior.
    pub const fn fixed_height(mut self, height: u32) -> Self {
        self.height = Length::Px(height);
        self.min_height = Some(height);
        self.max_height = Some(height);
        self
    }

    /// Set minimum size.
    pub const fn with_min(mut self, min_width: u32, min_height: u32) -> Self {
        self.min_width = Some(min_width);
        self.min_height = Some(min_height);
        self
    }

    /// Set minimum size constraints.
    pub const fn min(self, min_width: u32, min_height: u32) -> Self {
        self.with_min(min_width, min_height)
    }

    /// Set maximum size.
    pub const fn with_max(mut self, max_width: u32, max_height: u32) -> Self {
        self.max_width = Some(max_width);
        self.max_height = Some(max_height);
        self
    }

    /// Set maximum size constraints.
    pub const fn max(self, max_width: u32, max_height: u32) -> Self {
        self.with_max(max_width, max_height)
    }
}

/// Edge insets used by containers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EdgeInsets {
    /// Left inset in pixels.
    pub left: i32,
    /// Right inset in pixels.
    pub right: i32,
    /// Top inset in pixels.
    pub top: i32,
    /// Bottom inset in pixels.
    pub bottom: i32,
}

impl EdgeInsets {
    /// Uniform insets.
    pub const fn all(value: i32) -> Self {
        Self {
            left: value,
            right: value,
            top: value,
            bottom: value,
        }
    }

    /// Symmetric horizontal + vertical insets.
    pub const fn symmetric(horizontal: i32, vertical: i32) -> Self {
        Self {
            left: horizontal,
            right: horizontal,
            top: vertical,
            bottom: vertical,
        }
    }
}

/// Cross-axis alignment in flex layouts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    /// Start alignment.
    #[default]
    Start,
    /// Center alignment.
    Center,
    /// End alignment.
    End,
    /// Stretch across available cross-axis space.
    Stretch,
}

/// Main-axis distribution in flex layouts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Justify {
    /// Pack items at the start.
    #[default]
    Start,
    /// Center items in available main-axis space.
    Center,
    /// Pack items at the end.
    End,
}

/// Flex container specification.
#[derive(Clone, Debug)]
pub struct FlexSpec {
    /// Layout constraints for this container.
    pub layout: LayoutBox,
    /// Gap between children.
    pub gap: i32,
    /// Container padding.
    pub padding: EdgeInsets,
    /// Cross-axis alignment.
    pub align: Align,
    /// Main-axis distribution.
    pub justify: Justify,
    /// Child nodes.
    pub children: Vec<Node>,
}

impl FlexSpec {
    /// Create a row spec.
    pub fn row(children: Vec<Node>) -> Self {
        Self {
            layout: LayoutBox::auto(),
            gap: 12,
            padding: EdgeInsets::default(),
            align: Align::Start,
            justify: Justify::Start,
            children,
        }
    }

    /// Create a column spec.
    pub fn column(children: Vec<Node>) -> Self {
        Self {
            layout: LayoutBox::auto(),
            gap: 12,
            padding: EdgeInsets::default(),
            align: Align::Start,
            justify: Justify::Start,
            children,
        }
    }

    /// Override layout constraints.
    pub fn layout(mut self, layout: LayoutBox) -> Self {
        self.layout = layout;
        self
    }

    /// Override gap.
    pub fn gap(mut self, gap: i32) -> Self {
        self.gap = gap;
        self
    }

    /// Override padding.
    pub fn padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;
        self
    }

    /// Set uniform container padding.
    pub fn pad_all(mut self, value: i32) -> Self {
        self.padding = EdgeInsets::all(value);
        self
    }

    /// Set horizontal and vertical container padding.
    pub fn pad_xy(mut self, horizontal: i32, vertical: i32) -> Self {
        self.padding = EdgeInsets::symmetric(horizontal, vertical);
        self
    }

    /// Override cross-axis alignment.
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Override main-axis distribution.
    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    /// Align children to the cross-axis start.
    pub fn align_start(mut self) -> Self {
        self.align = Align::Start;
        self
    }

    /// Center children on the cross-axis.
    pub fn align_center(mut self) -> Self {
        self.align = Align::Center;
        self
    }

    /// Align children to the cross-axis end.
    pub fn align_end(mut self) -> Self {
        self.align = Align::End;
        self
    }

    /// Stretch children across the cross-axis.
    pub fn align_stretch(mut self) -> Self {
        self.align = Align::Stretch;
        self
    }

    /// Pack children at the main-axis start.
    pub fn justify_start(mut self) -> Self {
        self.justify = Justify::Start;
        self
    }

    /// Center children on the main axis.
    pub fn justify_center(mut self) -> Self {
        self.justify = Justify::Center;
        self
    }

    /// Pack children at the main-axis end.
    pub fn justify_end(mut self) -> Self {
        self.justify = Justify::End;
        self
    }
}

/// Grid track sizing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackSize {
    /// Fixed track size.
    Px(u32),
    /// Track size from intrinsic content.
    Auto,
    /// Fractional track fill weight.
    Fr(u16),
}

impl TrackSize {
    /// Return fractional weight.
    fn fr_weight(self) -> u32 {
        match self {
            Self::Fr(weight) => weight.max(1) as u32,
            _ => 0,
        }
    }
}

/// Grid template describing rows/columns.
#[derive(Clone, Debug)]
pub struct GridTemplate {
    /// Column tracks.
    pub columns: Vec<TrackSize>,
    /// Optional row tracks. Missing rows default to `Auto`.
    pub rows: Vec<TrackSize>,
    /// Track gap in pixels.
    pub gap: i32,
    /// Grid padding.
    pub padding: EdgeInsets,
}

impl GridTemplate {
    /// Build a grid template from column tracks.
    pub fn new(columns: Vec<TrackSize>) -> Self {
        Self {
            columns,
            rows: Vec::new(),
            gap: 12,
            padding: EdgeInsets::default(),
        }
    }

    /// Build a template with `count` equal fractional columns.
    pub fn columns_fr(count: usize) -> Self {
        let count = count.max(1);
        Self::new(vec![TrackSize::Fr(1); count])
    }

    /// Override row tracks.
    pub fn rows(mut self, rows: Vec<TrackSize>) -> Self {
        self.rows = rows;
        self
    }

    /// Override rows with equal fractional tracks.
    pub fn rows_fr(mut self, count: usize) -> Self {
        let count = count.max(1);
        self.rows = vec![TrackSize::Fr(1); count];
        self
    }

    /// Set uniform grid padding.
    pub fn pad_all(mut self, value: i32) -> Self {
        self.padding = EdgeInsets::all(value);
        self
    }

    /// Set horizontal and vertical grid padding.
    pub fn pad_xy(mut self, horizontal: i32, vertical: i32) -> Self {
        self.padding = EdgeInsets::symmetric(horizontal, vertical);
        self
    }

    /// Override track gap.
    pub fn gap(mut self, gap: i32) -> Self {
        self.gap = gap;
        self
    }

    /// Override padding.
    pub fn padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;
        self
    }
}

/// Grid container specification.
#[derive(Clone, Debug)]
pub struct GridSpec {
    /// Layout constraints for this container.
    pub layout: LayoutBox,
    /// Grid track template.
    pub template: GridTemplate,
    /// Child nodes in row-major order.
    pub children: Vec<Node>,
}

impl GridSpec {
    /// Create a grid specification.
    pub fn new(template: GridTemplate, children: Vec<Node>) -> Self {
        Self {
            layout: LayoutBox::auto(),
            template,
            children,
        }
    }

    /// Override layout constraints.
    pub fn layout(mut self, layout: LayoutBox) -> Self {
        self.layout = layout;
        self
    }
}

/// Panel container specification.
#[derive(Clone, Debug)]
pub struct PanelSpec {
    /// Stable panel key.
    pub key: String,
    /// Optional title.
    pub title: Option<String>,
    /// Inner padding.
    pub padding: i32,
    /// Optional background color override.
    pub background: Option<Color>,
    /// Optional outline color override.
    pub outline: Option<Color>,
    /// Optional header height override.
    pub header_height: Option<i32>,
    /// Layout constraints.
    pub layout: LayoutBox,
    /// Panel content.
    pub content: Box<Node>,
}

impl PanelSpec {
    /// Create a panel with key and content.
    pub fn new(key: impl Into<String>, content: Node) -> Self {
        Self {
            key: key.into(),
            title: None,
            padding: 12,
            background: None,
            outline: None,
            header_height: None,
            layout: LayoutBox::auto(),
            content: Box::new(content),
        }
    }

    /// Set panel title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Override panel padding.
    pub fn padding(mut self, padding: i32) -> Self {
        self.padding = padding;
        self
    }

    /// Override panel background color.
    pub fn background(mut self, background: Color) -> Self {
        self.background = Some(background);
        self
    }

    /// Override panel outline color.
    pub fn outline(mut self, outline: Color) -> Self {
        self.outline = Some(outline);
        self
    }

    /// Override panel header height.
    pub fn header_height(mut self, header_height: i32) -> Self {
        self.header_height = Some(header_height);
        self
    }

    /// Override panel layout constraints.
    pub fn layout(mut self, layout: LayoutBox) -> Self {
        self.layout = layout;
        self
    }
}

/// Absolute-positioned container specification.
#[derive(Clone, Debug)]
pub struct AbsoluteSpec {
    /// Layout constraints.
    pub layout: LayoutBox,
    /// Positioned children.
    pub children: Vec<AbsoluteChild>,
}

impl AbsoluteSpec {
    /// Create an absolute container.
    pub fn new(children: Vec<AbsoluteChild>) -> Self {
        Self {
            layout: LayoutBox::auto(),
            children,
        }
    }

    /// Override layout constraints.
    pub fn layout(mut self, layout: LayoutBox) -> Self {
        self.layout = layout;
        self
    }
}

/// Positioned child node.
#[derive(Clone, Debug)]
pub struct AbsoluteChild {
    /// Child origin relative to the container.
    pub origin: Point,
    /// Child node.
    pub node: Node,
}

impl AbsoluteChild {
    /// Create a positioned child.
    pub fn new(origin: Point, node: Node) -> Self {
        Self { origin, node }
    }
}

/// Label specification.
#[derive(Clone, Debug)]
pub struct LabelSpec {
    /// Label text.
    pub text: String,
    /// Optional text color override.
    pub color: Option<Color>,
    /// Layout constraints.
    pub layout: LayoutBox,
}

impl LabelSpec {
    /// Create a text label.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: None,
            layout: LayoutBox::auto(),
        }
    }

    /// Override layout constraints.
    pub fn layout(mut self, layout: LayoutBox) -> Self {
        self.layout = layout;
        self
    }
}

/// Spacer specification.
#[derive(Clone, Debug)]
pub struct SpacerSpec {
    /// Spacer size.
    pub size: Size,
}

impl SpacerSpec {
    /// Create a fixed spacer.
    pub const fn new(size: Size) -> Self {
        Self { size }
    }
}

/// Knob widget specification.
#[derive(Clone, Debug)]
pub struct KnobSpec {
    /// Stable widget key.
    pub key: String,
    /// Label displayed above the knob.
    pub label: String,
    /// Optional value label.
    pub value_label: Option<String>,
    /// Current value.
    pub value: f32,
    /// Value range.
    pub range: (f32, f32),
    /// Layout constraints.
    pub layout: LayoutBox,
}

impl KnobSpec {
    /// Create a knob.
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        value: f32,
        range: (f32, f32),
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            value_label: None,
            value,
            range,
            layout: LayoutBox::auto(),
        }
    }

    /// Override value label.
    pub fn value_label(mut self, value_label: impl Into<String>) -> Self {
        self.value_label = Some(value_label.into());
        self
    }

    /// Override layout constraints.
    pub fn layout(mut self, layout: LayoutBox) -> Self {
        self.layout = layout;
        self
    }
}

/// Slider widget specification.
#[derive(Clone, Debug)]
pub struct SliderSpec {
    /// Stable widget key.
    pub key: String,
    /// Label displayed above the slider.
    pub label: String,
    /// Current value.
    pub value: f32,
    /// Value range.
    pub range: (f32, f32),
    /// Optional explicit control size.
    pub control_size: Option<Size>,
    /// Layout constraints.
    pub layout: LayoutBox,
}

impl SliderSpec {
    /// Create a slider.
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        value: f32,
        range: (f32, f32),
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            value,
            range,
            control_size: None,
            layout: LayoutBox::auto(),
        }
    }

    /// Override control size.
    pub fn control_size(mut self, size: Size) -> Self {
        self.control_size = Some(size);
        self
    }

    /// Override layout constraints.
    pub fn layout(mut self, layout: LayoutBox) -> Self {
        self.layout = layout;
        self
    }
}

/// Toggle widget specification.
#[derive(Clone, Debug)]
pub struct ToggleSpec {
    /// Stable widget key.
    pub key: String,
    /// Label displayed above the toggle.
    pub label: String,
    /// Current value.
    pub value: bool,
    /// Optional explicit control size.
    pub control_size: Option<Size>,
    /// Layout constraints.
    pub layout: LayoutBox,
}

impl ToggleSpec {
    /// Create a toggle.
    pub fn new(key: impl Into<String>, label: impl Into<String>, value: bool) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            value,
            control_size: None,
            layout: LayoutBox::auto(),
        }
    }

    /// Override control size.
    pub fn control_size(mut self, size: Size) -> Self {
        self.control_size = Some(size);
        self
    }

    /// Override layout constraints.
    pub fn layout(mut self, layout: LayoutBox) -> Self {
        self.layout = layout;
        self
    }
}

/// Button widget specification.
#[derive(Clone, Debug)]
pub struct ButtonSpec {
    /// Stable widget key.
    pub key: String,
    /// Button label.
    pub label: String,
    /// Optional explicit control size.
    pub control_size: Option<Size>,
    /// Layout constraints.
    pub layout: LayoutBox,
}

impl ButtonSpec {
    /// Create a button.
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            control_size: None,
            layout: LayoutBox::auto(),
        }
    }

    /// Override control size.
    pub fn control_size(mut self, size: Size) -> Self {
        self.control_size = Some(size);
        self
    }

    /// Override layout constraints.
    pub fn layout(mut self, layout: LayoutBox) -> Self {
        self.layout = layout;
        self
    }
}

/// Dropdown widget specification.
#[derive(Clone, Debug)]
pub struct DropdownSpec {
    /// Stable widget key.
    pub key: String,
    /// Label displayed above the dropdown.
    pub label: String,
    /// Options list.
    pub options: Vec<String>,
    /// Selected index.
    pub selected: usize,
    /// Optional explicit control size.
    pub control_size: Option<Size>,
    /// Layout constraints.
    pub layout: LayoutBox,
}

impl DropdownSpec {
    /// Create a dropdown.
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        options: Vec<String>,
        selected: usize,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            options,
            selected,
            control_size: None,
            layout: LayoutBox::auto(),
        }
    }

    /// Override control size.
    pub fn control_size(mut self, size: Size) -> Self {
        self.control_size = Some(size);
        self
    }

    /// Override layout constraints.
    pub fn layout(mut self, layout: LayoutBox) -> Self {
        self.layout = layout;
        self
    }
}

/// Interactive region specification.
#[derive(Clone, Debug)]
pub struct RegionSpec {
    /// Stable widget key.
    pub key: String,
    /// Region size.
    pub size: Size,
}

impl RegionSpec {
    /// Create an interactive region.
    pub fn new(key: impl Into<String>, size: Size) -> Self {
        Self {
            key: key.into(),
            size,
        }
    }
}

/// Indicator specification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IndicatorSpec {
    /// Indicator size.
    pub size: Size,
    /// Active state.
    pub active: bool,
}

impl IndicatorSpec {
    /// Create an indicator.
    pub const fn new(size: Size, active: bool) -> Self {
        Self { size, active }
    }
}

/// Core color token set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorTokens {
    /// Window background.
    pub background: Color,
    /// Surface fill.
    pub surface: Color,
    /// Border color.
    pub border: Color,
    /// Primary text.
    pub text: Color,
    /// Accent color.
    pub accent: Color,
}

impl Default for ColorTokens {
    fn default() -> Self {
        Self {
            background: Color::rgb(18, 19, 22),
            surface: Color::rgb(52, 57, 66),
            border: Color::rgb(88, 94, 104),
            text: Color::rgb(238, 239, 242),
            accent: Color::rgb(90, 140, 220),
        }
    }
}

/// Typography token set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypographyTokens {
    /// Bitmap text scale.
    pub text_scale: u32,
}

impl Default for TypographyTokens {
    fn default() -> Self {
        Self { text_scale: 2 }
    }
}

/// Spacing token set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpacingTokens {
    /// Tiny spacing.
    pub xs: i32,
    /// Small spacing.
    pub sm: i32,
    /// Medium spacing.
    pub md: i32,
    /// Large spacing.
    pub lg: i32,
}

impl Default for SpacingTokens {
    fn default() -> Self {
        Self {
            xs: 4,
            sm: 8,
            md: 12,
            lg: 16,
        }
    }
}

/// Control-size token set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlTokens {
    /// Default knob diameter.
    pub knob_diameter: u32,
    /// Default slider width.
    pub slider_width: u32,
    /// Default slider height.
    pub slider_height: u32,
    /// Default toggle width.
    pub toggle_width: u32,
    /// Default toggle height.
    pub toggle_height: u32,
    /// Default button width.
    pub button_width: u32,
    /// Default button height.
    pub button_height: u32,
    /// Default dropdown width.
    pub dropdown_width: u32,
    /// Default dropdown height.
    pub dropdown_height: u32,
}

impl Default for ControlTokens {
    fn default() -> Self {
        Self {
            knob_diameter: 64,
            slider_width: 180,
            slider_height: 28,
            toggle_width: 64,
            toggle_height: 28,
            button_width: 120,
            button_height: 28,
            dropdown_width: 180,
            dropdown_height: 28,
        }
    }
}

/// Root design tokens for declarative rendering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ThemeTokens {
    /// Color token set.
    pub colors: ColorTokens,
    /// Typography token set.
    pub typography: TypographyTokens,
    /// Spacing token set.
    pub spacing: SpacingTokens,
    /// Control token set.
    pub controls: ControlTokens,
}

/// Measure the required size for a UI specification.
///
/// # Errors
/// Returns [`DeclarativeError`] when validation fails.
pub fn measure_checked(spec: &UiSpec) -> Result<Size, DeclarativeError> {
    validate_spec(spec)?;
    let tokens = spec.root.tokens.unwrap_or_default();
    Ok(measure_root_frame(&spec.root, &tokens))
}

/// Render a UI specification and collect typed actions.
///
/// # Errors
/// Returns [`DeclarativeError`] when validation fails.
pub fn render_checked(
    spec: &UiSpec,
    ui: &mut Ui<'_>,
    origin: Point,
) -> Result<RenderResult, DeclarativeError> {
    validate_spec(spec)?;
    let tokens = spec.root.tokens.unwrap_or_default();
    let measured = measure_root_frame(&spec.root, &tokens);
    let resolved = resolve_size(spec.root.layout, measured, measured);

    let style = RootFrameStyle {
        title: spec.root.title.as_deref(),
        padding: spec.root.padding,
        background: Some(tokens.colors.surface),
        outline: Some(tokens.colors.border),
        header_height: Some(panel_header_height(spec.root.title.as_deref(), &tokens)),
    };

    let mut actions = Vec::new();
    let _ = ui.root_frame_with_key_at(&spec.root.key, style, Some(resolved), origin, |ui, rect| {
        render_node(&spec.root.content, rect, ui, &tokens, &mut actions);
    });

    Ok(RenderResult {
        measured_size: resolved,
        actions,
    })
}

/// Validate the top-level UI specification.
fn validate_spec(spec: &UiSpec) -> Result<(), DeclarativeError> {
    if spec.root.key.trim().is_empty() {
        return Err(DeclarativeError::EmptyNodeKey {
            node_kind: "RootFrame",
        });
    }
    let mut seen = std::collections::HashSet::new();
    validate_node(&spec.root.content, &mut seen)
}

/// Validate a node subtree.
fn validate_node(
    node: &Node,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), DeclarativeError> {
    match node {
        Node::Panel(panel) => {
            validate_non_empty_key(&panel.key, "Panel")?;
            validate_unique_key(&panel.key, seen_keys)?;
            validate_node(&panel.content, seen_keys)?;
        }
        Node::Row(flex) | Node::Column(flex) => {
            for child in &flex.children {
                validate_node(child, seen_keys)?;
            }
        }
        Node::Grid(grid) => {
            if grid.template.columns.is_empty() {
                return Err(DeclarativeError::EmptyGridColumns);
            }
            for child in &grid.children {
                validate_node(child, seen_keys)?;
            }
        }
        Node::Absolute(absolute) => {
            for child in &absolute.children {
                validate_node(&child.node, seen_keys)?;
            }
        }
        Node::Label(_) | Node::Spacer(_) | Node::Indicator(_) => {}
        Node::Knob(knob) => {
            validate_non_empty_key(&knob.key, "Knob")?;
            validate_unique_key(&knob.key, seen_keys)?;
            validate_value_range("Knob", &knob.key, knob.range)?;
        }
        Node::Slider(slider) => {
            validate_non_empty_key(&slider.key, "Slider")?;
            validate_unique_key(&slider.key, seen_keys)?;
            validate_value_range("Slider", &slider.key, slider.range)?;
            if let Some(control_size) = slider.control_size {
                validate_control_size("Slider", &slider.key, control_size)?;
            }
        }
        Node::Toggle(toggle) => {
            validate_non_empty_key(&toggle.key, "Toggle")?;
            validate_unique_key(&toggle.key, seen_keys)?;
            if let Some(control_size) = toggle.control_size {
                validate_control_size("Toggle", &toggle.key, control_size)?;
            }
        }
        Node::Button(button) => {
            validate_non_empty_key(&button.key, "Button")?;
            validate_unique_key(&button.key, seen_keys)?;
            if let Some(control_size) = button.control_size {
                validate_control_size("Button", &button.key, control_size)?;
            }
        }
        Node::Dropdown(dropdown) => {
            validate_non_empty_key(&dropdown.key, "Dropdown")?;
            validate_unique_key(&dropdown.key, seen_keys)?;
            validate_dropdown_selection(dropdown)?;
            if let Some(control_size) = dropdown.control_size {
                validate_control_size("Dropdown", &dropdown.key, control_size)?;
            }
        }
        Node::Region(region) => {
            validate_non_empty_key(&region.key, "Region")?;
            validate_unique_key(&region.key, seen_keys)?;
        }
    }
    Ok(())
}

/// Validate that a key is non-empty.
fn validate_non_empty_key(key: &str, node_kind: &'static str) -> Result<(), DeclarativeError> {
    if key.trim().is_empty() {
        return Err(DeclarativeError::EmptyNodeKey { node_kind });
    }
    Ok(())
}

/// Validate key uniqueness.
fn validate_unique_key(
    key: &str,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), DeclarativeError> {
    if !seen_keys.insert(key.to_string()) {
        return Err(DeclarativeError::DuplicateNodeKey {
            key: key.to_string(),
        });
    }
    Ok(())
}

/// Validate a numeric value range.
fn validate_value_range(
    node_kind: &'static str,
    key: &str,
    range: (f32, f32),
) -> Result<(), DeclarativeError> {
    let (min, max) = range;
    if !min.is_finite() || !max.is_finite() || min >= max {
        return Err(DeclarativeError::InvalidValueRange {
            node_kind,
            key: key.to_string(),
        });
    }
    Ok(())
}

/// Validate an explicit control size override.
fn validate_control_size(
    node_kind: &'static str,
    key: &str,
    control_size: Size,
) -> Result<(), DeclarativeError> {
    if control_size.width == 0 || control_size.height == 0 {
        return Err(DeclarativeError::InvalidControlSize {
            node_kind,
            key: key.to_string(),
            width: control_size.width,
            height: control_size.height,
        });
    }
    Ok(())
}

/// Validate that dropdown selection references an existing option.
fn validate_dropdown_selection(dropdown: &DropdownSpec) -> Result<(), DeclarativeError> {
    if dropdown.selected >= dropdown.options.len() {
        return Err(DeclarativeError::InvalidDropdownSelection {
            key: dropdown.key.clone(),
            selected: dropdown.selected,
            options_len: dropdown.options.len(),
        });
    }
    Ok(())
}

/// Measure root frame size including header and padding.
fn measure_root_frame(frame: &RootFrameSpec, tokens: &ThemeTokens) -> Size {
    let content = measure_node(&frame.content, tokens);
    let header = panel_header_height(frame.title.as_deref(), tokens).max(0) as u32;
    let padding = frame.padding.max(0) as u32;
    let measured = Size {
        width: content.width + padding * 2,
        height: content.height + padding * 2 + header,
    };
    resolve_size(frame.layout, measured, measured)
}

/// Measure a node's intrinsic content size.
fn measure_node(node: &Node, tokens: &ThemeTokens) -> Size {
    match node {
        Node::Panel(panel) => measure_panel(panel, tokens),
        Node::Row(flex) => measure_flex(flex, tokens, Axis::Horizontal),
        Node::Column(flex) => measure_flex(flex, tokens, Axis::Vertical),
        Node::Grid(grid) => measure_grid(grid, tokens),
        Node::Absolute(absolute) => measure_absolute(absolute, tokens),
        Node::Label(label) => measure_label(label, tokens),
        Node::Spacer(spacer) => spacer.size,
        Node::Knob(knob) => measure_knob(knob, tokens),
        Node::Slider(slider) => measure_slider(slider, tokens),
        Node::Toggle(toggle) => measure_toggle(toggle, tokens),
        Node::Button(button) => measure_button(button, tokens),
        Node::Dropdown(dropdown) => measure_dropdown(dropdown, tokens),
        Node::Region(region) => region.size,
        Node::Indicator(indicator) => indicator.size,
    }
}

/// Measure a panel's intrinsic content size.
fn measure_panel(panel: &PanelSpec, tokens: &ThemeTokens) -> Size {
    let content = measure_node(&panel.content, tokens);
    let header = panel
        .header_height
        .unwrap_or_else(|| panel_header_height(panel.title.as_deref(), tokens))
        .max(0) as u32;
    let padding = panel.padding.max(0) as u32;
    let measured = Size {
        width: content.width + padding * 2,
        height: content.height + padding * 2 + header,
    };
    resolve_size(panel.layout, measured, measured)
}

/// Measure a flex container intrinsically.
fn measure_flex(flex: &FlexSpec, tokens: &ThemeTokens, axis: Axis) -> Size {
    let mut total_main = 0i32;
    let mut max_cross = 0i32;
    let mut child_count = 0i32;

    for child in &flex.children {
        let child_size = measure_node(child, tokens);
        let (main, cross) = match axis {
            Axis::Horizontal => (child_size.width as i32, child_size.height as i32),
            Axis::Vertical => (child_size.height as i32, child_size.width as i32),
        };
        total_main += main;
        max_cross = max_cross.max(cross);
        child_count += 1;
    }

    let gap = flex.gap.max(0);
    let gap_total = gap * child_count.saturating_sub(1);
    total_main += gap_total;

    let (main_padding, cross_padding) = match axis {
        Axis::Horizontal => (
            flex.padding.left + flex.padding.right,
            flex.padding.top + flex.padding.bottom,
        ),
        Axis::Vertical => (
            flex.padding.top + flex.padding.bottom,
            flex.padding.left + flex.padding.right,
        ),
    };

    let measured = match axis {
        Axis::Horizontal => Size {
            width: (total_main + main_padding).max(0) as u32,
            height: (max_cross + cross_padding).max(0) as u32,
        },
        Axis::Vertical => Size {
            width: (max_cross + cross_padding).max(0) as u32,
            height: (total_main + main_padding).max(0) as u32,
        },
    };

    resolve_size(flex.layout, measured, measured)
}

/// Measure a grid container intrinsically.
fn measure_grid(grid: &GridSpec, tokens: &ThemeTokens) -> Size {
    let column_count = grid.template.columns.len().max(1);
    let row_count = if grid.children.is_empty() {
        0
    } else {
        grid.children.len().div_ceil(column_count)
    };

    let mut column_widths = vec![0u32; column_count];
    let mut row_heights = vec![0u32; row_count];

    for (index, child) in grid.children.iter().enumerate() {
        let size = measure_node(child, tokens);
        let col = index % column_count;
        let row = index / column_count;
        column_widths[col] = column_widths[col].max(size.width);
        row_heights[row] = row_heights[row].max(size.height);
    }

    for (index, track) in grid.template.columns.iter().copied().enumerate() {
        if let Some(width) = column_widths.get_mut(index)
            && let TrackSize::Px(px) = track
        {
            *width = (*width).max(px);
        }
    }

    for (index, track) in grid.template.rows.iter().copied().enumerate() {
        if let Some(height) = row_heights.get_mut(index)
            && let TrackSize::Px(px) = track
        {
            *height = (*height).max(px);
        }
    }

    let gap = grid.template.gap.max(0) as u32;
    let width = column_widths.iter().copied().sum::<u32>()
        + gap.saturating_mul(column_widths.len().saturating_sub(1) as u32)
        + grid.template.padding.left.max(0) as u32
        + grid.template.padding.right.max(0) as u32;
    let height = row_heights.iter().copied().sum::<u32>()
        + gap.saturating_mul(row_heights.len().saturating_sub(1) as u32)
        + grid.template.padding.top.max(0) as u32
        + grid.template.padding.bottom.max(0) as u32;

    resolve_size(grid.layout, Size { width, height }, Size { width, height })
}

/// Measure an absolute container intrinsically.
fn measure_absolute(absolute: &AbsoluteSpec, tokens: &ThemeTokens) -> Size {
    let mut max_x = 0i32;
    let mut max_y = 0i32;

    for child in &absolute.children {
        let size = measure_node(&child.node, tokens);
        max_x = max_x.max(child.origin.x + size.width as i32);
        max_y = max_y.max(child.origin.y + size.height as i32);
    }

    resolve_size(
        absolute.layout,
        Size {
            width: max_x.max(0) as u32,
            height: max_y.max(0) as u32,
        },
        Size {
            width: max_x.max(0) as u32,
            height: max_y.max(0) as u32,
        },
    )
}

/// Measure a label node.
fn measure_label(label: &LabelSpec, tokens: &ThemeTokens) -> Size {
    let measured = text_size(&label.text, tokens.typography.text_scale);
    resolve_size(label.layout, measured, measured)
}

/// Measure a knob node.
fn measure_knob(knob: &KnobSpec, tokens: &ThemeTokens) -> Size {
    let control = tokens.controls.knob_diameter.max(1);
    let label_h = 8 * tokens.typography.text_scale.max(1);
    let spacing = tokens.spacing.xs.max(0) as u32;
    let label = text_size(&knob.label, tokens.typography.text_scale);
    let value_label = knob
        .value_label
        .clone()
        .unwrap_or_else(|| format_value(knob.value));
    let value = text_size(&value_label, tokens.typography.text_scale);
    let measured = Size {
        width: control.max(label.width).max(value.width),
        height: control + label_h * 2 + spacing * 2,
    };
    resolve_size(knob.layout, measured, measured)
}

/// Measure a slider node.
fn measure_slider(slider: &SliderSpec, tokens: &ThemeTokens) -> Size {
    let control = slider.control_size.unwrap_or(Size {
        width: tokens.controls.slider_width,
        height: tokens.controls.slider_height,
    });
    let label_h = if slider.label.is_empty() {
        0
    } else {
        8 * tokens.typography.text_scale.max(1)
    };
    let label = text_size(&slider.label, tokens.typography.text_scale);
    let measured = Size {
        width: control.width.max(label.width),
        height: control.height + label_h,
    };
    resolve_size(slider.layout, measured, measured)
}

/// Measure a toggle node.
fn measure_toggle(toggle: &ToggleSpec, tokens: &ThemeTokens) -> Size {
    let control = toggle.control_size.unwrap_or(Size {
        width: tokens.controls.toggle_width,
        height: tokens.controls.toggle_height,
    });
    let label_h = if toggle.label.is_empty() {
        0
    } else {
        8 * tokens.typography.text_scale.max(1)
    };
    let label = text_size(&toggle.label, tokens.typography.text_scale);
    let measured = Size {
        width: control.width.max(label.width),
        height: control.height + label_h,
    };
    resolve_size(toggle.layout, measured, measured)
}

/// Measure a button node.
fn measure_button(button: &ButtonSpec, tokens: &ThemeTokens) -> Size {
    let control = button.control_size.unwrap_or(Size {
        width: tokens.controls.button_width,
        height: tokens.controls.button_height,
    });
    let label = text_size(&button.label, tokens.typography.text_scale);
    let measured = Size {
        width: control.width.max(label.width + 8),
        height: control.height.max(label.height + 4),
    };
    resolve_size(button.layout, measured, measured)
}

/// Measure a dropdown node.
fn measure_dropdown(dropdown: &DropdownSpec, tokens: &ThemeTokens) -> Size {
    let control = dropdown.control_size.unwrap_or(Size {
        width: tokens.controls.dropdown_width,
        height: tokens.controls.dropdown_height,
    });
    let label_h = if dropdown.label.is_empty() {
        0
    } else {
        8 * tokens.typography.text_scale.max(1)
    };
    let label = text_size(&dropdown.label, tokens.typography.text_scale);
    let measured = Size {
        width: control.width.max(label.width),
        height: control.height + label_h,
    };
    resolve_size(dropdown.layout, measured, measured)
}

/// Resolve a measured size against box constraints.
fn resolve_size(layout: LayoutBox, measured: Size, available: Size) -> Size {
    Size {
        width: resolve_axis(
            layout.width,
            measured.width,
            available.width,
            layout.min_width,
            layout.max_width,
        ),
        height: resolve_axis(
            layout.height,
            measured.height,
            available.height,
            layout.min_height,
            layout.max_height,
        ),
    }
}

/// Resolve a single-axis length against constraints.
fn resolve_axis(
    length: Length,
    measured: u32,
    available: u32,
    min: Option<u32>,
    max: Option<u32>,
) -> u32 {
    let base = match length {
        Length::Auto => measured,
        Length::Px(px) => px,
        Length::Fill(_) => available.max(measured),
    };
    let min_applied = base.max(min.unwrap_or(0));
    if let Some(max_value) = max {
        min_applied.min(max_value)
    } else {
        min_applied
    }
}

/// Render a node subtree and collect actions.
fn render_node(
    node: &Node,
    rect: Rect,
    ui: &mut Ui<'_>,
    tokens: &ThemeTokens,
    actions: &mut Vec<UiAction>,
) {
    match node {
        Node::Panel(panel) => render_panel(panel, rect, ui, tokens, actions),
        Node::Row(flex) => render_flex(flex, rect, ui, tokens, Axis::Horizontal, actions),
        Node::Column(flex) => render_flex(flex, rect, ui, tokens, Axis::Vertical, actions),
        Node::Grid(grid) => render_grid(grid, rect, ui, tokens, actions),
        Node::Absolute(absolute) => render_absolute(absolute, rect, ui, tokens, actions),
        Node::Label(label) => render_label(label, rect, ui, tokens),
        Node::Spacer(_) => {}
        Node::Knob(knob) => render_knob(knob, rect, ui, tokens, actions),
        Node::Slider(slider) => render_slider(slider, rect, ui, tokens, actions),
        Node::Toggle(toggle) => render_toggle(toggle, rect, ui, tokens, actions),
        Node::Button(button) => render_button(button, rect, ui, tokens, actions),
        Node::Dropdown(dropdown) => render_dropdown(dropdown, rect, ui, tokens, actions),
        Node::Region(region) => render_region(region, rect, ui, actions),
        Node::Indicator(indicator) => render_indicator(indicator, rect, ui),
    }
}

/// Render a panel container.
fn render_panel(
    panel: &PanelSpec,
    rect: Rect,
    ui: &mut Ui<'_>,
    tokens: &ThemeTokens,
    actions: &mut Vec<UiAction>,
) {
    let title = panel.title.as_deref();
    let header_height = panel
        .header_height
        .unwrap_or_else(|| panel_header_height(title, tokens));
    let style = crate::ui::PanelStyle {
        title,
        padding: panel.padding,
        background: Some(panel.background.unwrap_or(tokens.colors.surface)),
        outline: Some(panel.outline.unwrap_or(tokens.colors.border)),
        header_height: Some(header_height),
    };

    let _ = ui.panel_with_key(&panel.key, style, Some(rect.size), |ui, content_rect| {
        render_node(&panel.content, content_rect, ui, tokens, actions);
    });
}

/// Render a flex container.
fn render_flex(
    flex: &FlexSpec,
    rect: Rect,
    ui: &mut Ui<'_>,
    tokens: &ThemeTokens,
    axis: Axis,
    actions: &mut Vec<UiAction>,
) {
    let child_count = flex.children.len();
    if child_count == 0 {
        return;
    }

    let mut intrinsic = Vec::with_capacity(child_count);
    for child in &flex.children {
        intrinsic.push(measure_node(child, tokens));
    }

    let gap = flex.gap.max(0);
    let inner = inset_rect(rect, flex.padding);
    let available_main = axis.main(inner.size) as i32;
    let available_cross = axis.cross(inner.size) as i32;

    let mut base_main = vec![0i32; child_count];
    let mut fill_weight_sum = 0u32;
    let mut main_sum = 0i32;

    for (index, child) in flex.children.iter().enumerate() {
        let layout = node_layout(child);
        let measured_main = axis.main(intrinsic[index]) as i32;
        let value = match axis.main_length(layout) {
            Length::Px(px) => px as i32,
            Length::Auto | Length::Fill(_) => measured_main,
        };
        base_main[index] = value.max(0);
        main_sum += base_main[index];
        fill_weight_sum += axis.main_length(layout).fill_weight();
    }

    let total_gap = gap * (child_count.saturating_sub(1) as i32);
    let remainder = (available_main - main_sum - total_gap).max(0);

    let mut resolved_main = base_main.clone();
    if fill_weight_sum > 0 {
        for (index, child) in flex.children.iter().enumerate() {
            let weight = axis.main_length(node_layout(child)).fill_weight();
            if weight > 0 {
                let extra =
                    ((remainder as i64) * (weight as i64) / (fill_weight_sum as i64)) as i32;
                resolved_main[index] += extra;
            }
        }
    }

    let occupied_main = resolved_main.iter().copied().sum::<i32>() + total_gap;
    let mut cursor_main = match flex.justify {
        Justify::Start => axis.origin_main(inner.origin),
        Justify::Center => axis.origin_main(inner.origin) + (available_main - occupied_main) / 2,
        Justify::End => axis.origin_main(inner.origin) + (available_main - occupied_main),
    };

    for (index, child) in flex.children.iter().enumerate() {
        let layout = node_layout(child);
        let intrinsic_cross = axis.cross(intrinsic[index]) as i32;
        let cross_size = match axis.cross_length(layout) {
            Length::Px(px) => px as i32,
            Length::Fill(_) => available_cross,
            Length::Auto => {
                if flex.align == Align::Stretch {
                    available_cross
                } else {
                    intrinsic_cross
                }
            }
        }
        .max(0);

        let cross_origin = match flex.align {
            Align::Start | Align::Stretch => axis.origin_cross(inner.origin),
            Align::Center => axis.origin_cross(inner.origin) + (available_cross - cross_size) / 2,
            Align::End => axis.origin_cross(inner.origin) + (available_cross - cross_size),
        };

        let child_rect =
            axis.compose_rect(cursor_main, cross_origin, resolved_main[index], cross_size);
        let resolved_child = resolve_size(layout, intrinsic[index], child_rect.size);
        let child_rect = Rect {
            origin: child_rect.origin,
            size: resolved_child,
        };

        render_node(child, child_rect, ui, tokens, actions);
        cursor_main += resolved_main[index] + gap;
    }
}

/// Render a grid container.
fn render_grid(
    grid: &GridSpec,
    rect: Rect,
    ui: &mut Ui<'_>,
    tokens: &ThemeTokens,
    actions: &mut Vec<UiAction>,
) {
    let columns = grid.template.columns.len().max(1);
    let rows = if grid.children.is_empty() {
        0
    } else {
        grid.children.len().div_ceil(columns)
    };
    if rows == 0 {
        return;
    }

    let inner = inset_rect(rect, grid.template.padding);
    let intrinsic: Vec<Size> = grid
        .children
        .iter()
        .map(|child| measure_node(child, tokens))
        .collect();

    let column_widths = resolve_grid_axis(
        &grid.template.columns,
        columns,
        rows,
        grid.template.gap.max(0),
        inner.size.width,
        true,
        &intrinsic,
    );
    let row_tracks = if grid.template.rows.is_empty() {
        vec![TrackSize::Auto; rows]
    } else {
        let mut tracks = grid.template.rows.clone();
        if tracks.len() < rows {
            tracks.resize(rows, TrackSize::Auto);
        }
        tracks
    };
    let row_heights = resolve_grid_axis(
        &row_tracks,
        columns,
        rows,
        grid.template.gap.max(0),
        inner.size.height,
        false,
        &intrinsic,
    );

    let gap = grid.template.gap.max(0);
    let mut y = inner.origin.y;
    for (row, row_height) in row_heights.iter().copied().enumerate().take(rows) {
        let mut x = inner.origin.x;
        for (col, col_width) in column_widths.iter().copied().enumerate().take(columns) {
            let index = row * columns + col;
            if let Some(child) = grid.children.get(index) {
                let cell_rect = Rect {
                    origin: Point { x, y },
                    size: Size {
                        width: col_width,
                        height: row_height,
                    },
                };
                let layout = node_layout(child);
                let measured = intrinsic[index];
                let resolved = resolve_size(layout, measured, cell_rect.size);
                render_node(
                    child,
                    Rect {
                        origin: cell_rect.origin,
                        size: resolved,
                    },
                    ui,
                    tokens,
                    actions,
                );
            }
            x += col_width as i32 + gap;
        }
        y += row_height as i32 + gap;
    }
}

/// Resolve one grid axis using track definitions and available space.
fn resolve_grid_axis(
    tracks: &[TrackSize],
    columns: usize,
    rows: usize,
    gap: i32,
    available: u32,
    is_columns: bool,
    intrinsic: &[Size],
) -> Vec<u32> {
    let count = if is_columns { columns } else { rows };
    let mut result = vec![0u32; count];

    for (index, value) in result.iter_mut().enumerate().take(count) {
        if let Some(track) = tracks.get(index).copied()
            && let TrackSize::Px(px) = track
        {
            *value = px;
        }
    }

    for (item, measured) in intrinsic.iter().enumerate() {
        let row = item / columns;
        let col = item % columns;
        let axis_index = if is_columns { col } else { row };
        let track = tracks.get(axis_index).copied().unwrap_or(TrackSize::Auto);
        if matches!(track, TrackSize::Auto) {
            let value = if is_columns {
                measured.width
            } else {
                measured.height
            };
            result[axis_index] = result[axis_index].max(value);
        }
    }

    let total_gap = gap.max(0) as u32 * count.saturating_sub(1) as u32;
    let used = result.iter().copied().sum::<u32>() + total_gap;
    let remainder = available.saturating_sub(used);

    let fr_sum: u32 = (0..count)
        .map(|index| {
            tracks
                .get(index)
                .copied()
                .unwrap_or(TrackSize::Auto)
                .fr_weight()
        })
        .sum();

    if fr_sum > 0 {
        for (index, value) in result.iter_mut().enumerate().take(count) {
            let track = tracks.get(index).copied().unwrap_or(TrackSize::Auto);
            let weight = track.fr_weight();
            if weight > 0 {
                *value += ((remainder as u64) * (weight as u64) / (fr_sum as u64)) as u32;
            }
        }
    }

    result
}

/// Render an absolute-positioned container.
fn render_absolute(
    absolute: &AbsoluteSpec,
    rect: Rect,
    ui: &mut Ui<'_>,
    tokens: &ThemeTokens,
    actions: &mut Vec<UiAction>,
) {
    for child in &absolute.children {
        let measured = measure_node(&child.node, tokens);
        let layout = node_layout(&child.node);
        let resolved = resolve_size(layout, measured, measured);
        let child_rect = Rect {
            origin: Point {
                x: rect.origin.x + child.origin.x,
                y: rect.origin.y + child.origin.y,
            },
            size: resolved,
        };
        render_node(&child.node, child_rect, ui, tokens, actions);
    }
}

/// Render a label node.
fn render_label(label: &LabelSpec, rect: Rect, ui: &mut Ui<'_>, tokens: &ThemeTokens) {
    let color = label.color.unwrap_or(tokens.colors.text);
    ui.text_with_color(rect.origin, &label.text, color);
}

/// Render a knob node and emit actions.
fn render_knob(
    knob: &KnobSpec,
    rect: Rect,
    ui: &mut Ui<'_>,
    _tokens: &ThemeTokens,
    actions: &mut Vec<UiAction>,
) {
    let id = WidgetId::from_label(&knob.key);
    let mut value = knob.value;
    let value_label = knob
        .value_label
        .clone()
        .unwrap_or_else(|| format_value(knob.value));
    let response =
        ui.knob_with_labels_in_rect(id, &knob.label, &value_label, &mut value, knob.range, rect);
    if response.changed {
        actions.push(UiAction::KnobChanged {
            key: knob.key.clone(),
            value,
        });
    }
}

/// Render a slider node and emit actions.
fn render_slider(
    slider: &SliderSpec,
    rect: Rect,
    ui: &mut Ui<'_>,
    tokens: &ThemeTokens,
    actions: &mut Vec<UiAction>,
) {
    let id = WidgetId::from_label(&slider.key);
    let mut value = slider.value;
    let control_size = slider.control_size.unwrap_or(Size {
        width: tokens.controls.slider_width,
        height: tokens.controls.slider_height,
    });
    let response = ui.slider_in_rect(
        id,
        &slider.label,
        &mut value,
        slider.range,
        control_size,
        rect,
    );
    if response.changed {
        actions.push(UiAction::SliderChanged {
            key: slider.key.clone(),
            value,
        });
    }
}

/// Render a toggle node and emit actions.
fn render_toggle(
    toggle: &ToggleSpec,
    rect: Rect,
    ui: &mut Ui<'_>,
    tokens: &ThemeTokens,
    actions: &mut Vec<UiAction>,
) {
    let id = WidgetId::from_label(&toggle.key);
    let mut value = toggle.value;
    let control_size = toggle.control_size.unwrap_or(Size {
        width: tokens.controls.toggle_width,
        height: tokens.controls.toggle_height,
    });
    let response = ui.toggle_in_rect(id, &toggle.label, &mut value, control_size, rect);
    if response.changed {
        actions.push(UiAction::ToggleChanged {
            key: toggle.key.clone(),
            value,
        });
    }
}

/// Render a button node and emit actions.
fn render_button(
    button: &ButtonSpec,
    rect: Rect,
    ui: &mut Ui<'_>,
    tokens: &ThemeTokens,
    actions: &mut Vec<UiAction>,
) {
    let id = WidgetId::from_label(&button.key);
    let control_size = button.control_size.unwrap_or(Size {
        width: tokens.controls.button_width,
        height: tokens.controls.button_height,
    });
    let response = ui.button_in_rect(id, &button.label, control_size, rect);
    if response.clicked {
        actions.push(UiAction::ButtonPressed {
            key: button.key.clone(),
        });
    }
}

/// Render a dropdown node and emit actions.
fn render_dropdown(
    dropdown: &DropdownSpec,
    rect: Rect,
    ui: &mut Ui<'_>,
    tokens: &ThemeTokens,
    actions: &mut Vec<UiAction>,
) {
    let id = WidgetId::from_label(&dropdown.key);
    let control_size = dropdown.control_size.unwrap_or(Size {
        width: tokens.controls.dropdown_width,
        height: tokens.controls.dropdown_height,
    });
    let mut selected = dropdown.selected;
    let option_refs: Vec<&str> = dropdown.options.iter().map(String::as_str).collect();
    let response = ui.dropdown_in_rect(
        id,
        &dropdown.label,
        &option_refs,
        &mut selected,
        control_size,
        rect,
    );
    if response.changed {
        actions.push(UiAction::DropdownSelected {
            key: dropdown.key.clone(),
            index: selected,
        });
    }
}

/// Render a region node and emit interaction actions.
fn render_region(region: &RegionSpec, rect: Rect, ui: &mut Ui<'_>, actions: &mut Vec<UiAction>) {
    let response = ui.region_with_key(&region.key, rect);
    push_region_actions(&region.key, response, actions);
}

/// Convert region responses to action list entries.
fn push_region_actions(key: &str, response: RegionResponse, actions: &mut Vec<UiAction>) {
    if response.pressed {
        actions.push(UiAction::RegionInteracted {
            key: key.to_string(),
            kind: RegionInteractionKind::Pressed,
        });
    }
    if response.released {
        actions.push(UiAction::RegionInteracted {
            key: key.to_string(),
            kind: RegionInteractionKind::Released,
        });
    }
    if response.dragged {
        actions.push(UiAction::RegionInteracted {
            key: key.to_string(),
            kind: RegionInteractionKind::Dragged,
        });
    }
    if response.secondary_clicked {
        actions.push(UiAction::RegionInteracted {
            key: key.to_string(),
            kind: RegionInteractionKind::SecondaryClicked,
        });
    }
    if response.double_clicked {
        actions.push(UiAction::RegionInteracted {
            key: key.to_string(),
            kind: RegionInteractionKind::DoubleClicked,
        });
    }
}

/// Render an indicator node.
fn render_indicator(indicator: &IndicatorSpec, rect: Rect, ui: &mut Ui<'_>) {
    ui.indicator(
        Rect {
            origin: rect.origin,
            size: indicator.size,
        },
        indicator.active,
    );
}

/// Return node layout constraints.
fn node_layout(node: &Node) -> LayoutBox {
    match node {
        Node::Panel(panel) => panel.layout,
        Node::Row(flex) | Node::Column(flex) => flex.layout,
        Node::Grid(grid) => grid.layout,
        Node::Absolute(absolute) => absolute.layout,
        Node::Label(label) => label.layout,
        Node::Spacer(spacer) => LayoutBox::fixed(spacer.size.width, spacer.size.height),
        Node::Knob(knob) => knob.layout,
        Node::Slider(slider) => slider.layout,
        Node::Toggle(toggle) => toggle.layout,
        Node::Button(button) => button.layout,
        Node::Dropdown(dropdown) => dropdown.layout,
        Node::Region(region) => LayoutBox::fixed(region.size.width, region.size.height),
        Node::Indicator(indicator) => LayoutBox::fixed(indicator.size.width, indicator.size.height),
    }
}

/// Compute header height for titled containers.
fn panel_header_height(title: Option<&str>, tokens: &ThemeTokens) -> i32 {
    if title.is_some() {
        (8 * tokens.typography.text_scale as i32 + tokens.spacing.xs).max(0)
    } else {
        0
    }
}

/// Inset a rectangle by edge insets.
fn inset_rect(rect: Rect, insets: EdgeInsets) -> Rect {
    let left = insets.left.max(0) as u32;
    let right = insets.right.max(0) as u32;
    let top = insets.top.max(0) as u32;
    let bottom = insets.bottom.max(0) as u32;

    Rect {
        origin: Point {
            x: rect.origin.x + left as i32,
            y: rect.origin.y + top as i32,
        },
        size: Size {
            width: rect.size.width.saturating_sub(left + right),
            height: rect.size.height.saturating_sub(top + bottom),
        },
    }
}

/// Format control values for generated labels.
fn format_value(value: f32) -> String {
    let mut text = if value.abs() >= 1.0 {
        format!("{value:.2}")
    } else if value.abs() >= 0.1 {
        format!("{value:.3}")
    } else {
        format!("{value:.4}")
    };
    if let Some(dot) = text.find('.') {
        while text.ends_with('0') && text.len() > dot + 2 {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
    }
    if text == "-0" { "0".to_string() } else { text }
}

/// Measure monospaced bitmap text bounds at a given scale.
fn text_size(text: &str, scale: u32) -> Size {
    let scale = scale.max(1) as i32;
    let mut max_cols = 0i32;
    let mut lines = 1i32;
    let mut current = 0i32;
    for ch in text.chars() {
        if ch == '\n' {
            max_cols = max_cols.max(current);
            current = 0;
            lines += 1;
        } else {
            current += 1;
        }
    }
    max_cols = max_cols.max(current);
    Size {
        width: (max_cols * 6 * scale).max(0) as u32,
        height: (lines * 8 * scale).max(0) as u32,
    }
}

/// Major axis marker for flex layout.
#[derive(Clone, Copy)]
enum Axis {
    /// Horizontal row axis.
    Horizontal,
    /// Vertical column axis.
    Vertical,
}

impl Axis {
    /// Return main-axis length from a size.
    fn main(self, size: Size) -> u32 {
        match self {
            Self::Horizontal => size.width,
            Self::Vertical => size.height,
        }
    }

    /// Return cross-axis length from a size.
    fn cross(self, size: Size) -> u32 {
        match self {
            Self::Horizontal => size.height,
            Self::Vertical => size.width,
        }
    }

    /// Return main-axis origin coordinate.
    fn origin_main(self, origin: Point) -> i32 {
        match self {
            Self::Horizontal => origin.x,
            Self::Vertical => origin.y,
        }
    }

    /// Return cross-axis origin coordinate.
    fn origin_cross(self, origin: Point) -> i32 {
        match self {
            Self::Horizontal => origin.y,
            Self::Vertical => origin.x,
        }
    }

    /// Return main-axis length constraint from a layout box.
    fn main_length(self, layout: LayoutBox) -> Length {
        match self {
            Self::Horizontal => layout.width,
            Self::Vertical => layout.height,
        }
    }

    /// Return cross-axis length constraint from a layout box.
    fn cross_length(self, layout: LayoutBox) -> Length {
        match self {
            Self::Horizontal => layout.height,
            Self::Vertical => layout.width,
        }
    }

    /// Compose a rectangle from axis-oriented values.
    fn compose_rect(self, main: i32, cross: i32, main_size: i32, cross_size: i32) -> Rect {
        match self {
            Self::Horizontal => Rect {
                origin: Point { x: main, y: cross },
                size: Size {
                    width: main_size.max(0) as u32,
                    height: cross_size.max(0) as u32,
                },
            },
            Self::Vertical => Rect {
                origin: Point { x: cross, y: main },
                size: Size {
                    width: cross_size.max(0) as u32,
                    height: main_size.max(0) as u32,
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::Canvas;
    use crate::host::InputState;
    use crate::ui::{Layout, Theme, UiState};

    #[test]
    fn rejects_duplicate_widget_keys() {
        let spec = UiSpec::new(RootFrameSpec::new(
            "root",
            Node::column(vec![
                Node::Knob(KnobSpec::new("k", "A", 0.5, (0.0, 1.0))),
                Node::Knob(KnobSpec::new("k", "B", 0.5, (0.0, 1.0))),
            ]),
        ));
        let error = measure_checked(&spec).expect_err("expected duplicate key error");
        assert!(matches!(error, DeclarativeError::DuplicateNodeKey { .. }));
    }

    #[test]
    fn measures_grid_from_template_and_children() {
        let grid = GridSpec::new(
            GridTemplate::new(vec![TrackSize::Px(32), TrackSize::Fr(1)]),
            vec![
                Node::Spacer(SpacerSpec::new(Size {
                    width: 10,
                    height: 12,
                })),
                Node::Spacer(SpacerSpec::new(Size {
                    width: 20,
                    height: 14,
                })),
            ],
        );
        let spec = UiSpec::new(RootFrameSpec::new("root", Node::Grid(grid)));
        let measured = measure_checked(&spec).expect("measurement should succeed");
        assert!(measured.width >= 32);
        assert!(measured.height >= 14);
    }

    #[test]
    fn rejects_invalid_knob_range() {
        let spec = UiSpec::new(RootFrameSpec::new(
            "root",
            Node::Knob(KnobSpec::new("k", "Drive", 0.5, (1.0, 1.0))),
        ));
        let error = measure_checked(&spec).expect_err("expected invalid range error");
        assert!(matches!(
            error,
            DeclarativeError::InvalidValueRange { node_kind, .. } if node_kind == "Knob"
        ));
    }

    #[test]
    fn rejects_invalid_slider_range() {
        let spec = UiSpec::new(RootFrameSpec::new(
            "root",
            Node::Slider(SliderSpec::new("s", "Shape", 0.5, (0.8, 0.2))),
        ));
        let error = measure_checked(&spec).expect_err("expected invalid range error");
        assert!(matches!(
            error,
            DeclarativeError::InvalidValueRange { node_kind, .. } if node_kind == "Slider"
        ));
    }

    #[test]
    fn rejects_dropdown_selection_out_of_bounds() {
        let spec = UiSpec::new(RootFrameSpec::new(
            "root",
            Node::Dropdown(DropdownSpec::new(
                "mode",
                "Mode",
                vec!["A".to_string(), "B".to_string()],
                2,
            )),
        ));
        let error = measure_checked(&spec).expect_err("expected invalid dropdown selection");
        assert!(matches!(
            error,
            DeclarativeError::InvalidDropdownSelection {
                key,
                selected,
                options_len
            } if key == "mode" && selected == 2 && options_len == 2
        ));
    }

    #[test]
    fn rejects_zero_control_size() {
        let spec = UiSpec::new(RootFrameSpec::new(
            "root",
            Node::Slider(
                SliderSpec::new("s", "Shape", 0.5, (0.0, 1.0)).control_size(Size {
                    width: 0,
                    height: 24,
                }),
            ),
        ));
        let error = measure_checked(&spec).expect_err("expected invalid control size");
        assert!(matches!(
            error,
            DeclarativeError::InvalidControlSize { node_kind, .. } if node_kind == "Slider"
        ));
    }

    #[test]
    fn helper_layout_box_methods_apply_expected_constraints() {
        let layout = LayoutBox::auto()
            .fill_width()
            .fixed_height(24)
            .min(10, 20)
            .max(200, 30);
        assert_eq!(layout.width, Length::Fill(1));
        assert_eq!(layout.height, Length::Px(24));
        assert_eq!(layout.min_width, Some(10));
        assert_eq!(layout.min_height, Some(20));
        assert_eq!(layout.max_width, Some(200));
        assert_eq!(layout.max_height, Some(30));
    }

    #[test]
    fn helper_node_constructors_build_valid_spec() {
        let controls = row(vec![
            knob("drive", "Drive", 0.5, (0.0, 1.0)),
            slider("mix", "Mix", 0.25, (0.0, 1.0)),
            toggle("sync", "Sync", false),
            button("ping", "Ping"),
            dropdown("mode", "Mode", vec!["A".to_string(), "B".to_string()], 1),
        ]);
        let content = column(vec![
            label("Header"),
            controls,
            grid(
                GridTemplate::columns_fr(2).rows_fr(1).pad_all(4).gap(8),
                vec![
                    spacer(Size {
                        width: 8,
                        height: 8,
                    }),
                    indicator(
                        Size {
                            width: 8,
                            height: 8,
                        },
                        true,
                    ),
                ],
            ),
            region(
                "plot",
                Size {
                    width: 120,
                    height: 40,
                },
            ),
        ]);
        let spec = UiSpec::new(RootFrameSpec::new(
            "root",
            Node::Panel(PanelSpec::new("main", content).layout(LayoutBox::fill())),
        ));
        let measured = measure_checked(&spec).expect("helper-composed tree should validate");
        assert!(measured.width > 0);
        assert!(measured.height > 0);
    }

    #[test]
    fn render_emits_button_action() {
        let mut canvas = Canvas::new(200, 120);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState {
            pointer_pos: Point { x: 24, y: 24 },
            mouse_pressed: true,
            ..InputState::default()
        };
        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);

        let button = ButtonSpec::new("ok", "OK").control_size(Size {
            width: 80,
            height: 24,
        });
        let spec = UiSpec::new(RootFrameSpec::new(
            "root",
            Node::Panel(PanelSpec::new("panel", Node::Button(button))),
        ));

        let result =
            render_checked(&spec, &mut ui, Point { x: 0, y: 0 }).expect("render should succeed");
        assert!(
            result
                .actions
                .iter()
                .any(|action| matches!(action, UiAction::ButtonPressed { key } if key == "ok"))
        );
    }
}
