
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

    let column_gap = grid.template.column_gap.max(0) as u32;
    let row_gap = grid.template.row_gap.max(0) as u32;
    let width = column_widths.iter().copied().sum::<u32>()
        + column_gap.saturating_mul(column_widths.len().saturating_sub(1) as u32)
        + grid.template.padding.left.max(0) as u32
        + grid.template.padding.right.max(0) as u32;
    let height = row_heights.iter().copied().sum::<u32>()
        + row_gap.saturating_mul(row_heights.len().saturating_sub(1) as u32)
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
    let measured = knob_block_size_for_diameter(control, tokens.typography.text_scale);
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

/// Mutable render-state threaded through declarative node traversal.
struct RenderCtx<'a> {
    /// Theme tokens used for sizing and rendering.
    tokens: &'a ThemeTokens,
    /// Collected UI actions for the current frame.
    actions: &'a mut Vec<UiAction>,
    /// Candidate container rectangles for debug-border selection.
    debug_border_candidates: &'a mut Vec<DebugBorderCandidate>,
    /// Current container depth in the render tree.
    depth: usize,
}

/// Render a node subtree and collect actions.
fn render_node(node: &Node, rect: Rect, ui: &mut Ui<'_>, ctx: &mut RenderCtx<'_>) {
    ui.with_clip(rect, |ui| match node {
        Node::Panel(panel) => render_panel(panel, rect, ui, ctx),
        Node::Row(flex) => render_flex(flex, rect, ui, Axis::Horizontal, ctx),
        Node::Column(flex) => render_flex(flex, rect, ui, Axis::Vertical, ctx),
        Node::Grid(grid) => render_grid(grid, rect, ui, ctx),
        Node::Absolute(absolute) => render_absolute(absolute, rect, ui, ctx),
        Node::Label(label) => render_label(label, rect, ui, ctx.tokens),
        Node::Spacer(_) => {}
        Node::Knob(knob) => render_knob(knob, rect, ui, ctx.tokens, ctx.actions),
        Node::Slider(slider) => render_slider(slider, rect, ui, ctx.tokens, ctx.actions),
        Node::Toggle(toggle) => render_toggle(toggle, rect, ui, ctx.tokens, ctx.actions),
        Node::Button(button) => render_button(button, rect, ui, ctx.tokens, ctx.actions),
        Node::Dropdown(dropdown) => render_dropdown(dropdown, rect, ui, ctx.tokens, ctx.actions),
        Node::Region(region) => render_region(region, rect, ui, ctx.actions),
        Node::Indicator(indicator) => render_indicator(indicator, rect, ui),
    });
}
