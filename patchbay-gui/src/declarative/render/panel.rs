/// Render a panel container.
fn render_panel(panel: &PanelSpec, rect: Rect, ui: &mut Ui<'_>, ctx: &mut RenderCtx<'_>) {
    let title = panel.title.as_deref();
    let header_height = panel
        .header_height
        .unwrap_or_else(|| panel_header_height(title, ctx.tokens));
    let style = crate::ui::PanelStyle {
        title,
        padding: panel.padding,
        background: Some(panel.background.unwrap_or(ctx.tokens.colors.surface)),
        outline: Some(panel.outline.unwrap_or(ctx.tokens.colors.border)),
        header_height: Some(header_height),
    };

    let mut outer_rect = rect;
    ui.with_layout(rect.origin, |ui| {
        let response = ui.panel_with_key(&panel.key, style, Some(rect.size), |ui, content_rect| {
            ctx.depth += 1;
            render_node(&panel.content, content_rect, ui, ctx);
            ctx.depth = ctx.depth.saturating_sub(1);
        });
        outer_rect = response.outer_rect;
    });
    collect_container_debug_border_candidate(
        ctx.debug_border_candidates,
        ui,
        outer_rect,
        ContainerKind::Panel,
        ctx.depth,
    );
}
