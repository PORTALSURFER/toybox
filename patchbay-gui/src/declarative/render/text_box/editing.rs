/// Static interaction context for one editable textbox render pass.
#[derive(Clone, Copy)]
struct TextEditInteractionCtx {
    /// Region response snapshot from the textbox interaction pass.
    region: crate::ui::RegionResponse,
    /// Single-line text draw rectangle used for caret and selection mapping.
    line_rect: crate::canvas::Rect,
    /// Resolved text scale for this textbox.
    text_scale: u32,
}

/// Emit pending edit actions from keyboard or pointer state.
fn emit_text_edit_actions(
    text_box: &TextBoxSpec,
    edit: &TextBoxEditSpec,
    ctx: TextEditInteractionCtx,
    runtime: &mut crate::ui::TextEditRuntimeState,
    ui: &Ui<'_>,
    actions: &mut Vec<UiAction>,
) -> bool {
    let text_len = text_box.text.chars().count();
    runtime.cursor = runtime.cursor.min(text_len);
    runtime.anchor = runtime.anchor.min(text_len);
    if ctx.region.released || !ui.input().mouse_down {
        runtime.pointer_selecting = false;
    }

    apply_pointer_edit_selection(
        text_box.text.as_str(),
        ctx.line_rect,
        ctx.text_scale,
        ctx.region,
        runtime,
        ui,
    );

    if let Some(ch) = ui.key_pressed() {
        runtime.pointer_selecting = false;

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

    if ui.input().mouse_pressed && !ctx.region.hovered {
        actions.push(UiAction::TextBoxEditCommitted {
            key: edit.key.clone(),
            text: text_box.text.clone(),
        });
        return true;
    }
    false
}

/// Apply pointer-driven cursor moves and selection changes for editable text.
fn apply_pointer_edit_selection(
    text: &str,
    line_rect: crate::canvas::Rect,
    text_scale: u32,
    region: crate::ui::RegionResponse,
    runtime: &mut crate::ui::TextEditRuntimeState,
    ui: &Ui<'_>,
) {
    if !(region.pressed || (region.active && ui.input().mouse_down)) {
        return;
    }

    let pointer_index =
        pointer_char_index_in_line(text, line_rect, text_scale, ui.input().pointer_pos.x);
    if region.pressed {
        runtime.pointer_selecting = true;
        if ui.input().shift_down {
            runtime.cursor = pointer_index;
            return;
        }
        runtime.cursor = pointer_index;
        runtime.anchor = pointer_index;
        return;
    }

    if runtime.pointer_selecting && region.active && ui.input().mouse_down {
        runtime.cursor = pointer_index;
    }
}

/// Return the visible text character capacity for a line rectangle.
fn visible_char_capacity_for_line(line_rect: crate::canvas::Rect, text_scale: u32) -> usize {
    let scale = text_scale.max(1) as i32;
    let char_width = 6i32.saturating_mul(scale);
    if char_width <= 0 {
        return 0;
    }
    (line_rect.size.width as i32 / char_width).max(0) as usize
}

/// Map a pointer x-position to one caret index inside the visible text range.
fn pointer_char_index_in_line(
    text: &str,
    line_rect: crate::canvas::Rect,
    text_scale: u32,
    pointer_x: i32,
) -> usize {
    let scale = text_scale.max(1) as i32;
    let char_width = 6i32.saturating_mul(scale).max(1);
    let visible_len = text
        .chars()
        .count()
        .min(visible_char_capacity_for_line(line_rect, text_scale));

    let relative_x = pointer_x.saturating_sub(line_rect.origin.x);
    let rounded = if relative_x <= 0 {
        0
    } else {
        relative_x
            .saturating_add(char_width / 2)
            .saturating_div(char_width)
    };
    rounded.clamp(0, visible_len as i32) as usize
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
