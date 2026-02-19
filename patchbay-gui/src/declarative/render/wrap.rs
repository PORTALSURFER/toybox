/// Render a wrap container.
fn render_wrap(wrap: &WrapSpec, rect: Rect, ui: &mut Ui<'_>, ctx: &mut RenderCtx<'_>) {
    let inner = inset_rect(rect, wrap.padding);
    if inner.size.width == 0 || inner.size.height == 0 || wrap.children.is_empty() {
        return;
    }

    let column_gap = wrap.column_gap.max(0);
    let row_gap = wrap.row_gap.max(0);
    let inner_width = i32::try_from(inner.size.width).unwrap_or(i32::MAX);
    let available = Size {
        width: inner.size.width,
        height: inner.size.height,
    };
    let measured: Vec<Size> = wrap
        .children
        .iter()
        .map(|child| {
            let layout = node_layout(child);
            clamp_size_to_available(resolve_size(layout, measure_node(child, ctx.tokens), available), available)
        })
        .collect();

    let rows = build_wrap_rows(&measured, inner_width, column_gap);
    let mut y = inner.origin.y;
    for row in rows {
        let row_height = row.height;
        let weights = justify_space_weights(wrap.justify, row.items.len());
        let extra_spaces = distribute_space((inner_width - row.width).max(0), &weights);
        let mut x = inner.origin.x + extra_spaces.first().copied().unwrap_or(0);
        let mut gaps = vec![column_gap; row.items.len().saturating_sub(1)];
        for (index, gap) in gaps.iter_mut().enumerate() {
            *gap += extra_spaces.get(index + 1).copied().unwrap_or(0);
        }
        for (item_offset, item_index) in row.items.iter().copied().enumerate() {
            let child_rect = Rect {
                origin: Point { x, y },
                size: measured[item_index],
            };
            let requested_rect = child_rect;
            let Some(child_rect) = overflow_rect_with_policy(
                child_rect,
                inner,
                wrap.overflow_policy(),
                ContainerKind::Wrap,
                ctx.layout_diagnostics,
            ) else {
                x = x.saturating_add(measured[item_index].width as i32);
                x = x.saturating_add(gaps.get(item_offset).copied().unwrap_or(0));
                continue;
            };
            if let Some(reason) =
                overflow_reason(requested_rect, child_rect, wrap.overflow_policy())
            {
                queue_next_node_reason(ctx, reason);
            }
            ctx.depth += 1;
            render_node(&wrap.children[item_index], child_rect, ui, ctx);
            ctx.depth = ctx.depth.saturating_sub(1);
            x = x.saturating_add(measured[item_index].width as i32);
            x = x.saturating_add(gaps.get(item_offset).copied().unwrap_or(0));
        }
        y = y.saturating_add(row_height).saturating_add(row_gap);
    }

    collect_container_debug_border_candidate(
        ctx.debug_border_candidates,
        ui,
        rect,
        ContainerKind::Wrap,
        ctx.depth,
    );
}

/// Row placement result for one wrap line.
struct WrapRow {
    /// Child indices contained in this row.
    items: Vec<usize>,
    /// Total width consumed by this row (without outer padding).
    width: i32,
    /// Maximum row height.
    height: i32,
}

/// Build deterministic wrap rows from measured child sizes.
fn build_wrap_rows(measured: &[Size], max_width: i32, column_gap: i32) -> Vec<WrapRow> {
    let mut rows = Vec::new();
    let mut current = WrapRow {
        items: Vec::new(),
        width: 0,
        height: 0,
    };

    for (index, size) in measured.iter().copied().enumerate() {
        let item_w = i32::try_from(size.width).unwrap_or(i32::MAX).min(max_width.max(0));
        let item_h = i32::try_from(size.height).unwrap_or(i32::MAX);
        let gap = if current.items.is_empty() { 0 } else { column_gap };
        let next_width = current.width.saturating_add(gap).saturating_add(item_w);
        if !current.items.is_empty() && next_width > max_width {
            rows.push(current);
            current = WrapRow {
                items: vec![index],
                width: item_w,
                height: item_h,
            };
            continue;
        }
        current.items.push(index);
        current.width = next_width;
        current.height = current.height.max(item_h);
    }

    if !current.items.is_empty() {
        rows.push(current);
    }
    rows
}
