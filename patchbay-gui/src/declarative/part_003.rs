
/// Resolve weighted section lengths that exactly consume the available space.
///
/// This uses the same deterministic largest-remainder distribution strategy as
/// Patchbay grid `Fr` track allocation so section math stays consistent between
/// plugin-side sizing helpers and renderer-side layout.
///
/// Weights are clamped to at least `1` to match [`weighted`].
pub fn weighted_section_lengths(total: u32, weights: &[u16]) -> Vec<u32> {
    let total_percent: u16 = weights.iter().copied().sum();
    if total_percent == 0 {
        return vec![0; weights.len()];
    }
    let target_total = total
        .saturating_mul(total_percent as u32)
        .saturating_div(100);
    let normalized: Vec<u32> = weights.iter().map(|weight| u32::from(*weight)).collect();
    distribute_weighted_u32(target_total, &normalized)
}

/// Create a weighted full-size column section layout.
///
/// Children fill width and split available height by relative `weight`.
pub fn column_sections(children: Vec<SectionChild>) -> Node {
    let rows: Vec<TrackSize> = children
        .iter()
        .map(|child| match child.size {
            SectionSize::Fraction(percent) => TrackSize::Percent(percent),
            SectionSize::Fill => TrackSize::Fill,
        })
        .collect();
    let nodes: Vec<Node> = children
        .into_iter()
        .map(|child| child.node.layout(LayoutBox::fill()))
        .collect();
    let mut spec = GridSpec::new(
        GridTemplate::new(vec![TrackSize::Fr(1)])
            .rows(rows)
            .gap(0)
            .pad_all(0)
            .justify_start(),
        nodes,
    );
    spec.kind = GridKind::SectionColumn;
    Node::Grid(spec).layout(LayoutBox::fill())
}

/// Create a weighted full-size row section layout.
///
/// Children fill height and split available width by relative `weight`.
pub fn row_sections(children: Vec<SectionChild>) -> Node {
    let columns: Vec<TrackSize> = children
        .iter()
        .map(|child| match child.size {
            SectionSize::Fraction(percent) => TrackSize::Percent(percent),
            SectionSize::Fill => TrackSize::Fill,
        })
        .collect();
    let nodes: Vec<Node> = children
        .into_iter()
        .map(|child| child.node.layout(LayoutBox::fill()))
        .collect();
    let mut spec = GridSpec::new(
        GridTemplate::new(columns)
            .rows(vec![TrackSize::Fr(1)])
            .gap(0)
            .pad_all(0)
            .justify_start(),
        nodes,
    );
    spec.kind = GridKind::SectionRow;
    Node::Grid(spec).layout(LayoutBox::fill())
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

    /// Create a fixed-size baseline box.
    ///
    /// The returned constraints use fixed pixel lengths as minimum floors.
    /// Content can still grow beyond these values when intrinsic measurement
    /// requires more space.
    pub const fn fixed(width: u32, height: u32) -> Self {
        Self {
            width: Length::Px(width),
            height: Length::Px(height),
            min_width: Some(width),
            min_height: Some(height),
            max_width: None,
            max_height: None,
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

    /// Set a fixed-width baseline while preserving current height behavior.
    ///
    /// The width acts as a minimum floor and may expand for larger intrinsic
    /// content unless an explicit max width is also applied.
    pub const fn fixed_width(mut self, width: u32) -> Self {
        self.width = Length::Px(width);
        self.min_width = Some(width);
        self.max_width = None;
        self
    }

    /// Set a fixed-height baseline while preserving current width behavior.
    ///
    /// The height acts as a minimum floor and may expand for larger intrinsic
    /// content unless an explicit max height is also applied.
    pub const fn fixed_height(mut self, height: u32) -> Self {
        self.height = Length::Px(height);
        self.min_height = Some(height);
        self.max_height = None;
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
