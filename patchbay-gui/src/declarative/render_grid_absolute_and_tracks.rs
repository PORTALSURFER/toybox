
/// Render a grid container.
fn render_grid(grid: &GridSpec, rect: Rect, ui: &mut Ui<'_>, ctx: &mut RenderCtx<'_>) {
    let Some(layout) = prepare_grid_layout(grid, rect, ctx.tokens) else {
        return;
    };
    let spacing = compute_grid_column_spacing(grid, &layout);
    render_grid_children(grid, &layout, &spacing, ui, ctx);

    collect_container_debug_border_candidate(
        ctx.debug_border_candidates,
        ui,
        rect,
        ContainerKind::Grid,
        ctx.depth,
    );
}

/// Prepared grid geometry and track sizing for a single render pass.
struct PreparedGridLayout {
    columns: usize,
    rows: usize,
    inner: Rect,
    intrinsic: Vec<Size>,
    column_widths: Vec<u32>,
    row_heights: Vec<u32>,
    row_gap: i32,
}

/// Horizontal spacing details computed from grid justification.
struct GridColumnSpacing {
    leading_space: i32,
    column_gaps: Vec<i32>,
}

/// Compute grid tracks and intrinsic measurements before rendering children.
fn prepare_grid_layout(grid: &GridSpec, rect: Rect, tokens: &ThemeTokens) -> Option<PreparedGridLayout> {
    let columns = grid.template.columns.len().max(1);
    let rows = grid_rows(grid, columns);
    if rows == 0 {
        return None;
    }

    let inner = inset_rect(rect, grid.template.padding);
    let intrinsic = measure_grid_intrinsic_sizes(grid, tokens);
    let row_tracks = expanded_row_tracks(grid, rows);
    let column_widths = resolve_grid_axis(
        &grid.template.columns,
        columns,
        rows,
        grid.template.column_gap.max(0),
        inner.size.width,
        true,
        &intrinsic,
    );
    let row_heights = resolve_grid_axis(
        &row_tracks,
        columns,
        rows,
        grid.template.row_gap.max(0),
        inner.size.height,
        false,
        &intrinsic,
    );
    Some(PreparedGridLayout { columns, rows, inner, intrinsic, column_widths, row_heights, row_gap: grid.template.row_gap.max(0) })
}

/// Count row tracks needed to fit all children in the configured column count.
fn grid_rows(grid: &GridSpec, columns: usize) -> usize {
    if grid.children.is_empty() {
        0
    } else {
        grid.children.len().div_ceil(columns)
    }
}

/// Measure each child once so track sizing and rendering can reuse the results.
fn measure_grid_intrinsic_sizes(grid: &GridSpec, tokens: &ThemeTokens) -> Vec<Size> {
    grid.children
        .iter()
        .map(|child| measure_node(child, tokens))
        .collect()
}

/// Expand row tracks to the actual row count, defaulting missing entries to auto.
fn expanded_row_tracks(grid: &GridSpec, rows: usize) -> Vec<TrackSize> {
    if grid.template.rows.is_empty() {
        return vec![TrackSize::Auto; rows];
    }
    let mut tracks = grid.template.rows.clone();
    if tracks.len() < rows {
        tracks.resize(rows, TrackSize::Auto);
    }
    tracks
}

/// Build horizontal spacing from resolved tracks and justification strategy.
fn compute_grid_column_spacing(grid: &GridSpec, layout: &PreparedGridLayout) -> GridColumnSpacing {
    let column_gap = grid.template.column_gap.max(0);
    let packed_width = layout.column_widths.iter().copied().sum::<u32>()
        + (column_gap as u32).saturating_mul(layout.columns.saturating_sub(1) as u32);
    let free_width = (layout.inner.size.width as i32 - packed_width as i32).max(0);
    let weights = justify_space_weights(grid.template.justify_x, layout.columns);
    let extra_spaces = distribute_space(free_width, &weights);
    let mut column_gaps = vec![column_gap; layout.columns.saturating_sub(1)];
    for (index, gap_value) in column_gaps.iter_mut().enumerate() {
        *gap_value += extra_spaces.get(index + 1).copied().unwrap_or(0);
    }
    GridColumnSpacing {
        leading_space: extra_spaces.first().copied().unwrap_or(0),
        column_gaps,
    }
}

/// Render all children into the prepared grid cells.
fn render_grid_children(
    grid: &GridSpec,
    layout: &PreparedGridLayout,
    spacing: &GridColumnSpacing,
    ui: &mut Ui<'_>,
    ctx: &mut RenderCtx<'_>,
) {
    let mut y = layout.inner.origin.y;
    for (row, row_height) in layout.row_heights.iter().copied().enumerate().take(layout.rows) {
        render_grid_row(row, row_height, y, grid, layout, spacing, ui, ctx);
        y += row_height as i32 + layout.row_gap;
    }
}

/// Render a single grid row.
fn render_grid_row(
    row: usize,
    row_height: u32,
    y: i32,
    grid: &GridSpec,
    layout: &PreparedGridLayout,
    spacing: &GridColumnSpacing,
    ui: &mut Ui<'_>,
    ctx: &mut RenderCtx<'_>,
) {
    let mut x = layout.inner.origin.x + spacing.leading_space;
    for (col, col_width) in layout.column_widths.iter().copied().enumerate().take(layout.columns) {
        let index = row * layout.columns + col;
        if let Some(child) = grid.children.get(index) {
            let cell_rect = Rect {
                origin: Point { x, y },
                size: Size {
                    width: col_width,
                    height: row_height,
                },
            };
            render_grid_child(child, cell_rect, layout.intrinsic[index], ui, ctx);
        }
        x += col_width as i32 + spacing.column_gaps.get(col).copied().unwrap_or(0);
    }
}

/// Render one child node inside a resolved grid cell.
fn render_grid_child(
    child: &Node,
    cell_rect: Rect,
    measured: Size,
    ui: &mut Ui<'_>,
    ctx: &mut RenderCtx<'_>,
) {
    let layout = node_layout(child);
    let resolved = clamp_size_to_available(resolve_size(layout, measured, cell_rect.size), cell_rect.size);
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
    let axis = if is_columns {
        GridAxis::Columns
    } else {
        GridAxis::Rows
    };
    let axis_count = if is_columns { columns } else { rows };
    resolve_grid_axis_plan(&GridAxisPlan {
        tracks,
        axis_count,
        columns,
        gap,
        available,
        axis,
        intrinsic,
    })
}

/// Resolve one grid axis from an already-built axis plan.
fn resolve_grid_axis_plan(plan: &GridAxisPlan<'_>) -> Vec<u32> {
    let mut result = seed_pixel_tracks(plan);
    apply_auto_tracks(plan, &mut result);
    assign_percent_tracks(plan, &mut result);
    distribute_remaining_tracks(plan, &mut result);
    result
}

/// Axis selection for grid resolution.
#[derive(Copy, Clone)]
enum GridAxis {
    Columns,
    Rows,
}

impl GridAxis {
    /// Return the axis index for a flattened grid child index.
    fn index_for_item(self, item: usize, columns: usize) -> usize {
        match self {
            Self::Columns => item % columns,
            Self::Rows => item / columns,
        }
    }

    /// Read the intrinsic size component matching this axis.
    fn intrinsic_size(self, measured: Size) -> u32 {
        match self {
            Self::Columns => measured.width,
            Self::Rows => measured.height,
        }
    }
}

/// Parameters required to resolve one grid axis.
struct GridAxisPlan<'a> {
    tracks: &'a [TrackSize],
    axis_count: usize,
    columns: usize,
    gap: i32,
    available: u32,
    axis: GridAxis,
    intrinsic: &'a [Size],
}

/// Resolve track definition at an index, defaulting to `Auto`.
fn axis_track(plan: &GridAxisPlan<'_>, index: usize) -> TrackSize {
    plan.tracks.get(index).copied().unwrap_or(TrackSize::Auto)
}

/// Seed the result with fixed pixel tracks.
fn seed_pixel_tracks(plan: &GridAxisPlan<'_>) -> Vec<u32> {
    let mut result = vec![0u32; plan.axis_count];
    for (index, value) in result.iter_mut().enumerate().take(plan.axis_count) {
        if let TrackSize::Px(px) = axis_track(plan, index) {
            *value = px;
        }
    }
    result
}

/// Size auto tracks from intrinsic child measurements.
fn apply_auto_tracks(plan: &GridAxisPlan<'_>, result: &mut [u32]) {
    for (item, measured) in plan.intrinsic.iter().enumerate() {
        let index = plan.axis.index_for_item(item, plan.columns);
        if matches!(axis_track(plan, index), TrackSize::Auto) {
            result[index] = result[index].max(plan.axis.intrinsic_size(*measured));
        }
    }
}

/// Assign percent tracks as fixed percentages of available axis space.
fn assign_percent_tracks(plan: &GridAxisPlan<'_>, result: &mut [u32]) {
    let percent_weights: Vec<u8> = (0..plan.axis_count)
        .map(|index| match axis_track(plan, index) {
            TrackSize::Percent(percent) => percent,
            _ => 0,
        })
        .collect();
    let total_percent: u16 = percent_weights.iter().map(|percent| *percent as u16).sum();
    if total_percent == 0 {
        return;
    }
    let target_total = plan.available.saturating_mul(total_percent as u32).saturating_div(100);
    let weights: Vec<u32> = percent_weights.iter().map(|percent| u32::from(*percent)).collect();
    let assigned = distribute_weighted_u32(target_total, &weights);
    for (index, value) in assigned.into_iter().enumerate() {
        if percent_weights[index] > 0 {
            result[index] = value;
        }
    }
}

/// Distribute remaining space to fill tracks, otherwise FR tracks.
fn distribute_remaining_tracks(plan: &GridAxisPlan<'_>, result: &mut [u32]) {
    let remainder = remaining_axis_space(plan, result);
    let fill_indices: Vec<usize> = (0..plan.axis_count)
        .filter(|index| matches!(axis_track(plan, *index), TrackSize::Fill))
        .collect();
    if !fill_indices.is_empty() {
        let fill = distribute_weighted_u32(remainder, &vec![1u32; fill_indices.len()]);
        for (offset, index) in fill_indices.into_iter().enumerate() {
            result[index] += fill[offset];
        }
        return;
    }
    let weights: Vec<u32> = (0..plan.axis_count)
        .map(|index| axis_track(plan, index).fr_weight())
        .collect();
    let fr = distribute_weighted_u32(remainder, &weights);
    for (value, added) in result.iter_mut().zip(fr.into_iter()) {
        *value += added;
    }
}

/// Remaining space after fixed, auto and percent track resolution.
fn remaining_axis_space(plan: &GridAxisPlan<'_>, result: &[u32]) -> u32 {
    let total_gap = plan.gap.max(0) as u32 * plan.axis_count.saturating_sub(1) as u32;
    let used = result.iter().copied().sum::<u32>() + total_gap;
    plan.available.saturating_sub(used)
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
