/// Render an absolute-positioned container.
fn render_absolute(absolute: &AbsoluteSpec, rect: Rect, ui: &mut Ui<'_>, ctx: &mut RenderCtx<'_>) {
    let overflow_policy = absolute.overflow_policy();
    for child in &absolute.children {
        let measured = measure_node(&child.node, ctx.tokens);
        let layout = node_layout(&child.node);
        let resolved = resolve_size_with_diagnostics(
            layout,
            measured,
            measured,
            ContainerKind::Absolute,
            ctx.layout_diagnostics,
        );
        let child_rect = Rect {
            origin: Point {
                x: rect.origin.x + child.origin.x,
                y: rect.origin.y + child.origin.y,
            },
            size: resolved,
        };
        let requested_rect = child_rect;
        let Some(child_rect) = overflow_rect_with_policy(
            child_rect,
            rect,
            overflow_policy,
            ContainerKind::Absolute,
            ctx.layout_diagnostics,
        ) else {
            continue;
        };
        if let Some(reason) = overflow_reason(requested_rect, child_rect, overflow_policy) {
            queue_next_node_reason(ctx, reason);
        }
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
