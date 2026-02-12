/// Record the current grid bounds as a debug-border candidate.
fn emit_grid_debug_candidate(rect: Rect, ui: &Ui<'_>, ctx: &mut RenderCtx<'_>) {
    collect_container_debug_border_candidate(
        ctx.debug_border_candidates,
        ui,
        rect,
        ContainerKind::Grid,
        ctx.depth,
    );
}
