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

    let color = text_box.color.unwrap_or(tokens.colors.text);
    let _ = ui.text_single_line_hard_clamped_in_rect_scaled(
        rect,
        &text_box.text,
        color,
        tokens.typography.text_scale,
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
    let response = ui.region_with_key(&edit.key, rect);
    if response.double_clicked && !edit.editing {
        actions.push(UiAction::TextBoxEditRequested {
            key: edit.key.clone(),
        });
    }

    if edit.editing {
        emit_text_edit_actions(text_box, edit, response.hovered, ui, actions);
    }

    let color = text_box.color.unwrap_or(tokens.colors.text);
    let mut rendered_text = text_box.text.clone();
    if edit.editing {
        rendered_text.push('|');
    }
    let _ = ui.text_single_line_hard_clamped_in_rect_scaled(
        rect,
        &rendered_text,
        color,
        tokens.typography.text_scale,
    );
}

/// Emit pending edit actions from keyboard or pointer state.
fn emit_text_edit_actions(
    text_box: &TextBoxSpec,
    edit: &TextBoxEditSpec,
    hovered: bool,
    ui: &Ui<'_>,
    actions: &mut Vec<UiAction>,
) {
    if let Some(ch) = ui.key_pressed() {
        match ch {
            '\r' | '\n' => {
                actions.push(UiAction::TextBoxEditCommitted {
                    key: edit.key.clone(),
                    text: text_box.text.clone(),
                });
                return;
            }
            '\u{1b}' => {
                actions.push(UiAction::TextBoxEditCanceled {
                    key: edit.key.clone(),
                });
                return;
            }
            '\u{8}' => {
                let mut updated = text_box.text.clone();
                updated.pop();
                actions.push(UiAction::TextBoxEdited {
                    key: edit.key.clone(),
                    text: updated,
                });
                return;
            }
            _ => {
                if !is_printable_edit_char(ch) {
                    return;
                }
                let mut updated = text_box.text.clone();
                if updated.chars().count() < edit.max_chars.max(1) {
                    updated.push(ch);
                    actions.push(UiAction::TextBoxEdited {
                        key: edit.key.clone(),
                        text: updated,
                    });
                }
                return;
            }
        }
    }

    if ui.input().mouse_pressed && !hovered {
        actions.push(UiAction::TextBoxEditCommitted {
            key: edit.key.clone(),
            text: text_box.text.clone(),
        });
    }
}

/// Return true when a typed character should be inserted into editable text.
fn is_printable_edit_char(ch: char) -> bool {
    !ch.is_control()
}
