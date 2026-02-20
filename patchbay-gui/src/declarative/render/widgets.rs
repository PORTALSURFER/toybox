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
    let knob_request = KnobRectRenderRequest::new(id, "", "", knob.range, tokens.controls.knob_diameter, rect)
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
        SliderRectRenderRequest::new(id, "", slider.range, control_size, rect)
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
        "",
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
        "",
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
    let option_labels = resolve_dropdown_option_labels(dropdown);
    let option_refs: Vec<&str> = option_labels.iter().map(String::as_str).collect();
    let mut dropdown_request =
        DropdownRectRenderRequest::new(id, "", &option_refs, control_size, rect)
            .with_text_scale(tokens.typography.text_scale);
    if let Some(color) = dropdown.background_override {
        dropdown_request = dropdown_request.with_background_color(color);
    }
    if let Some(color) = dropdown.hover_background_override {
        dropdown_request = dropdown_request.with_hover_background_color(color);
    }
    if let Some(color) = dropdown.active_background_override {
        dropdown_request = dropdown_request.with_active_background_color(color);
    }
    if let Some(color) = dropdown.outline_override {
        dropdown_request = dropdown_request.with_outline_color(color);
    }
    if let Some(color) = dropdown.text_color_override {
        dropdown_request = dropdown_request.with_text_color(color);
    }
    let response = ui.dropdown_in_rect_scaled(&mut selected, dropdown_request);
    if response.changed {
        actions.push(UiAction::DropdownSelected {
            key: dropdown.key.clone(),
            index: selected,
        });
    }
    if response.hovered && ui.input().mouse_double_clicked {
        actions.push(UiAction::DropdownDoubleClicked {
            key: dropdown.key.clone(),
        });
    }
}

/// Resolve the rendered option labels for a dropdown.
fn resolve_dropdown_option_labels(dropdown: &DropdownSpec) -> Vec<String> {
    let numeric_fallback = || (0..dropdown.option_count).map(|index| (index + 1).to_string());
    match dropdown.option_labels.as_ref() {
        None => numeric_fallback().collect(),
        Some(labels) => numeric_fallback()
            .enumerate()
            .map(|(index, fallback)| labels.get(index).cloned().unwrap_or(fallback))
            .collect(),
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

#[cfg(test)]
mod dropdown_label_tests {
    use super::resolve_dropdown_option_labels;
    use crate::declarative::DropdownSpec;

    #[test]
    fn dropdown_labels_default_to_numeric_indices() {
        let dropdown = DropdownSpec::new("division", 3, 0);
        assert_eq!(
            resolve_dropdown_option_labels(&dropdown),
            vec!["1".to_string(), "2".to_string(), "3".to_string()]
        );
    }

    #[test]
    fn dropdown_labels_use_custom_entries_with_numeric_fallback() {
        let dropdown = DropdownSpec::new("division", 4, 0)
            .option_labels(vec!["1/16".into(), "1/8T".into()]);
        assert_eq!(
            resolve_dropdown_option_labels(&dropdown),
            vec![
                "1/16".to_string(),
                "1/8T".to_string(),
                "3".to_string(),
                "4".to_string()
            ]
        );
    }
}
