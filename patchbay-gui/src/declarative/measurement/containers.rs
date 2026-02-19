/// Measure a node's intrinsic content size.
fn measure_node(node: &Node, tokens: &ThemeTokens) -> Size {
    match node {
        Node::Slot(slot) => measure_node(&slot.child, tokens),
        Node::Panel(panel) => measure_panel(panel, tokens),
        Node::PaddingBox(padding_box) => measure_padding_box(padding_box, tokens),
        Node::AlignBox(align_box) => measure_align_box(align_box, tokens),
        Node::AspectBox(aspect_box) => measure_aspect_box(aspect_box, tokens),
        Node::Row(flex) => measure_flex(flex, tokens, Axis::Horizontal),
        Node::Column(flex) => measure_flex(flex, tokens, Axis::Vertical),
        Node::Grid(grid) => measure_grid(grid, tokens),
        Node::Absolute(absolute) => measure_absolute(absolute, tokens),
        Node::Stack(stack) => measure_stack(stack, tokens),
        Node::ScrollView(scroll_view) => measure_scroll_view(scroll_view, tokens),
        Node::Wrap(wrap) => measure_wrap(wrap, tokens),
        Node::SwitchLayout(switch_layout) => measure_switch_layout(switch_layout, tokens),
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

/// Measure a padding-box container intrinsically.
fn measure_padding_box(padding_box: &PaddingBoxSpec, tokens: &ThemeTokens) -> Size {
    let content = measure_node(padding_box.content(), tokens);
    let measured = Size {
        width: content
            .width
            .saturating_add(padding_box.padding.left.max(0) as u32)
            .saturating_add(padding_box.padding.right.max(0) as u32),
        height: content
            .height
            .saturating_add(padding_box.padding.top.max(0) as u32)
            .saturating_add(padding_box.padding.bottom.max(0) as u32),
    };
    resolve_size(padding_box.layout.to_layout_box(), measured, measured)
}

/// Measure an align-box container intrinsically.
fn measure_align_box(align_box: &AlignBoxSpec, tokens: &ThemeTokens) -> Size {
    let measured = measure_node(align_box.content(), tokens);
    resolve_size(align_box.layout.to_layout_box(), measured, measured)
}

/// Measure an aspect-box container intrinsically.
fn measure_aspect_box(aspect_box: &AspectBoxSpec, tokens: &ThemeTokens) -> Size {
    let content = measure_node(aspect_box.content(), tokens);
    let measured = expand_size_to_aspect_containing(content, aspect_box.aspect_ratio);
    resolve_size(aspect_box.layout.to_layout_box(), measured, measured)
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

/// Measure a stack container intrinsically.
fn measure_stack(stack: &StackSpec, tokens: &ThemeTokens) -> Size {
    let mut max_width = 0u32;
    let mut max_height = 0u32;
    for child in &stack.children {
        let measured = measure_node(child, tokens);
        max_width = max_width.max(measured.width);
        max_height = max_height.max(measured.height);
    }
    let measured = Size {
        width: max_width
            .saturating_add(stack.padding.left.max(0) as u32)
            .saturating_add(stack.padding.right.max(0) as u32),
        height: max_height
            .saturating_add(stack.padding.top.max(0) as u32)
            .saturating_add(stack.padding.bottom.max(0) as u32),
    };
    resolve_size(stack.layout.to_layout_box(), measured, measured)
}

/// Measure a scroll-view container intrinsically.
fn measure_scroll_view(scroll_view: &ScrollViewSpec, tokens: &ThemeTokens) -> Size {
    let content = measure_node(scroll_view.content(), tokens);
    let measured = Size {
        width: content
            .width
            .saturating_add(scroll_view.padding.left.max(0) as u32)
            .saturating_add(scroll_view.padding.right.max(0) as u32),
        height: content
            .height
            .saturating_add(scroll_view.padding.top.max(0) as u32)
            .saturating_add(scroll_view.padding.bottom.max(0) as u32),
    };
    resolve_size(scroll_view.layout.to_layout_box(), measured, measured)
}

/// Measure a wrap container intrinsically.
fn measure_wrap(wrap: &WrapSpec, tokens: &ThemeTokens) -> Size {
    let mut total_width = 0u64;
    let mut max_height = 0u64;
    let mut child_count = 0u64;
    for child in &wrap.children {
        let measured = measure_node(child, tokens);
        total_width = total_width.saturating_add(u64::from(measured.width));
        max_height = max_height.max(u64::from(measured.height));
        child_count = child_count.saturating_add(1);
    }
    let gap_total = u64::from(wrap.column_gap.max(0) as u32).saturating_mul(child_count.saturating_sub(1));
    let measured = Size {
        width: total_width
            .saturating_add(gap_total)
            .saturating_add(wrap.padding.left.max(0) as u64)
            .saturating_add(wrap.padding.right.max(0) as u64)
            .min(u64::from(u32::MAX)) as u32,
        height: max_height
            .saturating_add(wrap.padding.top.max(0) as u64)
            .saturating_add(wrap.padding.bottom.max(0) as u64)
            .min(u64::from(u32::MAX)) as u32,
    };
    resolve_size(wrap.layout.to_layout_box(), measured, measured)
}

/// Measure a switch-layout container intrinsically.
///
/// Measurement uses the maximal case extent so initial host sizing can
/// accommodate any breakpoint variant deterministically.
fn measure_switch_layout(switch_layout: &SwitchLayoutSpec, tokens: &ThemeTokens) -> Size {
    let mut measured = measure_node(switch_layout.fallback(), tokens);
    for case in switch_layout.cases() {
        let case_measured = measure_node(case.child(), tokens);
        measured.width = measured.width.max(case_measured.width);
        measured.height = measured.height.max(case_measured.height);
    }
    resolve_size(
        switch_layout.layout.to_layout_box(),
        measured,
        measured,
    )
}

/// Convert a signed axis delta to a non-negative `u64` width contribution.
fn i32_to_nonnegative_u64(value: i32) -> u64 {
    value.max(0) as u64
}

/// Expand one size to satisfy an aspect ratio while containing original bounds.
fn expand_size_to_aspect_containing(size: Size, aspect_ratio: AspectRatio) -> Size {
    if aspect_ratio.width == 0 || aspect_ratio.height == 0 {
        return size;
    }
    let lhs = u64::from(size.width).saturating_mul(u64::from(aspect_ratio.height));
    let rhs = u64::from(size.height).saturating_mul(u64::from(aspect_ratio.width));
    if lhs >= rhs {
        let height = ceil_div_u64(
            u64::from(size.width).saturating_mul(u64::from(aspect_ratio.height)),
            u64::from(aspect_ratio.width),
        )
        .min(u64::from(u32::MAX)) as u32;
        Size {
            width: size.width,
            height,
        }
    } else {
        let width = ceil_div_u64(
            u64::from(size.height).saturating_mul(u64::from(aspect_ratio.width)),
            u64::from(aspect_ratio.height),
        )
        .min(u64::from(u32::MAX)) as u32;
        Size {
            width,
            height: size.height,
        }
    }
}

/// Divide integers with rounding up.
fn ceil_div_u64(value: u64, divisor: u64) -> u64 {
    if divisor == 0 {
        return value;
    }
    value
        .saturating_add(divisor.saturating_sub(1))
        .saturating_div(divisor)
}
