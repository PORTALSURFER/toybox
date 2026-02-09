
/// Render a knob node and emit actions.
fn render_knob(
    knob: &KnobSpec,
    rect: Rect,
    ui: &mut Ui<'_>,
    tokens: &ThemeTokens,
    actions: &mut Vec<UiAction>,
) {
    let id = WidgetId::from_label(&knob.key);
    let mut value = knob.value;
    let value_label = knob
        .value_label
        .clone()
        .unwrap_or_else(|| format_value(knob.value));
    let knob_request = KnobRectRenderRequest::new(
        id,
        &knob.label,
        &value_label,
        knob.range,
        tokens.controls.knob_diameter,
        rect,
    )
    .with_text_scale(tokens.typography.text_scale);
    let response = ui.knob_with_labels_in_rect_scaled(&mut value, knob_request);
    if response.changed {
        actions.push(UiAction::KnobChanged {
            key: knob.key.clone(),
            value,
        });
    }
}

/// Render a slider node and emit actions.
fn render_slider(
    slider: &SliderSpec,
    rect: Rect,
    ui: &mut Ui<'_>,
    tokens: &ThemeTokens,
    actions: &mut Vec<UiAction>,
) {
    let id = WidgetId::from_label(&slider.key);
    let mut value = slider.value;
    let control_size = slider.control_size.unwrap_or(Size {
        width: tokens.controls.slider_width,
        height: tokens.controls.slider_height,
    });
    let slider_request =
        SliderRectRenderRequest::new(id, &slider.label, slider.range, control_size, rect)
            .with_text_scale(tokens.typography.text_scale);
    let response = ui.slider_in_rect_scaled(&mut value, slider_request);
    if response.changed {
        actions.push(UiAction::SliderChanged {
            key: slider.key.clone(),
            value,
        });
    }
}

/// Render a toggle node and emit actions.
fn render_toggle(
    toggle: &ToggleSpec,
    rect: Rect,
    ui: &mut Ui<'_>,
    tokens: &ThemeTokens,
    actions: &mut Vec<UiAction>,
) {
    let id = WidgetId::from_label(&toggle.key);
    let mut value = toggle.value;
    let control_size = toggle.control_size.unwrap_or(Size {
        width: tokens.controls.toggle_width,
        height: tokens.controls.toggle_height,
    });
    let response = ui.toggle_in_rect_scaled(
        id,
        &toggle.label,
        &mut value,
        control_size,
        rect,
        tokens.typography.text_scale,
    );
    if response.changed {
        actions.push(UiAction::ToggleChanged {
            key: toggle.key.clone(),
            value,
        });
    }
}

/// Render a button node and emit actions.
fn render_button(
    button: &ButtonSpec,
    rect: Rect,
    ui: &mut Ui<'_>,
    tokens: &ThemeTokens,
    actions: &mut Vec<UiAction>,
) {
    let id = WidgetId::from_label(&button.key);
    let control_size = button.control_size.unwrap_or(Size {
        width: tokens.controls.button_width,
        height: tokens.controls.button_height,
    });
    let response = ui.button_in_rect_scaled(
        id,
        &button.label,
        control_size,
        rect,
        tokens.typography.text_scale,
    );
    if response.clicked {
        actions.push(UiAction::ButtonPressed {
            key: button.key.clone(),
        });
    }
}

/// Render a dropdown node and emit actions.
fn render_dropdown(
    dropdown: &DropdownSpec,
    rect: Rect,
    ui: &mut Ui<'_>,
    tokens: &ThemeTokens,
    actions: &mut Vec<UiAction>,
) {
    let id = WidgetId::from_label(&dropdown.key);
    let control_size = dropdown.control_size.unwrap_or(Size {
        width: tokens.controls.dropdown_width,
        height: tokens.controls.dropdown_height,
    });
    let mut selected = dropdown.selected;
    let option_refs: Vec<&str> = dropdown.options.iter().map(String::as_str).collect();
    let dropdown_request = DropdownRectRenderRequest::new(
        id,
        &dropdown.label,
        &option_refs,
        control_size,
        rect,
    )
    .with_text_scale(tokens.typography.text_scale);
    let response = ui.dropdown_in_rect_scaled(&mut selected, dropdown_request);
    if response.changed {
        actions.push(UiAction::DropdownSelected {
            key: dropdown.key.clone(),
            index: selected,
        });
    }
}

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
        DrawCommand::Line { start, end, color } => render_line_command(*start, *end, *color, origin, ui),
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
    ui.canvas()
        .draw_line(offset_point(start, origin), offset_point(end, origin), color);
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
    push_region_interaction_when(response.pressed, &key, RegionInteractionKind::Pressed, response, actions);
    push_region_interaction_when(
        response.released,
        &key,
        RegionInteractionKind::Released,
        response,
        actions,
    );
    push_region_interaction_when(response.dragged, &key, RegionInteractionKind::Dragged, response, actions);
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

/// Render an indicator node.
fn render_indicator(indicator: &IndicatorSpec, rect: Rect, ui: &mut Ui<'_>) {
    ui.indicator(
        Rect {
            origin: rect.origin,
            size: indicator.size,
        },
        indicator.active,
    );
}
