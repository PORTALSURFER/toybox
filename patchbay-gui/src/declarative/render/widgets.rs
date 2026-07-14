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
    let dot_color = resolve_widget_role_dot_color(knob.color_role, tokens, knob.disabled, knob.focused);
    let color_variants =
        resolve_control_color_variants(knob.color_role, tokens, knob.disabled, knob.focused);
    let knob_diameter = knob
        .control_size
        .map(|size| size.width.min(size.height).max(1))
        .unwrap_or(tokens.controls.knob_diameter);
    let mut knob_request =
        KnobRectRenderRequest::new(id, "", "", knob.range, knob_diameter, rect)
            .with_text_scale(tokens.typography.text_scale)
            .with_default_value(knob.default_value)
            .with_disabled(knob.disabled)
            .with_focused(knob.focused);
    if let Some(variants) = color_variants {
        knob_request = knob_request.with_color_variants(variants);
    }
    let response = ui.knob_with_labels_in_rect_scaled(&mut value, knob_request);
    draw_widget_role_dot(ui, rect, tokens, dot_color, knob.disabled);
    if !knob.disabled && response.changed {
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
    let dot_color =
        resolve_widget_role_dot_color(slider.color_role, tokens, slider.disabled, slider.focused);
    let color_variants =
        resolve_control_color_variants(slider.color_role, tokens, slider.disabled, slider.focused);
    let mut slider_request =
        SliderRectRenderRequest::new(id, "", slider.range, rect)
            .with_text_scale(tokens.typography.text_scale)
            .with_default_value(slider.default_value)
            .with_disabled(slider.disabled)
            .with_focused(slider.focused);
    if let Some(variants) = color_variants {
        slider_request = slider_request.with_color_variants(variants);
    }
    let response = ui.slider_in_rect_scaled(&mut value, slider_request);
    draw_widget_role_dot(ui, rect, tokens, dot_color, slider.disabled);
    if !slider.disabled && response.changed {
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
    let dot_color =
        resolve_widget_role_dot_color(toggle.color_role, tokens, toggle.disabled, toggle.focused);
    let color_variants =
        resolve_control_color_variants(toggle.color_role, tokens, toggle.disabled, toggle.focused);
    let control_size = toggle.control_size.unwrap_or(Size {
        width: tokens.controls.toggle_width,
        height: tokens.controls.toggle_height,
    });
    let mut toggle_request = crate::ui::ToggleRectRenderRequest::new(
        id,
        "",
        &mut value,
        control_size,
        rect,
    )
    .with_text_scale(tokens.typography.text_scale)
    .with_disabled(toggle.disabled)
    .with_focused(toggle.focused);
    if let Some(variants) = color_variants {
        toggle_request = toggle_request.with_color_variants(variants);
    }
    let response = ui.toggle_in_rect_styled(toggle_request);
    draw_widget_role_dot(ui, rect, tokens, dot_color, toggle.disabled);
    if !toggle.disabled && response.changed {
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
    let label = button.label.as_deref().unwrap_or("");
    let dot_color =
        resolve_widget_role_dot_color(button.color_role, tokens, button.disabled, button.focused);
    let color_variants =
        resolve_control_color_variants(button.color_role, tokens, button.disabled, button.focused);
    let control_size = button.control_size.unwrap_or(Size {
        width: tokens.controls.button_width,
        height: tokens.controls.button_height,
    });
    let mut button_request = crate::ui::ButtonRectRenderRequest::new(
        id,
        label,
        control_size,
        rect,
    )
    .with_text_scale(tokens.typography.text_scale)
    .with_disabled(button.disabled)
    .with_focused(button.focused);
    if let Some(variants) = color_variants {
        button_request = button_request.with_color_variants(variants);
    }
    let response = ui.button_in_rect_styled(button_request);
    draw_widget_role_dot(ui, rect, tokens, dot_color, button.disabled);
    if !button.disabled && response.clicked {
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
    let mut selected = dropdown.selected;
    let option_labels = resolve_dropdown_option_labels(dropdown);
    let option_refs: Vec<&str> = option_labels.iter().map(String::as_str).collect();
    let mut dropdown_request =
        DropdownRectRenderRequest::new(id, "", &option_refs, rect)
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
    if let Some(color) = dropdown.selected_option_background_override {
        dropdown_request =
            dropdown_request.with_selected_option_background_color(color);
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

/// Render a tab-bar node and emit actions.
fn render_tab_bar(
    tab_bar: &TabBarSpec,
    rect: Rect,
    ui: &mut Ui<'_>,
    tokens: &ThemeTokens,
    actions: &mut Vec<UiAction>,
) {
    if tab_bar.tab_count == 0 {
        return;
    }

    let labels = resolve_tab_bar_labels(tab_bar);
    let variants =
        resolve_control_color_variants(tab_bar.color_role, tokens, tab_bar.disabled, tab_bar.focused);
    let base_fill = variants.map(|v| v.base).unwrap_or(tokens.colors.surface);
    let hover_fill = variants
        .map(|v| v.hover)
        .unwrap_or(scale_alpha(tokens.colors.accent, 96));
    let active_fill = variants
        .map(|v| v.active)
        .unwrap_or(scale_alpha(tokens.colors.accent, 140));
    let disabled_fill = variants
        .map(|v| v.disabled)
        .unwrap_or(scale_alpha(tokens.colors.surface, 180));
    let border = if tab_bar.focused {
        variants.map(|v| v.focus_ring).unwrap_or(tokens.colors.accent)
    } else {
        tokens.colors.border
    };
    let text_color = if tab_bar.disabled {
        scale_alpha(tokens.colors.text, 170)
    } else {
        tokens.colors.text
    };

    let mut pointer_selection: Option<usize> = None;
    for index in 0..tab_bar.tab_count {
        let segment_rect = tab_bar_segment_rect(rect, index, tab_bar.tab_count);
        let label = labels.get(index).map(String::as_str).unwrap_or("-");
        let selected = index == tab_bar.selected;
        let (hovered, active, pressed) = if tab_bar.disabled {
            (false, false, false)
        } else {
            let response = ui.region_with_key(
                &format!("{}::tab[{index}]", tab_bar.key),
                segment_rect,
            );
            (
                response.hovered,
                response.active && ui.input().mouse_down,
                response.pressed,
            )
        };
        let fill = if tab_bar.disabled {
            disabled_fill
        } else if selected && active {
            active_fill
        } else if selected {
            hover_fill
        } else if active {
            active_fill
        } else if hovered {
            hover_fill
        } else {
            base_fill
        };
        ui.fill_rect_visual(segment_rect, fill);
        ui.stroke_rect_visual(segment_rect, 1.0, border);
        let _ = ui.text_single_line_hard_clamped_centered_in_rect_scaled(
            segment_rect,
            label,
            text_color,
            tokens.typography.text_scale,
        );

        if !tab_bar.disabled && pressed && !selected {
            pointer_selection = Some(index);
        }
    }

    if let Some(index) = pointer_selection {
        actions.push(UiAction::TabSelected {
            key: tab_bar.key.clone(),
            index,
        });
        return;
    }

    if tab_bar.disabled || !tab_bar.focused {
        return;
    }
    let Some(index) = tab_bar_selection_from_key(tab_bar.selected, tab_bar.tab_count, ui.key_pressed())
    else {
        return;
    };
    if index != tab_bar.selected {
        actions.push(UiAction::TabSelected {
            key: tab_bar.key.clone(),
            index,
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

/// Resolve the rendered labels for a tab bar.
fn resolve_tab_bar_labels(tab_bar: &TabBarSpec) -> Vec<String> {
    let numeric_fallback = || (0..tab_bar.tab_count).map(|index| (index + 1).to_string());
    match tab_bar.tab_labels.as_ref() {
        None => numeric_fallback().collect(),
        Some(labels) => numeric_fallback()
            .enumerate()
            .map(|(index, fallback)| labels.get(index).cloned().unwrap_or(fallback))
            .collect(),
    }
}

/// Resolve a new tab selection from one keypress.
fn tab_bar_selection_from_key(
    selected: usize,
    tab_count: usize,
    key_pressed: Option<char>,
) -> Option<usize> {
    let key = key_pressed?;
    if tab_count == 0 {
        return None;
    }
    match key {
        '\u{1c}' => selected.checked_sub(1),
        '\u{1d}' => (selected + 1 < tab_count).then_some(selected + 1),
        '\u{1e}' => Some(0),
        '\u{1f}' => Some(tab_count - 1),
        _ => None,
    }
}

/// Return one equal-width tab segment rect.
fn tab_bar_segment_rect(rect: Rect, index: usize, tab_count: usize) -> Rect {
    let total_width = rect.size.width;
    let count = tab_count.max(1) as u32;
    let base = total_width / count;
    let remainder = total_width % count;
    let index_u32 = index as u32;
    let extra_before = index_u32.min(remainder);
    let offset = index_u32
        .saturating_mul(base)
        .saturating_add(extra_before);
    let width = base + u32::from(index_u32 < remainder);

    Rect {
        origin: Point {
            x: rect.origin.x + offset as i32,
            y: rect.origin.y,
        },
        size: Size {
            width,
            height: rect.size.height,
        },
    }
}

/// Render an indicator node.
fn render_indicator(indicator: &IndicatorSpec, rect: Rect, ui: &mut Ui<'_>) {
    ui.indicator(rect, indicator.active);
}

/// Draw a compact color-role indicator dot in the widget corner.
fn draw_widget_role_dot(
    ui: &mut Ui<'_>,
    rect: Rect,
    tokens: &ThemeTokens,
    dot_color: Option<Color>,
    disabled: bool,
) {
    let Some(dot_color) = dot_color else {
        return;
    };
    if rect.size.width < 10 || rect.size.height < 10 {
        return;
    }

    let radius = ((rect.size.width.min(rect.size.height) as i32) / 7).clamp(2, 4);
    let center = Point {
        x: rect.origin.x + rect.size.width as i32 - radius - 2,
        y: rect.origin.y + radius + 2,
    };
    let fill = if disabled {
        scale_alpha(dot_color, 140)
    } else {
        scale_alpha(dot_color, 220)
    };
    ui.canvas().fill_circle(center, radius, fill);
    ui.canvas()
        .stroke_circle(center, radius, 1, tokens.colors.border);
}

/// Resolve optional UI control color variants for a declarative widget role.
fn resolve_control_color_variants(
    role: Option<WidgetColorRole>,
    tokens: &ThemeTokens,
    disabled: bool,
    focused: bool,
) -> Option<crate::ui::ControlColorVariants> {
    let role = role?;
    let resolver = DefaultWidgetColorResolver::new();
    let variants = resolver.resolve(
        role,
        WidgetColorContext {
            tokens: *tokens,
            disabled,
            focused,
        },
    );
    Some(crate::ui::ControlColorVariants {
        base: variants.base,
        hover: variants.hover,
        active: variants.active,
        disabled: variants.disabled,
        focus_ring: variants.focus_ring,
    })
}

/// Resolve a single indicator-dot color for one widget color role.
fn resolve_widget_role_dot_color(
    role: Option<WidgetColorRole>,
    tokens: &ThemeTokens,
    disabled: bool,
    focused: bool,
) -> Option<Color> {
    let role = role?;
    let resolver = DefaultWidgetColorResolver::new();
    let variants = resolver.resolve(
        role,
        WidgetColorContext {
            tokens: *tokens,
            disabled,
            focused,
        },
    );
    Some(if disabled { variants.disabled } else { variants.base })
}

/// Return `color` with alpha multiplied by `alpha` in `0..=255`.
fn scale_alpha(color: Color, alpha: u8) -> Color {
    let scaled = (u16::from(color.a) * u16::from(alpha) + 127) / 255;
    Color::rgba(color.r, color.g, color.b, scaled as u8)
}

/// Render a curve-editor node and emit model change actions.
fn render_curve_editor(
    curve_editor: &CurveEditorSpec,
    segment_move: Option<CurveSegmentMoveOptions>,
    point_horizontal_constraint: Option<CurveEditorModifier>,
    rect: Rect,
    ui: &mut Ui<'_>,
    actions: &mut Vec<UiAction>,
) {
    let mut model = curve_editor.model.clone();
    let request = crate::ui::CurveEditorRectRenderRequest::new(
        WidgetId::from_label(&curve_editor.key),
        rect,
        curve_editor.style.clone(),
        curve_editor.grid.clone(),
        curve_editor.interaction.clone(),
        curve_editor.playhead_x,
    );
    let request = if let Some(segment_move) = segment_move {
        request.segment_move(segment_move)
    } else {
        request
    };
    let request = if let Some(modifier) = point_horizontal_constraint {
        request.point_horizontal_constraint(modifier)
    } else {
        request
    };
    let response = ui.curve_editor_in_rect(&mut model, request);
    if response.changed {
        actions.push(UiAction::CurveEditorChanged {
            key: curve_editor.key.clone(),
            model,
        });
    }
}

#[cfg(test)]
mod dropdown_label_tests {
    use super::{
        resolve_dropdown_option_labels, resolve_tab_bar_labels, tab_bar_segment_rect,
        tab_bar_selection_from_key,
    };
    use crate::canvas::{Point, Rect, Size};
    use crate::declarative::{DropdownSpec, TabBarSpec};

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

    #[test]
    fn tab_labels_default_to_numeric_indices() {
        let tab_bar = TabBarSpec::new("family", 3, 0);
        assert_eq!(
            resolve_tab_bar_labels(&tab_bar),
            vec!["1".to_string(), "2".to_string(), "3".to_string()]
        );
    }

    #[test]
    fn tab_labels_use_custom_entries_with_numeric_fallback() {
        let tab_bar = TabBarSpec::new("family", 4, 0)
            .tab_labels(vec!["Kick".into(), "Ride".into()]);
        assert_eq!(
            resolve_tab_bar_labels(&tab_bar),
            vec![
                "Kick".to_string(),
                "Ride".to_string(),
                "3".to_string(),
                "4".to_string()
            ]
        );
    }

    #[test]
    fn tab_selection_key_rules_do_not_wrap() {
        assert_eq!(tab_bar_selection_from_key(0, 3, Some('\u{1c}')), None);
        assert_eq!(tab_bar_selection_from_key(0, 3, Some('\u{1d}')), Some(1));
        assert_eq!(tab_bar_selection_from_key(2, 3, Some('\u{1d}')), None);
        assert_eq!(tab_bar_selection_from_key(1, 3, Some('\u{1e}')), Some(0));
        assert_eq!(tab_bar_selection_from_key(1, 3, Some('\u{1f}')), Some(2));
    }

    #[test]
    fn tab_segment_distribution_is_equal_width_with_deterministic_remainder() {
        let rect = Rect {
            origin: Point { x: 5, y: 8 },
            size: Size {
                width: 10,
                height: 20,
            },
        };
        let first = tab_bar_segment_rect(rect, 0, 3);
        let second = tab_bar_segment_rect(rect, 1, 3);
        let third = tab_bar_segment_rect(rect, 2, 3);

        assert_eq!(first.size.width, 4);
        assert_eq!(second.size.width, 3);
        assert_eq!(third.size.width, 3);
        assert_eq!(first.origin.x, 5);
        assert_eq!(second.origin.x, 9);
        assert_eq!(third.origin.x, 12);
    }
}
