/// Render a single-slot padding container.
fn render_padding_box(
    padding_box: &PaddingBoxSpec,
    rect: Rect,
    ui: &mut Ui<'_>,
    ctx: &mut RenderCtx<'_>,
) {
    let inner = inset_rect(rect, padding_box.padding);
    let child = padding_box.content();
    let measured = measure_node(child, ctx.tokens);
    let layout = node_layout(child);
    let resolved = clamp_size_to_available(
        resolve_size_with_diagnostics(
            layout,
            measured,
            inner.size,
            ContainerKind::PaddingBox,
            ctx.layout_diagnostics,
        ),
        inner.size,
    );
    let child_rect = Rect {
        origin: inner.origin,
        size: resolved,
    };
    let requested_rect = child_rect;
    let Some(child_rect) = overflow_rect_with_policy(
        child_rect,
        inner,
        padding_box.overflow_policy(),
        ContainerKind::PaddingBox,
        ctx.layout_diagnostics,
    ) else {
        return;
    };
    if let Some(reason) = overflow_reason(requested_rect, child_rect, padding_box.overflow_policy()) {
        queue_next_node_reason(ctx, reason);
    }
    ctx.depth += 1;
    render_node(child, child_rect, ui, ctx);
    ctx.depth = ctx.depth.saturating_sub(1);

    collect_container_debug_border_candidate(
        ctx.debug_border_candidates,
        ui,
        rect,
        ContainerKind::PaddingBox,
        ctx.depth,
    );
}
