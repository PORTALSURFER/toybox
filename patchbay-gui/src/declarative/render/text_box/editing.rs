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
