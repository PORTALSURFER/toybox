/// Render a width-based responsive switch container.
fn render_switch_layout(
    switch_layout: &SwitchLayoutSpec,
    rect: Rect,
    ui: &mut Ui<'_>,
    ctx: &mut RenderCtx<'_>,
) {
    let root_content_width = ctx.root_content_width.max(1);
    let child = switch_layout.selected_child(root_content_width);
    let is_fallback = std::ptr::eq(child, switch_layout.fallback());
    let measured = measure_node(child, ctx.tokens);
    let layout = node_layout(child);
    let resolved = clamp_size_to_available(resolve_size(layout, measured, rect.size), rect.size);
    let child_rect = Rect {
        origin: rect.origin,
        size: resolved,
    };
    let requested_rect = child_rect;
    let Some(child_rect) = overflow_rect_with_policy(
        child_rect,
        rect,
        switch_layout.overflow_policy(),
        ContainerKind::SwitchLayout,
        ctx.layout_diagnostics,
    ) else {
        return;
    };
    let mut reasons = vec![if is_fallback {
        LayoutNodeDiagnosticReason::FallbackSelected
    } else {
        LayoutNodeDiagnosticReason::SwitchCaseSelected
    }];
    if let Some(reason) = overflow_reason(requested_rect, child_rect, switch_layout.overflow_policy())
    {
        reasons.push(reason);
    }
    queue_next_node_reasons(ctx, reasons);
    ctx.depth += 1;
    render_node(child, child_rect, ui, ctx);
    ctx.depth = ctx.depth.saturating_sub(1);

    collect_container_debug_border_candidate(
        ctx.debug_border_candidates,
        ui,
        rect,
        ContainerKind::SwitchLayout,
        ctx.depth,
    );
}
