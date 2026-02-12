/// Render a grid container.
fn render_grid(grid: &GridSpec, rect: Rect, ui: &mut Ui<'_>, ctx: &mut RenderCtx<'_>) {
    let Some(pass) = prepare_grid_render_pass(grid, rect, ctx.tokens) else {
        return;
    };
    render_prepared_grid(&pass, ui, ctx);
    emit_grid_debug_candidate(rect, ui, ctx);
}
