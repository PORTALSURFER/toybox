/// Measure root frame size through the persistent layout engine cache.
fn measure_root_frame_with_engine(
    frame: &RootFrameSpec,
    tokens: &ThemeTokens,
    engine: &mut LayoutEngineState,
) -> Size {
    let root_id = engine
        .root_node_id()
        .unwrap_or_else(|| NodeId::from_hash(stable_debug_hash(&format!("root-frame:{}", frame.key))));
    let token_hash = LayoutEngineState::token_hash(tokens);
    let root_hash = stable_debug_hash(frame);
    let key = MeasureCacheKey {
        node_id: root_id,
        node_hash: root_hash,
        token_hash,
    };
    engine.resolve_cached_subtree_measure(key, |engine| {
        let content_id = engine.child_node_id(root_id, 0).unwrap_or(root_id);
        let content = measure_node_cached(&frame.content, content_id, tokens, token_hash, engine);
        let header = panel_header_height(frame.title.as_deref(), tokens).max(0) as u32;
        let padding = frame.padding.max(0) as u32;
        let padding_total = padding.saturating_mul(2);
        let measured = Size {
            width: content.width.saturating_add(padding_total),
            height: content
                .height
                .saturating_add(padding_total)
                .saturating_add(header),
        };
        resolve_size(frame.layout, measured, measured)
    })
}

/// Measure one node subtree through the layout engine cache.
fn measure_node_cached(
    node: &Node,
    node_id: NodeId,
    tokens: &ThemeTokens,
    token_hash: u64,
    engine: &mut LayoutEngineState,
) -> Size {
    let node_hash = engine.node_hash(node_id).unwrap_or_else(|| stable_debug_hash(node));
    let key = MeasureCacheKey {
        node_id,
        node_hash,
        token_hash,
    };
    engine.resolve_cached_subtree_measure(key, |engine| match node {
        Node::Slot(slot) => {
            let child_id = engine.child_node_id(node_id, 0).unwrap_or(node_id);
            measure_node_cached(slot.child(), child_id, tokens, token_hash, engine)
        }
        Node::Panel(panel) => {
            let child_id = engine.child_node_id(node_id, 0).unwrap_or(node_id);
            let content = measure_node_cached(&panel.content, child_id, tokens, token_hash, engine);
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
        Node::Row(flex) => measure_flex_cached(flex, node_id, tokens, token_hash, Axis::Horizontal, engine),
        Node::Column(flex) => {
            measure_flex_cached(flex, node_id, tokens, token_hash, Axis::Vertical, engine)
        }
        Node::Grid(grid) => measure_grid_cached(grid, node_id, tokens, token_hash, engine),
        Node::Absolute(absolute) => {
            let mut max_x = 0i64;
            let mut max_y = 0i64;
            for (index, child) in absolute.children.iter().enumerate() {
                let child_id = engine.child_node_id(node_id, index).unwrap_or(node_id);
                let size = measure_node_cached(&child.node, child_id, tokens, token_hash, engine);
                let right = i64::from(child.origin.x) + i64::from(size.width);
                let bottom = i64::from(child.origin.y) + i64::from(size.height);
                max_x = max_x.max(right);
                max_y = max_y.max(bottom);
            }
            let measured = Size {
                width: max_x.max(0) as u32,
                height: max_y.max(0) as u32,
            };
            resolve_size(absolute.layout.to_layout_box(), measured, measured)
        }
        Node::Stack(stack) => {
            let mut max_width = 0u32;
            let mut max_height = 0u32;
            for (index, child) in stack.children.iter().enumerate() {
                let child_id = engine.child_node_id(node_id, index).unwrap_or(node_id);
                let measured = measure_node_cached(child, child_id, tokens, token_hash, engine);
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
        Node::ScrollView(scroll_view) => {
            let child_id = engine.child_node_id(node_id, 0).unwrap_or(node_id);
            let content =
                measure_node_cached(scroll_view.content(), child_id, tokens, token_hash, engine);
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
        Node::Wrap(wrap) => {
            let mut total_width = 0u64;
            let mut max_height = 0u64;
            let mut child_count = 0u64;
            for (index, child) in wrap.children.iter().enumerate() {
                let child_id = engine.child_node_id(node_id, index).unwrap_or(node_id);
                let measured = measure_node_cached(child, child_id, tokens, token_hash, engine);
                total_width = total_width.saturating_add(u64::from(measured.width));
                max_height = max_height.max(u64::from(measured.height));
                child_count = child_count.saturating_add(1);
            }
            let gap_total =
                u64::from(wrap.column_gap.max(0) as u32).saturating_mul(child_count.saturating_sub(1));
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
        Node::SwitchLayout(switch_layout) => {
            let fallback_index = switch_layout.cases().len();
            let fallback_id = engine.child_node_id(node_id, fallback_index).unwrap_or(node_id);
            let mut measured = measure_node_cached(
                switch_layout.fallback(),
                fallback_id,
                tokens,
                token_hash,
                engine,
            );
            for (index, case_entry) in switch_layout.cases().iter().enumerate() {
                let case_id = engine.child_node_id(node_id, index).unwrap_or(node_id);
                let case_measured = measure_node_cached(
                    case_entry.child(),
                    case_id,
                    tokens,
                    token_hash,
                    engine,
                );
                measured.width = measured.width.max(case_measured.width);
                measured.height = measured.height.max(case_measured.height);
            }
            resolve_size(switch_layout.layout.to_layout_box(), measured, measured)
        }
        Node::Label(label) => measure_label(label, tokens),
        Node::Spacer(spacer) => spacer.size,
        Node::Knob(knob) => measure_knob(knob, tokens),
        Node::Slider(slider) => measure_slider(slider, tokens),
        Node::Toggle(toggle) => measure_toggle(toggle, tokens),
        Node::Button(button) => measure_button(button, tokens),
        Node::Dropdown(dropdown) => measure_dropdown(dropdown, tokens),
        Node::Region(region) => region.size,
        Node::Indicator(indicator) => indicator.size,
    })
}

/// Measure a flex container using cached child subtree measurements.
fn measure_flex_cached(
    flex: &FlexSpec,
    node_id: NodeId,
    tokens: &ThemeTokens,
    token_hash: u64,
    axis: Axis,
    engine: &mut LayoutEngineState,
) -> Size {
    let mut total_main = 0u64;
    let mut max_cross = 0u64;
    let mut child_count = 0u64;
    for (index, child) in flex.children.iter().enumerate() {
        let child_id = engine.child_node_id(node_id, index).unwrap_or(node_id);
        let child_size = measure_node_cached(child, child_id, tokens, token_hash, engine);
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

/// Measure a grid container using cached child subtree measurements.
fn measure_grid_cached(
    grid: &GridSpec,
    node_id: NodeId,
    tokens: &ThemeTokens,
    token_hash: u64,
    engine: &mut LayoutEngineState,
) -> Size {
    let column_count = grid.template.columns.len().max(1);
    let row_count = grid_row_count(grid, column_count);
    let mut column_widths = vec![0u32; column_count];
    let mut row_heights = vec![0u32; row_count];
    for (index, child) in grid.children.iter().enumerate() {
        let child_id = engine.child_node_id(node_id, index).unwrap_or(node_id);
        let size = measure_node_cached(child, child_id, tokens, token_hash, engine);
        let col = index % column_count;
        let row = index / column_count;
        column_widths[col] = column_widths[col].max(size.width);
        row_heights[row] = row_heights[row].max(size.height);
    }
    apply_px_track_mins(&grid.template.columns, &mut column_widths);
    apply_px_track_mins(&grid.template.rows, &mut row_heights);
    let (width, height) = measured_grid_extent(grid, &column_widths, &row_heights);
    resolve_size(
        grid.layout.to_layout_box(),
        Size { width, height },
        Size { width, height },
    )
}
