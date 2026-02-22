/// Render a text-box node.
fn render_text_box(
    text_box: &TextBoxSpec,
    rect: Rect,
    ui: &mut Ui<'_>,
    tokens: &ThemeTokens,
    actions: &mut Vec<UiAction>,
) {
    if let Some(edit) = text_box.edit.as_ref() {
        render_editable_text_box(text_box, edit, rect, ui, tokens, actions);
        return;
    }

    let text_scale = resolve_text_box_scale(rect);
    let text_rect = inset_text_box_rect(rect, text_scale);
    let line_rect = resolve_text_line_rect(text_rect, text_scale);
    let color = text_box.color.unwrap_or(tokens.colors.text);
    let _ = draw_text_box_line(
        ui,
        line_rect,
        text_box.text.as_str(),
        color,
        text_scale,
        text_box.align,
    );
}

/// Render an editable text box and emit edit actions.
fn render_editable_text_box(
    text_box: &TextBoxSpec,
    edit: &TextBoxEditSpec,
    rect: Rect,
    ui: &mut Ui<'_>,
    tokens: &ThemeTokens,
    actions: &mut Vec<UiAction>,
) {
    let text_scale = resolve_text_box_scale(rect);
    let text_rect = inset_text_box_rect(rect, text_scale);
    let line_rect = resolve_text_line_rect(text_rect, text_scale);
    let response = ui.region_with_key(&edit.key, rect);
    if response.double_clicked && !edit.editing {
        actions.push(UiAction::TextBoxEditRequested {
            key: edit.key.clone(),
        });
    }

    if !edit.editing {
        ui.clear_text_edit_runtime(&edit.key);
    }
    let mut runtime = ui.begin_text_edit_runtime(&edit.key, text_box.text.chars().count());

    let mut should_clear_runtime = false;
    if edit.editing {
        runtime.cursor_pulse_frame = runtime.cursor_pulse_frame.wrapping_add(1);
        should_clear_runtime = emit_text_edit_actions(
            text_box,
            edit,
            TextEditInteractionCtx {
                region: response,
                line_rect,
                text_scale,
            },
            &mut runtime,
            ui,
            actions,
        );
        ui.set_text_edit_runtime(&edit.key, runtime);
    }

    let color = text_box.color.unwrap_or(tokens.colors.text);
    if edit.editing {
        draw_text_selection_background(
            text_box.text.as_str(),
            line_rect,
            runtime,
            text_scale,
            ui,
            tokens,
        );
    }
    let align = if edit.editing {
        TextBoxAlign::Start
    } else {
        text_box.align
    };
    let _ = draw_text_box_line(
        ui,
        line_rect,
        text_box.text.as_str(),
        color,
        text_scale,
        align,
    );
    if edit.editing {
        draw_text_cursor(line_rect, runtime, text_scale, ui, tokens);
    }
    if should_clear_runtime {
        ui.clear_text_edit_runtime(&edit.key);
    }
}

/// Draw one hard-clamped textbox line using the requested alignment mode.
fn draw_text_box_line(
    ui: &mut Ui<'_>,
    rect: Rect,
    text: &str,
    color: Color,
    text_scale: u32,
    align: TextBoxAlign,
) -> Size {
    match align {
        TextBoxAlign::Start => {
            ui.text_single_line_hard_clamped_in_rect_scaled(rect, text, color, text_scale)
        }
        TextBoxAlign::Center => ui.text_single_line_hard_clamped_centered_in_rect_scaled(
            rect, text, color, text_scale,
        ),
    }
}
