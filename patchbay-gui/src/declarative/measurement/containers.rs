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
        Length::Px(px) => px.max(measured),
        Length::Fill(_) => available,
    };
    let min_applied = base.max(min.unwrap_or(0));
    if let Some(max_value) = max {
        min_applied.min(max_value)
    } else {
        min_applied
    }
}
