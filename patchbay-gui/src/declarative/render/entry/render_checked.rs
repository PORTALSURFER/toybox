/// Render a UI specification and collect typed actions.
///
/// # Errors
/// Returns [`DeclarativeError`] when validation fails.
pub fn render_checked(
    spec: &UiSpec,
    ui: &mut Ui<'_>,
    origin: Point,
) -> Result<RenderResult, DeclarativeError> {
    validate_spec(spec)?;
    let tokens = spec.root.tokens.unwrap_or_default();
    let plan = plan_root_render(spec, ui.input().window_size);
    let resolved = plan.layout_size;

    let style = RootFrameStyle {
        title: spec.root.title.as_deref(),
        padding: spec.root.padding,
        background: Some(tokens.colors.surface),
        outline: Some(tokens.colors.border),
        header_height: Some(panel_header_height(spec.root.title.as_deref(), &tokens)),
    };

    let mut actions = Vec::new();
    let mut debug_border_candidates = Vec::new();
    let response = {
        let mut ctx = RenderCtx {
            tokens: &tokens,
            actions: &mut actions,
            debug_border_candidates: &mut debug_border_candidates,
            depth: 1,
        };
        let response =
            ui.root_frame_with_key_at(&spec.root.key, style, Some(resolved), origin, |ui, rect| {
                render_node(&spec.root.content, rect, ui, &mut ctx);
            });
        collect_container_debug_border_candidate(
            ctx.debug_border_candidates,
            ui,
            response.outer_rect,
            ContainerKind::RootFrame,
            0,
        );
        response
    };
    if let Some(candidate) = select_container_debug_border_candidate(&debug_border_candidates)
        && let Some(color) = container_debug_border_color(candidate.kind, candidate.depth)
        && let Some(draw_rect) = debug_border_draw_rect(candidate.rect, 1)
    {
        ui.debug_stroke_rect(draw_rect, 1, color);
    }

    Ok(RenderResult {
        measured_size: resolved,
        actions,
        resolved_scale: plan.resolved_scale,
        content_rect: response.content_rect,
    })
}
