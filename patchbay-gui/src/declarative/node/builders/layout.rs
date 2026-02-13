impl Node {
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
}
