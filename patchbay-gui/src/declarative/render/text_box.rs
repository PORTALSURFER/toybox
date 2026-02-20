/// Render a text-box node.
fn render_text_box(
    text_box: &TextBoxSpec,
    rect: Rect,
    ui: &mut Ui<'_>,
    tokens: &ThemeTokens,
    actions: &mut Vec<UiAction>,
) {
    let text_rect = inset_text_box_rect(rect);
    if let Some(edit) = text_box.edit.as_ref() {
        render_editable_text_box(text_box, edit, rect, text_rect, ui, tokens, actions);
        return;
    }

    let color = text_box.color.unwrap_or(tokens.colors.text);
    let _ = ui.text_single_line_hard_clamped_in_rect_scaled(
        text_rect,
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
    text_rect: Rect,
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
        text_rect,
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

/// Return a textbox content rectangle inset from its outer bounds.
fn inset_text_box_rect(rect: Rect) -> Rect {
    const TEXTBOX_INSET_PX: i32 = 2;
    let inset = TEXTBOX_INSET_PX.max(0);
    let max_horizontal = (rect.size.width / 2) as i32;
    let max_vertical = (rect.size.height / 2) as i32;
    let x_inset = inset.min(max_horizontal);
    let y_inset = inset.min(max_vertical);
    Rect {
        origin: Point {
            x: rect.origin.x + x_inset,
            y: rect.origin.y + y_inset,
        },
        size: Size {
            width: rect.size.width.saturating_sub((x_inset as u32).saturating_mul(2)),
            height: rect
                .size
                .height
                .saturating_sub((y_inset as u32).saturating_mul(2)),
        },
    }
}

#[cfg(test)]
mod text_box_inset_tests {
    use super::*;

    #[test]
    fn inset_text_box_rect_applies_small_padding_when_space_allows() {
        let inset = inset_text_box_rect(Rect {
            origin: Point { x: 10, y: 20 },
            size: Size {
                width: 40,
                height: 20,
            },
        });
        assert_eq!(inset.origin.x, 12);
        assert_eq!(inset.origin.y, 22);
        assert_eq!(inset.size.width, 36);
        assert_eq!(inset.size.height, 16);
    }

    #[test]
    fn inset_text_box_rect_clamps_for_tiny_bounds() {
        let inset = inset_text_box_rect(Rect {
            origin: Point { x: 0, y: 0 },
            size: Size {
                width: 1,
                height: 1,
            },
        });
        assert_eq!(inset.origin.x, 0);
        assert_eq!(inset.origin.y, 0);
        assert_eq!(inset.size.width, 1);
        assert_eq!(inset.size.height, 1);
    }
}
