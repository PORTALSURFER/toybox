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
    let color = text_box.color.unwrap_or(tokens.colors.text);
    let _ = draw_text_box_line(ui, text_rect, text_box.text.as_str(), color, text_scale, text_box.align);
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
        should_clear_runtime =
            emit_text_edit_actions(text_box, edit, response.hovered, &mut runtime, ui, actions);
        ui.set_text_edit_runtime(&edit.key, runtime);
    }

    let color = text_box.color.unwrap_or(tokens.colors.text);
    if edit.editing {
        draw_text_selection_background(
            text_box.text.as_str(),
            text_rect,
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
        text_rect,
        text_box.text.as_str(),
        color,
        text_scale,
        align,
    );
    if edit.editing {
        draw_text_cursor(text_rect, runtime, text_scale, ui, tokens);
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

/// Emit pending edit actions from keyboard or pointer state.
fn emit_text_edit_actions(
    text_box: &TextBoxSpec,
    edit: &TextBoxEditSpec,
    hovered: bool,
    runtime: &mut crate::ui::TextEditRuntimeState,
    ui: &Ui<'_>,
    actions: &mut Vec<UiAction>,
) -> bool {
    if let Some(ch) = ui.key_pressed() {
        let text_len = text_box.text.chars().count();
        runtime.cursor = runtime.cursor.min(text_len);
        runtime.anchor = runtime.anchor.min(text_len);

        match ch {
            '\r' | '\n' => {
                actions.push(UiAction::TextBoxEditCommitted {
                    key: edit.key.clone(),
                    text: text_box.text.clone(),
                });
                return true;
            }
            '\u{1b}' => {
                actions.push(UiAction::TextBoxEditCanceled {
                    key: edit.key.clone(),
                });
                return true;
            }
            '\u{8}' => {
                let (updated, changed) = delete_backward(text_box.text.as_str(), runtime);
                if changed {
                    actions.push(UiAction::TextBoxEdited {
                        key: edit.key.clone(),
                        text: updated,
                    });
                }
                return false;
            }
            '\u{7f}' => {
                let (updated, changed) = delete_forward(text_box.text.as_str(), runtime);
                if changed {
                    actions.push(UiAction::TextBoxEdited {
                        key: edit.key.clone(),
                        text: updated,
                    });
                }
                return false;
            }
            '\u{1c}' => {
                move_cursor_left(runtime, ui.input().shift_down);
                return false;
            }
            '\u{1d}' => {
                move_cursor_right(runtime, text_len, ui.input().shift_down);
                return false;
            }
            '\u{1e}' => {
                move_cursor_home(runtime, ui.input().shift_down);
                return false;
            }
            '\u{1f}' => {
                move_cursor_end(runtime, text_len, ui.input().shift_down);
                return false;
            }
            _ => {
                if !is_printable_edit_char(ch) {
                    return false;
                }
                let (updated, changed) = insert_character(
                    text_box.text.as_str(),
                    runtime,
                    ch,
                    edit.max_chars.max(1),
                );
                if changed {
                    actions.push(UiAction::TextBoxEdited {
                        key: edit.key.clone(),
                        text: updated,
                    });
                }
                return false;
            }
        }
    }

    if ui.input().mouse_pressed && !hovered {
        actions.push(UiAction::TextBoxEditCommitted {
            key: edit.key.clone(),
            text: text_box.text.clone(),
        });
        return true;
    }
    false
}

/// Return true when a typed character should be inserted into editable text.
fn is_printable_edit_char(ch: char) -> bool {
    !ch.is_control()
}

/// Return a selected text range in character indices, if any.
fn selection_range(runtime: crate::ui::TextEditRuntimeState) -> Option<(usize, usize)> {
    if runtime.cursor == runtime.anchor {
        None
    } else if runtime.cursor < runtime.anchor {
        Some((runtime.cursor, runtime.anchor))
    } else {
        Some((runtime.anchor, runtime.cursor))
    }
}

/// Convert a character index to a byte index.
fn char_to_byte_index(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(byte, _)| byte)
        .unwrap_or(text.len())
}

/// Return `text` with one character-range replaced by `insert`.
fn replace_char_range(text: &str, start: usize, end: usize, insert: &str) -> String {
    let start_byte = char_to_byte_index(text, start);
    let end_byte = char_to_byte_index(text, end);
    let mut updated =
        String::with_capacity(start_byte + insert.len() + text.len().saturating_sub(end_byte));
    updated.push_str(&text[..start_byte]);
    updated.push_str(insert);
    updated.push_str(&text[end_byte..]);
    updated
}

/// Insert one character at the current cursor or replacing current selection.
fn insert_character(
    text: &str,
    runtime: &mut crate::ui::TextEditRuntimeState,
    ch: char,
    max_chars: usize,
) -> (String, bool) {
    let (start, end) = selection_range(*runtime).unwrap_or((runtime.cursor, runtime.cursor));
    let selected_len = end.saturating_sub(start);
    let text_len = text.chars().count();
    let new_len = text_len.saturating_sub(selected_len).saturating_add(1);
    if new_len > max_chars {
        return (text.to_string(), false);
    }

    let mut insert_text = String::new();
    insert_text.push(ch);
    let updated = replace_char_range(text, start, end, &insert_text);
    let changed = updated != text;
    let next = start.saturating_add(1);
    runtime.cursor = next;
    runtime.anchor = next;
    (updated, changed)
}

/// Delete the selected range or one character before the cursor.
fn delete_backward(
    text: &str,
    runtime: &mut crate::ui::TextEditRuntimeState,
) -> (String, bool) {
    if let Some((start, end)) = selection_range(*runtime) {
        let updated = replace_char_range(text, start, end, "");
        let changed = updated != text;
        runtime.cursor = start;
        runtime.anchor = start;
        return (updated, changed);
    }
    if runtime.cursor == 0 {
        return (text.to_string(), false);
    }
    let start = runtime.cursor.saturating_sub(1);
    let updated = replace_char_range(text, start, runtime.cursor, "");
    let changed = updated != text;
    runtime.cursor = start;
    runtime.anchor = start;
    (updated, changed)
}

/// Delete the selected range or one character at the cursor.
fn delete_forward(
    text: &str,
    runtime: &mut crate::ui::TextEditRuntimeState,
) -> (String, bool) {
    if let Some((start, end)) = selection_range(*runtime) {
        let updated = replace_char_range(text, start, end, "");
        let changed = updated != text;
        runtime.cursor = start;
        runtime.anchor = start;
        return (updated, changed);
    }
    let text_len = text.chars().count();
    if runtime.cursor >= text_len {
        return (text.to_string(), false);
    }
    let updated = replace_char_range(text, runtime.cursor, runtime.cursor.saturating_add(1), "");
    let changed = updated != text;
    runtime.anchor = runtime.cursor;
    (updated, changed)
}

/// Move cursor one character left.
fn move_cursor_left(runtime: &mut crate::ui::TextEditRuntimeState, keep_selection: bool) {
    let next = runtime.cursor.saturating_sub(1);
    runtime.cursor = next;
    if !keep_selection {
        runtime.anchor = next;
    }
}

/// Move cursor one character right.
fn move_cursor_right(
    runtime: &mut crate::ui::TextEditRuntimeState,
    text_len: usize,
    keep_selection: bool,
) {
    let next = runtime.cursor.saturating_add(1).min(text_len);
    runtime.cursor = next;
    if !keep_selection {
        runtime.anchor = next;
    }
}

/// Move cursor to the start of the text.
fn move_cursor_home(runtime: &mut crate::ui::TextEditRuntimeState, keep_selection: bool) {
    runtime.cursor = 0;
    if !keep_selection {
        runtime.anchor = 0;
    }
}

/// Move cursor to the end of the text.
fn move_cursor_end(
    runtime: &mut crate::ui::TextEditRuntimeState,
    text_len: usize,
    keep_selection: bool,
) {
    runtime.cursor = text_len;
    if !keep_selection {
        runtime.anchor = text_len;
    }
}

/// Draw selected text background for the visible selection range.
fn draw_text_selection_background(
    text: &str,
    text_rect: Rect,
    runtime: crate::ui::TextEditRuntimeState,
    text_scale: u32,
    ui: &mut Ui<'_>,
    tokens: &ThemeTokens,
) {
    let Some((start, end)) = selection_range(runtime) else {
        return;
    };
    if text_rect.size.width == 0 || text_rect.size.height == 0 {
        return;
    }
    let scale = text_scale.max(1);
    let char_width = (6 * scale) as i32;
    if char_width <= 0 {
        return;
    }
    let max_visible = (text_rect.size.width as i32 / char_width).max(0) as usize;
    let text_len = text.chars().count();
    let visible_end = text_len.min(max_visible);
    let visible_start = start.min(visible_end);
    let visible_stop = end.min(visible_end);
    if visible_start >= visible_stop {
        return;
    }

    let selection_x = text_rect.origin.x + (visible_start as i32 * char_width);
    let selection_width = ((visible_stop - visible_start) as i32 * char_width).max(0) as u32;
    if selection_width == 0 {
        return;
    }
    let selection_height = text_rect
        .size
        .height
        .min(8u32.saturating_mul(scale))
        .max(1);
    let selection_color = Color::rgba(
        tokens.colors.accent.r,
        tokens.colors.accent.g,
        tokens.colors.accent.b,
        96,
    );
    ui.canvas().fill_rect(
        Rect {
            origin: Point {
                x: selection_x,
                y: text_rect.origin.y,
            },
            size: Size {
                width: selection_width,
                height: selection_height,
            },
        },
        selection_color,
    );
}

/// Draw the text cursor at the current visible caret position.
fn draw_text_cursor(
    text_rect: Rect,
    runtime: crate::ui::TextEditRuntimeState,
    text_scale: u32,
    ui: &mut Ui<'_>,
    tokens: &ThemeTokens,
) {
    if text_rect.size.width == 0 || text_rect.size.height == 0 {
        return;
    }
    let scale = text_scale.max(1);
    let char_width = (6 * scale) as i32;
    if char_width <= 0 {
        return;
    }
    let max_visible = (text_rect.size.width as i32 / char_width).max(0) as usize;
    let caret_index = runtime.cursor.min(max_visible);
    let caret_x = text_rect.origin.x + (caret_index as i32 * char_width);
    let caret_width = (scale / 2).max(1);
    let caret_height = text_rect
        .size
        .height
        .min(8u32.saturating_mul(scale))
        .max(1);
    ui.canvas().fill_rect(
        Rect {
            origin: Point {
                x: caret_x,
                y: text_rect.origin.y,
            },
            size: Size {
                width: caret_width,
                height: caret_height,
            },
        },
        tokens.colors.text,
    );
}

/// Return a textbox content rectangle inset from its outer bounds.
fn inset_text_box_rect(rect: Rect, text_scale: u32) -> Rect {
    const TEXTBOX_INSET_PX: i32 = 2;
    const TEXT_LINE_HEIGHT_BASE_PX: u32 = 8;
    let inset = TEXTBOX_INSET_PX.max(0);
    let max_horizontal = (rect.size.width / 2) as i32;
    let max_vertical = (rect.size.height / 2) as i32;
    let x_inset = inset.min(max_horizontal);
    let line_height = TEXT_LINE_HEIGHT_BASE_PX.saturating_mul(text_scale.max(1));
    let max_vertical_with_text = rect.size.height.saturating_sub(line_height).saturating_div(2) as i32;
    let y_inset = inset.min(max_vertical).min(max_vertical_with_text.max(0));
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

/// Resolve text scale directly from textbox height.
///
/// This keeps text sizing deterministic and tied to textbox geometry while the
/// draw path still hard-clamps overlong text to textbox width.
fn resolve_text_box_scale(rect: Rect) -> u32 {
    const TEXT_LINE_HEIGHT_BASE_PX: u32 = 8;
    rect.size
        .height
        .saturating_div(TEXT_LINE_HEIGHT_BASE_PX)
        .max(1)
}

#[cfg(test)]
mod text_box_inset_tests {
    use super::*;

    #[test]
    fn inset_text_box_rect_applies_small_padding_when_space_allows() {
        let inset = inset_text_box_rect(
            Rect {
                origin: Point { x: 10, y: 20 },
                size: Size {
                    width: 40,
                    height: 20,
                },
            },
            2,
        );
        assert_eq!(inset.origin.x, 12);
        assert_eq!(inset.origin.y, 22);
        assert_eq!(inset.size.width, 36);
        assert_eq!(inset.size.height, 16);
    }

    #[test]
    fn inset_text_box_rect_clamps_for_tiny_bounds() {
        let inset = inset_text_box_rect(
            Rect {
                origin: Point { x: 0, y: 0 },
                size: Size {
                    width: 1,
                    height: 1,
                },
            },
            1,
        );
        assert_eq!(inset.origin.x, 0);
        assert_eq!(inset.origin.y, 0);
        assert_eq!(inset.size.width, 1);
        assert_eq!(inset.size.height, 1);
    }

    #[test]
    fn inset_text_box_rect_reduces_vertical_inset_to_preserve_line_height() {
        let inset = inset_text_box_rect(
            Rect {
                origin: Point { x: 0, y: 0 },
                size: Size {
                    width: 20,
                    height: 10,
                },
            },
            1,
        );
        assert_eq!(inset.origin.y, 1);
        assert_eq!(inset.size.height, 8);
    }

    #[test]
    fn resolve_text_box_scale_follows_textbox_height() {
        assert_eq!(
            resolve_text_box_scale(Rect {
                origin: Point { x: 0, y: 0 },
                size: Size {
                    width: 20,
                    height: 16,
                },
            }),
            2
        );
        assert_eq!(
            resolve_text_box_scale(Rect {
                origin: Point { x: 0, y: 0 },
                size: Size {
                    width: 20,
                    height: 10,
                },
            }),
            1
        );
    }
}
