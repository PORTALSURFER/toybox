impl Node {
    /// Apply layout constraints to widget nodes.
    ///
    /// Container nodes are unchanged. Use [`Node::container_layout`] for
    /// container sizing.
    /// Nodes with intrinsic fixed size (`Spacer`, `Region`, `Indicator`) ignore
    /// this request and are returned unchanged.
    pub fn widget_layout(mut self, layout: LayoutBox) -> Self {
        match &mut self {
            Self::Slot(_) | Self::Panel(_) | Self::Row(_) | Self::Column(_) | Self::Grid(_) | Self::Absolute(_) => {}
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

    /// Apply layout constraints to container nodes.
    ///
    /// Widget nodes are unchanged. Use [`Node::widget_layout`] for widget
    /// sizing.
    pub fn container_layout(mut self, layout: ContainerLayout) -> Self {
        match &mut self {
            Self::Panel(panel) => panel.layout = layout,
            Self::Row(flex) | Self::Column(flex) => flex.layout = layout,
            Self::Grid(grid) => grid.layout = layout,
            Self::Absolute(absolute) => absolute.layout = layout,
            _ => {}
        }
        self
    }

    /// Set node layout to fill available width and height where supported.
    pub fn fill(mut self) -> Self {
        match &mut self {
            Self::Panel(panel) => panel.layout = ContainerLayout::fill(),
            Self::Row(flex) | Self::Column(flex) => flex.layout = ContainerLayout::fill(),
            Self::Grid(grid) => grid.layout = ContainerLayout::fill(),
            Self::Absolute(absolute) => absolute.layout = ContainerLayout::fill(),
            Self::Label(label) => label.layout = LayoutBox::fill(),
            Self::Knob(knob) => knob.layout = LayoutBox::fill(),
            Self::Slider(slider) => slider.layout = LayoutBox::fill(),
            Self::Toggle(toggle) => toggle.layout = LayoutBox::fill(),
            Self::Button(button) => button.layout = LayoutBox::fill(),
            Self::Dropdown(dropdown) => dropdown.layout = LayoutBox::fill(),
            Self::Slot(_) | Self::Spacer(_) | Self::Region(_) | Self::Indicator(_) => {}
        }
        self
    }

    /// Set node layout to fill available width where supported.
    pub fn fill_width(mut self) -> Self {
        match &mut self {
            Self::Panel(panel) => panel.layout = ContainerLayout::auto().fill_width(),
            Self::Row(flex) | Self::Column(flex) => {
                flex.layout = ContainerLayout::auto().fill_width()
            }
            Self::Grid(grid) => grid.layout = ContainerLayout::auto().fill_width(),
            Self::Absolute(absolute) => absolute.layout = ContainerLayout::auto().fill_width(),
            Self::Label(label) => label.layout = LayoutBox::auto().fill_width(),
            Self::Knob(knob) => knob.layout = LayoutBox::auto().fill_width(),
            Self::Slider(slider) => slider.layout = LayoutBox::auto().fill_width(),
            Self::Toggle(toggle) => toggle.layout = LayoutBox::auto().fill_width(),
            Self::Button(button) => button.layout = LayoutBox::auto().fill_width(),
            Self::Dropdown(dropdown) => dropdown.layout = LayoutBox::auto().fill_width(),
            Self::Slot(_) | Self::Spacer(_) | Self::Region(_) | Self::Indicator(_) => {}
        }
        self
    }

    /// Set node layout to fill available height where supported.
    pub fn fill_height(mut self) -> Self {
        match &mut self {
            Self::Panel(panel) => panel.layout = ContainerLayout::auto().fill_height(),
            Self::Row(flex) | Self::Column(flex) => {
                flex.layout = ContainerLayout::auto().fill_height()
            }
            Self::Grid(grid) => grid.layout = ContainerLayout::auto().fill_height(),
            Self::Absolute(absolute) => absolute.layout = ContainerLayout::auto().fill_height(),
            Self::Label(label) => label.layout = LayoutBox::auto().fill_height(),
            Self::Knob(knob) => knob.layout = LayoutBox::auto().fill_height(),
            Self::Slider(slider) => slider.layout = LayoutBox::auto().fill_height(),
            Self::Toggle(toggle) => toggle.layout = LayoutBox::auto().fill_height(),
            Self::Button(button) => button.layout = LayoutBox::auto().fill_height(),
            Self::Dropdown(dropdown) => dropdown.layout = LayoutBox::auto().fill_height(),
            Self::Slot(_) | Self::Spacer(_) | Self::Region(_) | Self::Indicator(_) => {}
        }
        self
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
