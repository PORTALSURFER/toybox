/// Measure a node's intrinsic content size.
fn measure_node(node: &Node, tokens: &ThemeTokens) -> Size {
    match node {
        Node::Slot(slot) => measure_node(&slot.child, tokens),
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
    let padding_total = padding.saturating_mul(2);
    let measured = Size {
        width: content.width.saturating_add(padding_total),
        height: content
            .height
            .saturating_add(padding_total)
            .saturating_add(header),
    };
    resolve_size(panel.layout.to_layout_box(), measured, measured)
}

/// Measure a flex container intrinsically.
fn measure_flex(flex: &FlexSpec, tokens: &ThemeTokens, axis: Axis) -> Size {
    let mut total_main = 0u64;
    let mut max_cross = 0u64;
    let mut child_count = 0u64;

    for child in &flex.children {
        let child_size = measure_node(child, tokens);
        let (main, cross) = match axis {
            Axis::Horizontal => (u64::from(child_size.width), u64::from(child_size.height)),
            Axis::Vertical => (u64::from(child_size.height), u64::from(child_size.width)),
        };
        total_main = total_main.saturating_add(main);
        max_cross = max_cross.max(cross);
        child_count = child_count.saturating_add(1);
    }

    let gap = u64::from(flex.gap.max(0) as u32);
    let gap_total = gap.saturating_mul(child_count.saturating_sub(1));
    total_main = total_main.saturating_add(gap_total);

    let (main_padding, cross_padding) = match axis {
        Axis::Horizontal => (
            i32_to_nonnegative_u64(flex.padding.left) + i32_to_nonnegative_u64(flex.padding.right),
            i32_to_nonnegative_u64(flex.padding.top) + i32_to_nonnegative_u64(flex.padding.bottom),
        ),
        Axis::Vertical => (
            i32_to_nonnegative_u64(flex.padding.top) + i32_to_nonnegative_u64(flex.padding.bottom),
            i32_to_nonnegative_u64(flex.padding.left) + i32_to_nonnegative_u64(flex.padding.right),
        ),
    };

    let measured = match axis {
        Axis::Horizontal => Size {
            width: total_main.saturating_add(main_padding).min(u32::MAX as u64) as u32,
            height: max_cross.saturating_add(cross_padding).min(u32::MAX as u64) as u32,
        },
        Axis::Vertical => Size {
            width: max_cross.saturating_add(cross_padding).min(u32::MAX as u64) as u32,
            height: total_main.saturating_add(main_padding).min(u32::MAX as u64) as u32,
        },
    };

    resolve_size(flex.layout.to_layout_box(), measured, measured)
}

/// Measure an absolute container intrinsically.
fn measure_absolute(absolute: &AbsoluteSpec, tokens: &ThemeTokens) -> Size {
    let mut max_x = 0i64;
    let mut max_y = 0i64;

    for child in &absolute.children {
        let size = measure_node(&child.node, tokens);
        let right = i64::from(child.origin.x) + i64::from(size.width);
        let bottom = i64::from(child.origin.y) + i64::from(size.height);
        max_x = max_x.max(right);
        max_y = max_y.max(bottom);
    }

    resolve_size(
        absolute.layout.to_layout_box(),
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

/// Convert a signed axis delta to a non-negative `u64` width contribution.
fn i32_to_nonnegative_u64(value: i32) -> u64 {
    value.max(0) as u64
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
    let (min, max) = normalize_axis_bounds(min, max, "layout-axis");
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

/// Emit a warning when min/max constraints are ordered invalidly.
fn normalize_axis_bounds(
    min: Option<u32>,
    max: Option<u32>,
    axis: &'static str,
) -> (Option<u32>, Option<u32>) {
    match (min, max) {
        (Some(min_value), Some(max_value)) if min_value > max_value => {
            emit_layout_bound_warning(axis, min_value, max_value);
            (
                Some(max_value),
                Some(max_value),
            )
        }
        _ => (min, max),
    }
}

#[cfg(feature = "layout-overflow-warnings")]
/// Emit a layout warning when axis min/max constraints are invalid and need
/// normalization.
fn emit_layout_bound_warning(axis: &'static str, min: u32, max: u32) {
    eprintln!("patchbay-gui warning: {axis} min ({min}) exceeds max ({max}); normalizing to {max}");
    debug_assert!(
        false,
        "{axis}: layout min constraint ({min}) exceeds max constraint ({max})"
    );
}

#[cfg(not(feature = "layout-overflow-warnings"))]
/// Compile-time assertion guard used when overflow diagnostics are disabled.
fn emit_layout_bound_warning(_axis: &'static str, _min: u32, _max: u32) {
    debug_assert!(
        false,
        "{_axis}: layout min constraint ({_min}) exceeds max constraint ({_max})"
    );
}
