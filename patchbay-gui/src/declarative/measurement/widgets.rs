/// Measure a label node.
fn measure_label(label: &LabelSpec, tokens: &ThemeTokens) -> Size {
    let measured = text_size(&label.text, tokens.typography.text_scale);
    resolve_size(label.layout, measured, measured)
}

/// Measure a knob node.
fn measure_knob(knob: &KnobSpec, tokens: &ThemeTokens) -> Size {
    let control = tokens.controls.knob_diameter.max(1);
    let text_scale = knob.text_scale.unwrap_or(tokens.typography.text_scale);
    let measured = knob_block_size_for_diameter(control, text_scale);
    resolve_size(knob.layout, measured, measured)
}

/// Measure a slider node.
fn measure_slider(slider: &SliderSpec, tokens: &ThemeTokens) -> Size {
    let control = slider.control_size.unwrap_or(Size {
        width: tokens.controls.slider_width,
        height: tokens.controls.slider_height,
    });
    let label_h = if slider.label.is_empty() {
        0
    } else {
        8 * tokens.typography.text_scale.max(1)
    };
    let label = text_size(&slider.label, tokens.typography.text_scale);
    let measured = Size {
        width: control.width.max(label.width),
        height: control.height + label_h,
    };
    resolve_size(slider.layout, measured, measured)
}

/// Measure a toggle node.
fn measure_toggle(toggle: &ToggleSpec, tokens: &ThemeTokens) -> Size {
    let control = toggle.control_size.unwrap_or(Size {
        width: tokens.controls.toggle_width,
        height: tokens.controls.toggle_height,
    });
    let label_h = if toggle.label.is_empty() {
        0
    } else {
        8 * tokens.typography.text_scale.max(1)
    };
    let label = text_size(&toggle.label, tokens.typography.text_scale);
    let measured = Size {
        width: control.width.max(label.width),
        height: control.height + label_h,
    };
    resolve_size(toggle.layout, measured, measured)
}

/// Measure a button node.
fn measure_button(button: &ButtonSpec, tokens: &ThemeTokens) -> Size {
    let control = button.control_size.unwrap_or(Size {
        width: tokens.controls.button_width,
        height: tokens.controls.button_height,
    });
    let label = text_size(&button.label, tokens.typography.text_scale);
    let measured = Size {
        width: control.width.max(label.width + 8),
        height: control.height.max(label.height + 4),
    };
    resolve_size(button.layout, measured, measured)
}

/// Measure a dropdown node.
fn measure_dropdown(dropdown: &DropdownSpec, tokens: &ThemeTokens) -> Size {
    let control = dropdown.control_size.unwrap_or(Size {
        width: tokens.controls.dropdown_width,
        height: tokens.controls.dropdown_height,
    });
    let label_h = if dropdown.label.is_empty() {
        0
    } else {
        8 * tokens.typography.text_scale.max(1)
    };
    let label = text_size(&dropdown.label, tokens.typography.text_scale);
    let measured = Size {
        width: control.width.max(label.width),
        height: control.height + label_h,
    };
    resolve_size(dropdown.layout, measured, measured)
}
