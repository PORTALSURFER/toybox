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
