/// Shared immutable inputs used while rendering grid rows.
struct GridRowRenderContext<'a> {
    /// Source grid specification.
    grid: &'a GridSpec,
    /// Prepared geometry reused by every row.
    layout: &'a PreparedGridLayout,
    /// Column spacing and leading offset derived from justification.
    spacing: &'a GridColumnSpacing,
}

/// Row-local placement parameters for one render pass.
struct GridRowGeometry {
    /// Zero-based row index.
    row: usize,
    /// Row top edge in surface coordinates.
    y: i32,
    /// Row track height.
    height: u32,
}

/// Render a previously prepared grid pass.
fn render_prepared_grid(
    pass: &PreparedGridRenderPass<'_>,
    ui: &mut Ui<'_>,
    ctx: &mut RenderCtx<'_>,
) {
    render_grid_children(pass.grid, &pass.layout, &pass.spacing, ui, ctx);
}

/// Render all children into the prepared grid cells.
fn render_grid_children(
    grid: &GridSpec,
    layout: &PreparedGridLayout,
    spacing: &GridColumnSpacing,
    ui: &mut Ui<'_>,
    ctx: &mut RenderCtx<'_>,
) {
    let row_ctx = GridRowRenderContext {
        grid,
        layout,
        spacing,
    };
    let mut y = layout.inner.origin.y;
    for (row, height) in layout.row_heights.iter().copied().enumerate().take(layout.rows) {
        render_grid_row(GridRowGeometry { row, y, height }, &row_ctx, ui, ctx);
        y += height as i32 + layout.row_gap;
    }
}

/// Render a single grid row.
fn render_grid_row(
    row_geometry: GridRowGeometry,
    row_ctx: &GridRowRenderContext<'_>,
    ui: &mut Ui<'_>,
    ctx: &mut RenderCtx<'_>,
) {
    let mut x = row_ctx.layout.inner.origin.x + row_ctx.spacing.leading_space;
    for (col, width) in row_ctx
        .layout
        .column_widths
        .iter()
        .copied()
        .enumerate()
        .take(row_ctx.layout.columns)
    {
        let index = row_geometry.row * row_ctx.layout.columns + col;
        if let Some(child) = row_ctx.grid.children.get(index) {
            let cell_rect = Rect {
                origin: Point {
                    x,
                    y: row_geometry.y,
                },
                size: Size {
                    width,
                    height: row_geometry.height,
                },
            };
            render_grid_child(
                child,
                row_ctx.layout.inner,
                cell_rect,
                row_ctx.layout.intrinsic[index],
                row_ctx.grid.overflow_policy(),
                ui,
                ctx,
            );
        }
        x += width as i32 + row_ctx.spacing.column_gaps.get(col).copied().unwrap_or(0);
    }
}

/// Render one child node inside a resolved grid cell.
fn render_grid_child(
    child: &Node,
    container_bounds: Rect,
    cell_rect: Rect,
    measured: Size,
    overflow_policy: OverflowPolicy,
    ui: &mut Ui<'_>,
    ctx: &mut RenderCtx<'_>,
) {
    let layout = node_layout(child);
    let resolved = clamp_size_to_available(resolve_size(layout, measured, cell_rect.size), cell_rect.size);
    let child_rect = Rect {
        origin: cell_rect.origin,
        size: resolved,
    };
    let requested_rect = child_rect;
    let Some(clipped_rect) = overflow_rect_with_policy(
        child_rect,
        container_bounds,
        overflow_policy,
        ContainerKind::Grid,
        ctx.layout_diagnostics,
    ) else {
        return;
    };
    if let Some(reason) = overflow_reason(requested_rect, clipped_rect, overflow_policy) {
        queue_next_node_reason(ctx, reason);
    }
    ctx.depth += 1;
    render_node(child, clipped_rect, ui, ctx);
    ctx.depth = ctx.depth.saturating_sub(1);
}

/// Clamp a resolved child size so it cannot exceed the available slot size.
fn clamp_size_to_available(resolved: Size, available: Size) -> Size {
    Size {
        width: resolved.width.min(available.width),
        height: resolved.height.min(available.height),
    }
}
