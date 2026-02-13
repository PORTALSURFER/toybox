/// Run the full root/panel frame lifecycle and return measured metadata.
fn render_container_frame<F>(
    ui: &mut Ui<'_>,
    spec: ContainerFrameSpec<'_>,
    explicit_size_policy: ExplicitSizePolicy,
    effects: ContainerFrameEffects,
    mut f: F,
) -> ContainerFrameResult
where
    F: FnMut(&mut Ui<'_>, Rect),
{
    let id = WidgetId::from_label(spec.key);
    let resolved = resolve_container_frame(spec, id, ui.state.layout.get(id), ui.theme.text_scale);
    draw_container_frame_shell(ui, resolved);
    let measured_bounds = measure_container_frame_content(ui, &resolved, &mut f);
    let measured_size = finalize_container_frame_size(&resolved, measured_bounds, explicit_size_policy);
    persist_container_frame_state(ui, &resolved, measured_size, effects);
    ContainerFrameResult {
        outer_rect: resolved.outer_rect,
        content_rect: resolved.content_rect,
        measured_size,
    }
}

/// Draw the frame shell and optional header title.
fn draw_container_frame_shell(ui: &mut Ui<'_>, resolved: ContainerFrameResolved<'_>) {
    ui.fill_rect_clipped(resolved.outer_rect, resolved.background);
    ui.stroke_rect_clipped(resolved.outer_rect, 1, resolved.outline);
    draw_container_frame_title(ui, resolved);
}

/// Draw frame title text and track its footprint when present.
fn draw_container_frame_title(ui: &mut Ui<'_>, resolved: ContainerFrameResolved<'_>) {
    let Some(title) = resolved.title else {
        return;
    };
    let title_pos = Point {
        x: resolved.origin.x + resolved.padding,
        y: resolved.origin.y + resolved.padding,
    };
    ui.draw_text_internal(title_pos, title, ui.theme.text, ui.theme.text_scale);
    ui.track_rect_internal(Rect {
        origin: title_pos,
        size: text_size(title, ui.theme.text_scale),
    });
}
