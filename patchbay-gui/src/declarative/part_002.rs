
impl Node {
    /// Create a row container.
    pub fn row(children: Vec<Node>) -> Self {
        Self::Row(FlexSpec::row(children))
    }

    /// Create a column container.
    pub fn column(children: Vec<Node>) -> Self {
        Self::Column(FlexSpec::column(children))
    }

    /// Apply layout constraints to nodes that support [`LayoutBox`].
    ///
    /// Nodes with intrinsic fixed size (`Spacer`, `Region`, `Indicator`) ignore
    /// this request and are returned unchanged.
    pub fn layout(mut self, layout: LayoutBox) -> Self {
        match &mut self {
            Self::Panel(panel) => panel.layout = layout,
            Self::Row(flex) | Self::Column(flex) => flex.layout = layout,
            Self::Grid(grid) => grid.layout = layout,
            Self::Absolute(absolute) => absolute.layout = layout,
            Self::Label(label) => label.layout = layout,
            Self::Knob(knob) => knob.layout = layout,
            Self::Slider(slider) => slider.layout = layout,
            Self::Toggle(toggle) => toggle.layout = layout,
            Self::Button(button) => button.layout = layout,
            Self::Dropdown(dropdown) => dropdown.layout = layout,
            Self::Spacer(_) | Self::Region(_) | Self::Indicator(_) => {}
        }
        self
    }

    /// Set node layout to fill available width and height where supported.
    pub fn fill(self) -> Self {
        self.layout(LayoutBox::fill())
    }

    /// Set node layout to fill available width where supported.
    pub fn fill_width(self) -> Self {
        self.layout(LayoutBox::auto().fill_width())
    }

    /// Set node layout to fill available height where supported.
    pub fn fill_height(self) -> Self {
        self.layout(LayoutBox::auto().fill_height())
    }

    /// Set container gap for row/column/grid nodes.
    ///
    /// For grid nodes this sets both column and row gaps. Other node kinds are
    /// returned unchanged.
    pub fn gap(mut self, gap: i32) -> Self {
        match &mut self {
            Self::Row(flex) | Self::Column(flex) => {
                flex.gap = gap;
            }
            Self::Grid(grid) => {
                grid.template.column_gap = gap;
                grid.template.row_gap = gap;
            }
            _ => {}
        }
        self
    }

    /// Set independent column/row gaps for grid nodes.
    ///
    /// Non-grid node kinds are returned unchanged.
    pub fn gap_xy(mut self, column_gap: i32, row_gap: i32) -> Self {
        if let Self::Grid(grid) = &mut self {
            grid.template.column_gap = column_gap;
            grid.template.row_gap = row_gap;
        }
        self
    }

    /// Set uniform padding for panel/flex/grid nodes.
    ///
    /// Non-container node kinds are returned unchanged.
    pub fn pad_all(mut self, value: i32) -> Self {
        match &mut self {
            Self::Panel(panel) => panel.padding = value,
            Self::Row(flex) | Self::Column(flex) => flex.padding = EdgeInsets::all(value),
            Self::Grid(grid) => grid.template.padding = EdgeInsets::all(value),
            _ => {}
        }
        self
    }

    /// Set horizontal/vertical padding for flex/grid nodes.
    ///
    /// Panel and non-container node kinds are returned unchanged.
    pub fn pad_xy(mut self, horizontal: i32, vertical: i32) -> Self {
        match &mut self {
            Self::Row(flex) | Self::Column(flex) => {
                flex.padding = EdgeInsets::symmetric(horizontal, vertical)
            }
            Self::Grid(grid) => grid.template.padding = EdgeInsets::symmetric(horizontal, vertical),
            _ => {}
        }
        self
    }

    /// Set cross-axis alignment for row/column nodes.
    ///
    /// Non-flex node kinds are returned unchanged.
    pub fn align(mut self, align: Align) -> Self {
        if let Self::Row(flex) | Self::Column(flex) = &mut self {
            flex.align = align;
        }
        self
    }

    /// Align row/column children to cross-axis start.
    pub fn align_start(self) -> Self {
        self.align(Align::Start)
    }

    /// Center row/column children on the cross-axis.
    pub fn align_center(self) -> Self {
        self.align(Align::Center)
    }

    /// Align row/column children to cross-axis end.
    pub fn align_end(self) -> Self {
        self.align(Align::End)
    }

    /// Stretch row/column children across the cross-axis.
    pub fn align_stretch(self) -> Self {
        self.align(Align::Stretch)
    }

    /// Set main-axis distribution for row/column nodes.
    ///
    /// Non-flex node kinds are returned unchanged.
    pub fn justify(mut self, justify: Justify) -> Self {
        if let Self::Row(flex) | Self::Column(flex) = &mut self {
            flex.justify = justify;
        }
        self
    }

    /// Pack row/column children at main-axis start.
    pub fn justify_start(self) -> Self {
        self.justify(Justify::Start)
    }

    /// Center row/column children on the main axis.
    pub fn justify_center(self) -> Self {
        self.justify(Justify::Center)
    }

    /// Pack row/column children at main-axis end.
    pub fn justify_end(self) -> Self {
        self.justify(Justify::End)
    }

    /// Distribute row/column spacing between children.
    pub fn justify_space_between(self) -> Self {
        self.justify(Justify::SpaceBetween)
    }

    /// Distribute row/column spacing around children.
    pub fn justify_space_around(self) -> Self {
        self.justify(Justify::SpaceAround)
    }

    /// Distribute row/column spacing evenly including edges.
    pub fn justify_space_evenly(self) -> Self {
        self.justify(Justify::SpaceEvenly)
    }

    /// Set title for panel nodes.
    ///
    /// Non-panel node kinds are returned unchanged.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        if let Self::Panel(panel) = &mut self {
            panel.title = Some(title.into());
        }
        self
    }

    /// Set background color for panel nodes.
    ///
    /// Non-panel node kinds are returned unchanged.
    pub fn background(mut self, color: Color) -> Self {
        if let Self::Panel(panel) = &mut self {
            panel.background = Some(color);
        }
        self
    }

    /// Set outline color for panel nodes.
    ///
    /// Non-panel node kinds are returned unchanged.
    pub fn outline(mut self, color: Color) -> Self {
        if let Self::Panel(panel) = &mut self {
            panel.outline = Some(color);
        }
        self
    }

    /// Set text color for label nodes.
    ///
    /// Non-label node kinds are returned unchanged.
    pub fn text_color(mut self, color: Color) -> Self {
        if let Self::Label(label) = &mut self {
            label.color = Some(color);
        }
        self
    }

    /// Set explicit control size for slider/toggle/button/dropdown nodes.
    ///
    /// Non-control node kinds and knobs are returned unchanged.
    pub fn control_size(mut self, size: Size) -> Self {
        match &mut self {
            Self::Slider(slider) => slider.control_size = Some(size),
            Self::Toggle(toggle) => toggle.control_size = Some(size),
            Self::Button(button) => button.control_size = Some(size),
            Self::Dropdown(dropdown) => dropdown.control_size = Some(size),
            _ => {}
        }
        self
    }

    /// Set value label text for knob nodes.
    ///
    /// Non-knob node kinds are returned unchanged.
    pub fn value_label(mut self, value_label: impl Into<String>) -> Self {
        if let Self::Knob(knob) = &mut self {
            knob.value_label = Some(value_label.into());
        }
        self
    }

    /// Set selected option index for dropdown nodes.
    ///
    /// Non-dropdown node kinds are returned unchanged.
    pub fn selected(mut self, selected: usize) -> Self {
        if let Self::Dropdown(dropdown) = &mut self {
            dropdown.selected = selected;
        }
        self
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

/// Section sizing mode for canonical row/column section layouts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectionSize {
    /// Fractional section sized as a percentage of parent bounds.
    ///
    /// Percentages are resolved first against the total parent extent.
    Fraction(u8),
    /// Fill section that shares remaining space with other fill siblings.
    Fill,
}

/// Child node paired with a section sizing definition.
#[derive(Clone, Debug)]
pub struct SectionChild {
    /// Child node rendered inside the section.
    pub node: Node,
    /// Section sizing definition.
    pub size: SectionSize,
}

/// Backward-compatible alias for section children.
pub type WeightedChild = SectionChild;

/// Create a fraction-based section child.
pub fn fraction(node: Node, percent: u8) -> SectionChild {
    SectionChild {
        node,
        size: SectionSize::Fraction(percent),
    }
}

/// Create a fill section child.
pub fn fill_section(node: Node) -> SectionChild {
    SectionChild {
        node,
        size: SectionSize::Fill,
    }
}

/// Backward-compatible weighted section helper.
///
/// This maps weights into fractional percentages for strict section sizing.
pub fn weighted(node: Node, weight: u16) -> WeightedChild {
    let percent = weight.clamp(1, 100) as u8;
    fraction(node, percent)
}
