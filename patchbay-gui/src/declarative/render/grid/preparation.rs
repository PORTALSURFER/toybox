/// Fully prepared render inputs for a grid container.
struct PreparedGridRenderPass<'a> {
    /// Source grid specification.
    grid: &'a GridSpec,
    /// Resolved layout tracks and intrinsic measurements.
    layout: PreparedGridLayout,
    /// Horizontal spacing values derived from justification.
    spacing: GridColumnSpacing,
}

/// Prepared grid geometry and track sizing for a single render pass.
struct PreparedGridLayout {
    /// Total number of columns resolved from the template.
    columns: usize,
    /// Total number of rows needed for the current child count.
    rows: usize,
    /// Insets-applied content bounds used for cell placement.
    inner: Rect,
    /// One intrinsic measurement per child node.
    intrinsic: Vec<Size>,
    /// Resolved width for each column track.
    column_widths: Vec<u32>,
    /// Resolved height for each row track.
    row_heights: Vec<u32>,
    /// Final row gap value applied during placement.
    row_gap: i32,
}

/// Horizontal spacing details computed from grid justification.
struct GridColumnSpacing {
    /// Leading horizontal offset before the first column starts.
    leading_space: i32,
    /// Gap width between adjacent columns.
    column_gaps: Vec<i32>,
}

/// Build a render pass for a grid when children are present.
fn prepare_grid_render_pass<'a>(
    grid: &'a GridSpec,
    rect: Rect,
    tokens: &ThemeTokens,
) -> Option<PreparedGridRenderPass<'a>> {
    let layout = prepare_grid_layout(grid, rect, tokens)?;
    let spacing = compute_grid_column_spacing(grid, &layout);
    Some(PreparedGridRenderPass {
        grid,
        layout,
        spacing,
    })
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
    let (column_widths, row_heights) = resolve_grid_tracks(ResolveGridTracksRequest {
        grid,
        columns,
        rows,
        inner,
        row_tracks: &row_tracks,
        intrinsic: &intrinsic,
    });
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

/// Resolve column widths and row heights for the current grid bounds.
fn resolve_grid_tracks(request: ResolveGridTracksRequest<'_>) -> (Vec<u32>, Vec<u32>) {
    let column_widths = resolve_grid_axis(GridAxisResolveRequest {
        tracks: &request.grid.template.columns,
        columns: request.columns,
        rows: request.rows,
        gap: request.grid.template.column_gap.max(0),
        available: request.inner.size.width,
        is_columns: true,
        intrinsic: request.intrinsic,
    });
    let row_heights = resolve_grid_axis(GridAxisResolveRequest {
        tracks: request.row_tracks,
        columns: request.columns,
        rows: request.rows,
        gap: request.grid.template.row_gap.max(0),
        available: request.inner.size.height,
        is_columns: false,
        intrinsic: request.intrinsic,
    });
    (column_widths, row_heights)
}

/// Inputs required to resolve the full set of grid tracks for one layout pass.
struct ResolveGridTracksRequest<'a> {
    /// Source grid specification.
    grid: &'a GridSpec,
    /// Resolved column count.
    columns: usize,
    /// Resolved row count.
    rows: usize,
    /// Insets-applied inner render bounds.
    inner: Rect,
    /// Expanded row track definitions.
    row_tracks: &'a [TrackSize],
    /// Measured intrinsic child sizes.
    intrinsic: &'a [Size],
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
