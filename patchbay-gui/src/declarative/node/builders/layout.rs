impl Node {
    /// Apply layout constraints to widget nodes.
    ///
    /// Container nodes are unchanged. Use [`Node::container_layout`] for
    /// container sizing.
    /// Nodes with intrinsic fixed size (`Spacer`, `Region`, `Indicator`) ignore
    /// this request and are returned unchanged.
    pub fn widget_layout(mut self, layout: LayoutBox) -> Self {
        match &mut self {
            Self::Slot(_)
            | Self::Panel(_)
            | Self::PaddingBox(_)
            | Self::AlignBox(_)
            | Self::AspectBox(_)
            | Self::Row(_)
            | Self::Column(_)
            | Self::Grid(_)
            | Self::Absolute(_)
            | Self::Stack(_)
            | Self::ScrollView(_)
            | Self::Wrap(_)
            | Self::SwitchLayout(_) => {}
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
            Self::PaddingBox(padding_box) => padding_box.layout = layout,
            Self::AlignBox(align_box) => align_box.layout = layout,
            Self::AspectBox(aspect_box) => aspect_box.layout = layout,
            Self::Row(flex) | Self::Column(flex) => flex.layout = layout,
            Self::Grid(grid) => grid.layout = layout,
            Self::Absolute(absolute) => absolute.layout = layout,
            Self::Stack(stack) => stack.layout = layout,
            Self::ScrollView(scroll_view) => scroll_view.layout = layout,
            Self::Wrap(wrap) => wrap.layout = layout,
            Self::SwitchLayout(switch_layout) => switch_layout.layout = layout,
            _ => {}
        }
        self
    }

    /// Set node layout to fill available width and height where supported.
    pub fn fill(mut self) -> Self {
        match &mut self {
            Self::Panel(panel) => panel.layout = panel.layout.fill_width().fill_height(),
            Self::PaddingBox(padding_box) => {
                padding_box.layout = padding_box.layout.fill_width().fill_height()
            }
            Self::AlignBox(align_box) => {
                align_box.layout = align_box.layout.fill_width().fill_height()
            }
            Self::AspectBox(aspect_box) => {
                aspect_box.layout = aspect_box.layout.fill_width().fill_height()
            }
            Self::Row(flex) | Self::Column(flex) => {
                flex.layout = flex.layout.fill_width().fill_height()
            }
            Self::Grid(grid) => grid.layout = grid.layout.fill_width().fill_height(),
            Self::Absolute(absolute) => {
                absolute.layout = absolute.layout.fill_width().fill_height()
            }
            Self::Stack(stack) => stack.layout = stack.layout.fill_width().fill_height(),
            Self::ScrollView(scroll_view) => {
                scroll_view.layout = scroll_view.layout.fill_width().fill_height()
            }
            Self::Wrap(wrap) => wrap.layout = wrap.layout.fill_width().fill_height(),
            Self::SwitchLayout(switch_layout) => {
                switch_layout.layout = switch_layout.layout.fill_width().fill_height()
            }
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
            Self::Panel(panel) => panel.layout = panel.layout.fill_width(),
            Self::PaddingBox(padding_box) => padding_box.layout = padding_box.layout.fill_width(),
            Self::AlignBox(align_box) => align_box.layout = align_box.layout.fill_width(),
            Self::AspectBox(aspect_box) => aspect_box.layout = aspect_box.layout.fill_width(),
            Self::Row(flex) | Self::Column(flex) => flex.layout = flex.layout.fill_width(),
            Self::Grid(grid) => grid.layout = grid.layout.fill_width(),
            Self::Absolute(absolute) => absolute.layout = absolute.layout.fill_width(),
            Self::Stack(stack) => stack.layout = stack.layout.fill_width(),
            Self::ScrollView(scroll_view) => scroll_view.layout = scroll_view.layout.fill_width(),
            Self::Wrap(wrap) => wrap.layout = wrap.layout.fill_width(),
            Self::SwitchLayout(switch_layout) => {
                switch_layout.layout = switch_layout.layout.fill_width()
            }
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
            Self::Panel(panel) => panel.layout = panel.layout.fill_height(),
            Self::PaddingBox(padding_box) => {
                padding_box.layout = padding_box.layout.fill_height()
            }
            Self::AlignBox(align_box) => align_box.layout = align_box.layout.fill_height(),
            Self::AspectBox(aspect_box) => aspect_box.layout = aspect_box.layout.fill_height(),
            Self::Row(flex) | Self::Column(flex) => flex.layout = flex.layout.fill_height(),
            Self::Grid(grid) => grid.layout = grid.layout.fill_height(),
            Self::Absolute(absolute) => absolute.layout = absolute.layout.fill_height(),
            Self::Stack(stack) => stack.layout = stack.layout.fill_height(),
            Self::ScrollView(scroll_view) => {
                scroll_view.layout = scroll_view.layout.fill_height()
            }
            Self::Wrap(wrap) => wrap.layout = wrap.layout.fill_height(),
            Self::SwitchLayout(switch_layout) => {
                switch_layout.layout = switch_layout.layout.fill_height()
            }
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

    /// Set overflow behavior for container nodes.
    ///
    /// Widget and slot node kinds are returned unchanged.
    pub fn container_overflow(mut self, overflow_policy: OverflowPolicy) -> Self {
        match &mut self {
            Self::Panel(panel) => panel.layout = panel.layout.overflow(overflow_policy),
            Self::PaddingBox(padding_box) => {
                padding_box.layout = padding_box.layout.overflow(overflow_policy)
            }
            Self::AlignBox(align_box) => align_box.layout = align_box.layout.overflow(overflow_policy),
            Self::AspectBox(aspect_box) => {
                aspect_box.layout = aspect_box.layout.overflow(overflow_policy)
            }
            Self::Row(flex) | Self::Column(flex) => {
                flex.layout = flex.layout.overflow(overflow_policy)
            }
            Self::Grid(grid) => grid.layout = grid.layout.overflow(overflow_policy),
            Self::Absolute(absolute) => {
                absolute.layout = absolute.layout.overflow(overflow_policy)
            }
            Self::Stack(stack) => stack.layout = stack.layout.overflow(overflow_policy),
            Self::ScrollView(scroll_view) => {
                scroll_view.layout = scroll_view.layout.overflow(overflow_policy)
            }
            Self::Wrap(wrap) => wrap.layout = wrap.layout.overflow(overflow_policy),
            Self::SwitchLayout(switch_layout) => {
                switch_layout.layout = switch_layout.layout.overflow(overflow_policy)
            }
            _ => {}
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
            Self::Wrap(wrap) => {
                wrap.column_gap = gap;
                wrap.row_gap = gap;
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
        } else if let Self::Wrap(wrap) = &mut self {
            wrap.column_gap = column_gap;
            wrap.row_gap = row_gap;
        }
        self
    }

    /// Set uniform padding for padding-capable container nodes.
    ///
    /// Non-container node kinds are returned unchanged.
    pub fn pad_all(mut self, value: i32) -> Self {
        match &mut self {
            Self::Panel(panel) => panel.padding = value,
            Self::PaddingBox(padding_box) => padding_box.padding = EdgeInsets::all(value),
            Self::Row(flex) | Self::Column(flex) => flex.padding = EdgeInsets::all(value),
            Self::Grid(grid) => grid.template.padding = EdgeInsets::all(value),
            Self::Stack(stack) => stack.padding = EdgeInsets::all(value),
            Self::ScrollView(scroll_view) => scroll_view.padding = EdgeInsets::all(value),
            Self::Wrap(wrap) => wrap.padding = EdgeInsets::all(value),
            _ => {}
        }
        self
    }

    /// Set horizontal/vertical padding for padding-capable container nodes.
    ///
    /// Panel and non-container node kinds are returned unchanged.
    pub fn pad_xy(mut self, horizontal: i32, vertical: i32) -> Self {
        match &mut self {
            Self::PaddingBox(padding_box) => {
                padding_box.padding = EdgeInsets::symmetric(horizontal, vertical)
            }
            Self::Row(flex) | Self::Column(flex) => {
                flex.padding = EdgeInsets::symmetric(horizontal, vertical)
            }
            Self::Grid(grid) => grid.template.padding = EdgeInsets::symmetric(horizontal, vertical),
            Self::Stack(stack) => stack.padding = EdgeInsets::symmetric(horizontal, vertical),
            Self::ScrollView(scroll_view) => {
                scroll_view.padding = EdgeInsets::symmetric(horizontal, vertical)
            }
            Self::Wrap(wrap) => wrap.padding = EdgeInsets::symmetric(horizontal, vertical),
            _ => {}
        }
        self
    }

    /// Set slot-alignment for single-slot overlay/alignment containers.
    ///
    /// Applies to `AlignBox`, `AspectBox`, and `Stack`; other node kinds are unchanged.
    pub fn slot_align(mut self, align_x: SlotAlign, align_y: SlotAlign) -> Self {
        match &mut self {
            Self::AlignBox(align_box) => {
                align_box.align_x = align_x;
                align_box.align_y = align_y;
            }
            Self::AspectBox(aspect_box) => {
                aspect_box.align_x = align_x;
                aspect_box.align_y = align_y;
            }
            Self::Stack(stack) => {
                stack.align_x = align_x;
                stack.align_y = align_y;
            }
            _ => {}
        }
        self
    }

    /// Set aspect-ratio components for `AspectBox` nodes.
    ///
    /// Non-`AspectBox` node kinds are returned unchanged.
    pub fn aspect_ratio(mut self, width: u32, height: u32) -> Self {
        if let Self::AspectBox(aspect_box) = &mut self {
            aspect_box.aspect_ratio = AspectRatio::new(width, height);
        }
        self
    }
}
