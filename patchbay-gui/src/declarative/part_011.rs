
/// Render a grid container.
fn render_grid(grid: &GridSpec, rect: Rect, ui: &mut Ui<'_>, ctx: &mut RenderCtx<'_>) {
    let columns = grid.template.columns.len().max(1);
    let rows = if grid.children.is_empty() {
        0
    } else {
        grid.children.len().div_ceil(columns)
    };
    if rows == 0 {
        return;
    }

    let inner = inset_rect(rect, grid.template.padding);
    let intrinsic: Vec<Size> = grid
        .children
        .iter()
        .map(|child| measure_node(child, ctx.tokens))
        .collect();

    let column_widths = resolve_grid_axis(
        &grid.template.columns,
        columns,
        rows,
        grid.template.column_gap.max(0),
        inner.size.width,
        true,
        &intrinsic,
    );
    let row_tracks = if grid.template.rows.is_empty() {
        vec![TrackSize::Auto; rows]
    } else {
        let mut tracks = grid.template.rows.clone();
        if tracks.len() < rows {
            tracks.resize(rows, TrackSize::Auto);
        }
        tracks
    };
    let row_heights = resolve_grid_axis(
        &row_tracks,
        columns,
        rows,
        grid.template.row_gap.max(0),
        inner.size.height,
        false,
        &intrinsic,
    );

    let column_gap = grid.template.column_gap.max(0);
    let row_gap = grid.template.row_gap.max(0);
    let packed_columns_width = column_widths.iter().copied().sum::<u32>()
        + (column_gap as u32).saturating_mul(columns.saturating_sub(1) as u32);
    let free_width = (inner.size.width as i32 - packed_columns_width as i32).max(0);
    let space_weights = justify_space_weights(grid.template.justify_x, columns);
    let extra_spaces = distribute_space(free_width, &space_weights);
    let mut column_gaps = vec![column_gap; columns.saturating_sub(1)];
    for (index, gap_value) in column_gaps.iter_mut().enumerate() {
        *gap_value += extra_spaces.get(index + 1).copied().unwrap_or(0);
    }
    let mut y = inner.origin.y;
    for (row, row_height) in row_heights.iter().copied().enumerate().take(rows) {
        let mut x = inner.origin.x + extra_spaces.first().copied().unwrap_or(0);
        for (col, col_width) in column_widths.iter().copied().enumerate().take(columns) {
            let index = row * columns + col;
            if let Some(child) = grid.children.get(index) {
                let cell_rect = Rect {
                    origin: Point { x, y },
                    size: Size {
                        width: col_width,
                        height: row_height,
                    },
                };
                let layout = node_layout(child);
                let measured = intrinsic[index];
                let resolved = clamp_size_to_available(
                    resolve_size(layout, measured, cell_rect.size),
                    cell_rect.size,
                );
                ctx.depth += 1;
                render_node(
                    child,
                    Rect {
                        origin: cell_rect.origin,
                        size: resolved,
                    },
                    ui,
                    ctx,
                );
                ctx.depth = ctx.depth.saturating_sub(1);
            }
            let next_gap = column_gaps.get(col).copied().unwrap_or(0);
            x += col_width as i32 + next_gap;
        }
        y += row_height as i32 + row_gap;
    }

    collect_container_debug_border_candidate(
        ctx.debug_border_candidates,
        ui,
        rect,
        ContainerKind::Grid,
        ctx.depth,
    );
}

/// Clamp a resolved child size so it cannot exceed the available slot size.
fn clamp_size_to_available(resolved: Size, available: Size) -> Size {
    Size {
        width: resolved.width.min(available.width),
        height: resolved.height.min(available.height),
    }
}

/// Resolve one grid axis using track definitions and available space.
fn resolve_grid_axis(
    tracks: &[TrackSize],
    columns: usize,
    rows: usize,
    gap: i32,
    available: u32,
    is_columns: bool,
    intrinsic: &[Size],
) -> Vec<u32> {
    let count = if is_columns { columns } else { rows };
    let mut result = vec![0u32; count];

    for (index, value) in result.iter_mut().enumerate().take(count) {
        if let Some(track) = tracks.get(index).copied()
            && let TrackSize::Px(px) = track
        {
            *value = px;
        }
    }

    for (item, measured) in intrinsic.iter().enumerate() {
        let row = item / columns;
        let col = item % columns;
        let axis_index = if is_columns { col } else { row };
        let track = tracks.get(axis_index).copied().unwrap_or(TrackSize::Auto);
        if matches!(track, TrackSize::Auto) {
            let value = if is_columns {
                measured.width
            } else {
                measured.height
            };
            result[axis_index] = result[axis_index].max(value);
        }
    }

    let percent_weights: Vec<u8> = (0..count)
        .map(
            |index| match tracks.get(index).copied().unwrap_or(TrackSize::Auto) {
                TrackSize::Percent(percent) => percent,
                _ => 0,
            },
        )
        .collect();
    let total_percent: u16 = percent_weights.iter().map(|percent| *percent as u16).sum();
    if total_percent > 0 {
        let percent_target_total = available
            .saturating_mul(total_percent as u32)
            .saturating_div(100);
        let percent_distribution = distribute_weighted_u32(
            percent_target_total,
            &percent_weights
                .iter()
                .map(|percent| u32::from(*percent))
                .collect::<Vec<u32>>(),
        );
        for (index, assigned) in percent_distribution.into_iter().enumerate() {
            if percent_weights[index] > 0 {
                result[index] = assigned;
            }
        }
    }

    let total_gap = gap.max(0) as u32 * count.saturating_sub(1) as u32;
    let used = result.iter().copied().sum::<u32>() + total_gap;
    let remainder = available.saturating_sub(used);

    let fill_count = (0..count)
        .filter(|index| {
            matches!(
                tracks.get(*index).copied().unwrap_or(TrackSize::Auto),
                TrackSize::Fill
            )
        })
        .count();
    if fill_count > 0 {
        let fill_weights = vec![1u32; fill_count];
        let fill_lengths = distribute_weighted_u32(remainder, &fill_weights);
        let mut fill_cursor = 0usize;
        for (index, value) in result.iter_mut().enumerate() {
            if matches!(
                tracks.get(index).copied().unwrap_or(TrackSize::Auto),
                TrackSize::Fill
            ) {
                *value += fill_lengths[fill_cursor];
                fill_cursor += 1;
            }
        }
    } else {
        let fr_weights: Vec<u32> = (0..count)
            .map(|index| {
                tracks
                    .get(index)
                    .copied()
                    .unwrap_or(TrackSize::Auto)
                    .fr_weight()
            })
            .collect();
        let fr_lengths = distribute_weighted_u32(remainder, &fr_weights);
        for (value, added) in result.iter_mut().zip(fr_lengths.into_iter()) {
            *value += added;
        }
    }

    result
}

/// Distribute integer lengths across weighted slots using largest remainder.
///
/// The returned vector length matches `weights.len()`, and the sum of all
/// returned values equals `total` when at least one weight is non-zero.
fn distribute_weighted_u32(total: u32, weights: &[u32]) -> Vec<u32> {
    if weights.is_empty() {
        return Vec::new();
    }
    if total == 0 {
        return vec![0; weights.len()];
    }

    let weight_sum: u64 = weights.iter().map(|weight| u64::from(*weight)).sum();
    if weight_sum == 0 {
        return vec![0; weights.len()];
    }

    let total_u64 = u64::from(total);
    let mut distributed = vec![0u32; weights.len()];
    let mut used = 0u64;
    let mut remainder_order = Vec::new();

    for (index, weight) in weights.iter().copied().enumerate() {
        if weight == 0 {
            continue;
        }
        let numerator = total_u64 * u64::from(weight);
        let base = numerator / weight_sum;
        distributed[index] = base as u32;
        used += base;
        remainder_order.push((index, numerator % weight_sum));
    }

    let leftover = total_u64.saturating_sub(used) as usize;
    if leftover > 0 && !remainder_order.is_empty() {
        remainder_order
            .sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
        for offset in 0..leftover {
            let index = remainder_order[offset % remainder_order.len()].0;
            distributed[index] = distributed[index].saturating_add(1);
        }
    }

    distributed
}

/// Render an absolute-positioned container.
fn render_absolute(absolute: &AbsoluteSpec, rect: Rect, ui: &mut Ui<'_>, ctx: &mut RenderCtx<'_>) {
    for child in &absolute.children {
        let measured = measure_node(&child.node, ctx.tokens);
        let layout = node_layout(&child.node);
        let resolved = resolve_size(layout, measured, measured);
        let child_rect = Rect {
            origin: Point {
                x: rect.origin.x + child.origin.x,
                y: rect.origin.y + child.origin.y,
            },
            size: resolved,
        };
        ctx.depth += 1;
        render_node(&child.node, child_rect, ui, ctx);
        ctx.depth = ctx.depth.saturating_sub(1);
    }

    collect_container_debug_border_candidate(
        ctx.debug_border_candidates,
        ui,
        rect,
        ContainerKind::Absolute,
        ctx.depth,
    );
}

/// Render a label node.
fn render_label(label: &LabelSpec, rect: Rect, ui: &mut Ui<'_>, tokens: &ThemeTokens) {
    let color = label.color.unwrap_or(tokens.colors.text);
    let _ = ui.text_single_line_hard_clamped_in_rect_scaled(
        rect,
        &label.text,
        color,
        tokens.typography.text_scale,
    );
}
