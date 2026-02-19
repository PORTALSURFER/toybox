/// Measure a text-box node.
fn measure_text_box(text_box: &TextBoxSpec, tokens: &ThemeTokens) -> Size {
    let measured = text_size(&text_box.text, tokens.typography.text_scale);
    resolve_size(text_box.layout, measured, measured)
}

/// Measure a knob node.
fn measure_knob(knob: &KnobSpec, tokens: &ThemeTokens) -> Size {
    let control = tokens.controls.knob_diameter.max(1);
    let measured = knob_block_size_for_diameter(control, tokens.typography.text_scale);
    resolve_size(knob.layout, measured, measured)
}

/// Measure a slider node.
fn measure_slider(slider: &SliderSpec, tokens: &ThemeTokens) -> Size {
    let measured = slider.control_size.unwrap_or(Size {
        width: tokens.controls.slider_width,
        height: tokens.controls.slider_height,
    });
    resolve_size(slider.layout, measured, measured)
}

/// Measure a toggle node.
fn measure_toggle(toggle: &ToggleSpec, tokens: &ThemeTokens) -> Size {
    let measured = toggle.control_size.unwrap_or(Size {
        width: tokens.controls.toggle_width,
        height: tokens.controls.toggle_height,
    });
    resolve_size(toggle.layout, measured, measured)
}

/// Measure a button node.
fn measure_button(button: &ButtonSpec, tokens: &ThemeTokens) -> Size {
    let measured = button.control_size.unwrap_or(Size {
        width: tokens.controls.button_width,
        height: tokens.controls.button_height,
    });
    resolve_size(button.layout, measured, measured)
}

/// Measure a dropdown node.
fn measure_dropdown(dropdown: &DropdownSpec, tokens: &ThemeTokens) -> Size {
    let measured = dropdown.control_size.unwrap_or(Size {
        width: tokens.controls.dropdown_width,
        height: tokens.controls.dropdown_height,
    });
    resolve_size(dropdown.layout, measured, measured)
}
