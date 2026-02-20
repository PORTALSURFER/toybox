/// Measure a grid container intrinsically.
fn measure_grid(grid: &GridSpec, tokens: &ThemeTokens) -> Size {
    let column_count = grid.template.columns.len().max(1);
    let row_count = grid_row_count(grid, column_count);
    let mut column_widths = vec![0u32; column_count];
    let mut row_heights = vec![0u32; row_count];
    measure_grid_children_into_tracks(
        grid,
        tokens,
        column_count,
        &mut column_widths,
        &mut row_heights,
    );
    apply_px_track_mins(&grid.template.columns, &mut column_widths);
    apply_px_track_mins(&grid.template.rows, &mut row_heights);
    let (width, height) = measured_grid_extent(grid, &column_widths, &row_heights);
    resolve_size(
        grid.layout.to_layout_box(),
        Size { width, height },
        Size { width, height },
    )
}

/// Return the number of grid rows required for all children.
fn grid_row_count(grid: &GridSpec, column_count: usize) -> usize {
    if grid.children.is_empty() {
        0
    } else {
        grid.children.len().div_ceil(column_count)
    }
}

/// Measure children and update per-column and per-row intrinsic track maxima.
fn measure_grid_children_into_tracks(
    grid: &GridSpec,
    tokens: &ThemeTokens,
    column_count: usize,
    column_widths: &mut [u32],
    row_heights: &mut [u32],
) {
    for (index, child) in grid.children.iter().enumerate() {
        let size = measure_node(child, tokens);
        let col = index % column_count;
        let row = index / column_count;
        column_widths[col] = column_widths[col].max(size.width);
        row_heights[row] = row_heights[row].max(size.height);
    }
}

/// Apply fixed pixel track minimums to measured track vectors.
fn apply_px_track_mins(tracks: &[TrackSize], measured: &mut [u32]) {
    for (index, track) in tracks.iter().copied().enumerate() {
        if let Some(value) = measured.get_mut(index)
            && let TrackSize::Px(px) = track
        {
            *value = (*value).max(px);
        }
    }
}

/// Compute final measured width/height from tracks and padding.
fn measured_grid_extent(grid: &GridSpec, column_widths: &[u32], row_heights: &[u32]) -> (u32, u32) {
    let width = column_widths.iter().copied().sum::<u32>()
        + grid.template.padding.left.max(0) as u32
        + grid.template.padding.right.max(0) as u32;
    let height = row_heights.iter().copied().sum::<u32>()
        + grid.template.padding.top.max(0) as u32
        + grid.template.padding.bottom.max(0) as u32;
    (width, height)
}
