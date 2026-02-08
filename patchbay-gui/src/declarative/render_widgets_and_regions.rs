
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
        match command {
            DrawCommand::FillRect { rect: local, color } => {
                let draw_rect = offset_rect(*local, rect.origin);
                if let Some(clipped) = ui.clipped_rect(draw_rect) {
                    ui.canvas().fill_rect(clipped, *color);
                }
            }
            DrawCommand::StrokeRect {
                rect: local,
                thickness,
                color,
            } => {
                let draw_rect = offset_rect(*local, rect.origin);
                if let Some(clipped) = ui.clipped_rect(draw_rect) {
                    ui.canvas().stroke_rect(clipped, *thickness, *color);
                }
            }
            DrawCommand::FillCircle {
                center,
                radius,
                color,
            } => {
                let center = offset_point(*center, rect.origin);
                let bounds = Rect {
                    origin: Point {
                        x: center.x - *radius,
                        y: center.y - *radius,
                    },
                    size: Size {
                        width: (*radius * 2).max(0) as u32,
                        height: (*radius * 2).max(0) as u32,
                    },
                };
                if ui.clipped_rect(bounds).is_some() {
                    ui.canvas().fill_circle(center, *radius, *color);
                }
            }
            DrawCommand::StrokeCircle {
                center,
                radius,
                thickness,
                color,
            } => {
                let center = offset_point(*center, rect.origin);
                let bounds = Rect {
                    origin: Point {
                        x: center.x - *radius,
                        y: center.y - *radius,
                    },
                    size: Size {
                        width: (*radius * 2).max(0) as u32,
                        height: (*radius * 2).max(0) as u32,
                    },
                };
                if ui.clipped_rect(bounds).is_some() {
                    ui.canvas()
                        .stroke_circle(center, *radius, *thickness, *color);
                }
            }
            DrawCommand::Line { start, end, color } => {
                ui.canvas().draw_line(
                    offset_point(*start, rect.origin),
                    offset_point(*end, rect.origin),
                    *color,
                );
            }
            DrawCommand::Text {
                origin,
                text,
                color,
                scale,
            } => {
                ui.text_scaled_with_color(
                    offset_point(*origin, rect.origin),
                    text,
                    *color,
                    (*scale).max(1),
                );
            }
        }
    }
}

/// Convert low-level region interaction responses into declarative UI actions.
fn push_region_actions(key: &str, response: RegionResponse, actions: &mut Vec<UiAction>) {
    let local_pointer = response.local_pointer;
    let raw_local_pointer = response.raw_local_pointer;
    actions.push(UiAction::RegionHover {
        key: key.to_string(),
        hovered: response.hovered,
        local_pointer,
    });

    if response.pressed {
        actions.push(UiAction::RegionInteracted {
            key: key.to_string(),
            kind: RegionInteractionKind::Pressed,
            local_pointer,
            raw_local_pointer,
            alt_down: response.alt_down,
        });
    }
    if response.released {
        actions.push(UiAction::RegionInteracted {
            key: key.to_string(),
            kind: RegionInteractionKind::Released,
            local_pointer,
            raw_local_pointer,
            alt_down: response.alt_down,
        });
    }
    if response.dragged {
        actions.push(UiAction::RegionInteracted {
            key: key.to_string(),
            kind: RegionInteractionKind::Dragged,
            local_pointer,
            raw_local_pointer,
            alt_down: response.alt_down,
        });
    }
    if response.secondary_clicked {
        actions.push(UiAction::RegionInteracted {
            key: key.to_string(),
            kind: RegionInteractionKind::SecondaryClicked,
            local_pointer,
            raw_local_pointer,
            alt_down: response.alt_down,
        });
    }
    if response.double_clicked {
        actions.push(UiAction::RegionInteracted {
            key: key.to_string(),
            kind: RegionInteractionKind::DoubleClicked,
            local_pointer,
            raw_local_pointer,
            alt_down: response.alt_down,
        });
    }
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
