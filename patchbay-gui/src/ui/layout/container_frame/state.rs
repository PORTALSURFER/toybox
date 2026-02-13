/// Persist measured size and apply container-specific post effects.
fn persist_container_frame_state(
    ui: &mut Ui<'_>,
    resolved: &ContainerFrameResolved<'_>,
    measured_size: Size,
    effects: ContainerFrameEffects,
) {
    ui.state.layout.set(resolved.id, measured_size);
    ui.track_rect_internal(resolved.outer_rect);
    if effects.advance_layout_cursor {
        ui.layout.cursor.y = resolved.origin.y + measured_size.height as i32 + ui.layout.spacing;
    }
    if effects.update_root_frame_size {
        ui.state.set_root_frame_size(measured_size);
    }
}
