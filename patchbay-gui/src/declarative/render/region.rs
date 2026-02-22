/// Render a region node and emit interaction actions.
fn render_region(region: &RegionSpec, rect: Rect, ui: &mut Ui<'_>, actions: &mut Vec<UiAction>) {
    render_region_draw_commands(&region.draw, rect, ui);
    let response = ui.region_with_key(&region.key, rect);
    push_region_actions(&region.key, response, actions);
}

/// Render region-local drawing commands into absolute canvas coordinates.
fn render_region_draw_commands(commands: &[DrawCommand], rect: Rect, ui: &mut Ui<'_>) {
    for command in commands {
        render_region_draw_command(command, rect.origin, ui);
    }
}

/// Render one region drawing command into absolute canvas coordinates.
fn render_region_draw_command(command: &DrawCommand, origin: Point, ui: &mut Ui<'_>) {
    match command {
        DrawCommand::FillRect { rect, color } => render_fill_rect_command(*rect, *color, origin, ui),
        DrawCommand::StrokeRect {
            rect,
            thickness,
            color,
        } => render_stroke_rect_command(*rect, *thickness, *color, origin, ui),
        DrawCommand::FillCircle {
            center,
            radius,
            color,
        } => render_fill_circle_command(*center, *radius, *color, origin, ui),
        DrawCommand::StrokeCircle {
            center,
            radius,
            thickness,
            color,
        } => render_stroke_circle_command(*center, *radius, *thickness, *color, origin, ui),
        DrawCommand::Line { start, end, color } => {
            render_line_command(*start, *end, *color, origin, ui)
        }
        DrawCommand::Text {
            origin: text_origin,
            text,
            color,
            scale,
        } => render_text_command(*text_origin, text, *color, *scale, origin, ui),
    }
}

/// Render a rectangle fill command clipped to the viewport.
fn render_fill_rect_command(local: Rect, color: Color, origin: Point, ui: &mut Ui<'_>) {
    let draw_rect = offset_rect(local, origin);
    if let Some(clipped) = ui.clipped_rect(draw_rect) {
        ui.canvas().fill_rect(clipped, color);
    }
}

/// Render a rectangle stroke command clipped to the viewport.
fn render_stroke_rect_command(
    local: Rect,
    thickness: u32,
    color: Color,
    origin: Point,
    ui: &mut Ui<'_>,
) {
    let draw_rect = offset_rect(local, origin);
    if let Some(clipped) = ui.clipped_rect(draw_rect) {
        ui.canvas().stroke_rect(clipped, thickness, color);
    }
}

/// Render a circle fill command when its bounds intersect the viewport.
fn render_fill_circle_command(
    center: Point,
    radius: i32,
    color: Color,
    origin: Point,
    ui: &mut Ui<'_>,
) {
    let center = offset_point(center, origin);
    if ui.clipped_rect(circle_bounds(center, radius)).is_some() {
        ui.canvas().fill_circle(center, radius, color);
    }
}

/// Render a circle stroke command when its bounds intersect the viewport.
fn render_stroke_circle_command(
    center: Point,
    radius: i32,
    thickness: i32,
    color: Color,
    origin: Point,
    ui: &mut Ui<'_>,
) {
    let center = offset_point(center, origin);
    if ui.clipped_rect(circle_bounds(center, radius)).is_some() {
        ui.canvas().stroke_circle(center, radius, thickness, color);
    }
}

/// Compute the axis-aligned bounds for a circle at a given center and radius.
fn circle_bounds(center: Point, radius: i32) -> Rect {
    Rect {
        origin: Point {
            x: center.x - radius,
            y: center.y - radius,
        },
        size: Size {
            width: (radius * 2).max(0) as u32,
            height: (radius * 2).max(0) as u32,
        },
    }
}

/// Render a line command in absolute coordinates.
fn render_line_command(start: Point, end: Point, color: Color, origin: Point, ui: &mut Ui<'_>) {
    ui.draw_line_visual(offset_point(start, origin), offset_point(end, origin), 1.0, color);
}

/// Render a text command in absolute coordinates.
fn render_text_command(
    text_origin: Point,
    text: &str,
    color: Color,
    scale: u32,
    origin: Point,
    ui: &mut Ui<'_>,
) {
    ui.text_scaled_with_color(offset_point(text_origin, origin), text, color, scale.max(1));
}

/// Convert low-level region interaction responses into declarative UI actions.
fn push_region_actions(key: &str, response: RegionResponse, actions: &mut Vec<UiAction>) {
    let key = key.to_string();
    actions.push(region_hover_action(&key, response));
    push_region_interaction_when(
        response.pressed,
        &key,
        RegionInteractionKind::Pressed,
        response,
        actions,
    );
    push_region_interaction_when(
        response.released,
        &key,
        RegionInteractionKind::Released,
        response,
        actions,
    );
    push_region_interaction_when(
        response.dragged,
        &key,
        RegionInteractionKind::Dragged,
        response,
        actions,
    );
    push_region_interaction_when(
        response.secondary_clicked,
        &key,
        RegionInteractionKind::SecondaryClicked,
        response,
        actions,
    );
    push_region_interaction_when(
        response.double_clicked,
        &key,
        RegionInteractionKind::DoubleClicked,
        response,
        actions,
    );
}

/// Build a hover action from a region interaction response.
fn region_hover_action(key: &str, response: RegionResponse) -> UiAction {
    UiAction::RegionHover {
        key: key.to_string(),
        hovered: response.hovered,
        local_pointer: response.local_pointer,
    }
}

/// Append a region interaction action when its trigger condition is true.
fn push_region_interaction_when(
    condition: bool,
    key: &str,
    kind: RegionInteractionKind,
    response: RegionResponse,
    actions: &mut Vec<UiAction>,
) {
    if !condition {
        return;
    }
    actions.push(UiAction::RegionInteracted {
        key: key.to_string(),
        kind,
        local_pointer: response.local_pointer,
        raw_local_pointer: response.raw_local_pointer,
        alt_down: response.alt_down,
    });
}
