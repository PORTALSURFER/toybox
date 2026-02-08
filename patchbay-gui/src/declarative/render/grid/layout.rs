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
    Some(PreparedGridLayout {
        columns,
        rows,
        inner,
        intrinsic,
        column_widths,
        row_heights,
        row_gap: grid.template.row_gap.max(0),
    })
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
