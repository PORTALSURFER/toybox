/// Draw selected text background for the visible selection range.
fn draw_text_selection_background(
    text: &str,
    line_rect: Rect,
    runtime: crate::ui::TextEditRuntimeState,
    text_scale: u32,
    ui: &mut Ui<'_>,
    tokens: &ThemeTokens,
) {
    let Some((start, end)) = selection_range(runtime) else {
        return;
    };
    if line_rect.size.width == 0 || line_rect.size.height == 0 {
        return;
    }
    let scale = text_scale.max(1);
    let char_width = (6 * scale).max(1) as i32;
    let max_visible = visible_char_capacity_for_line(line_rect, text_scale);
    if max_visible == 0 {
        return;
    }
    let text_len = text.chars().count();
    let visible_end = text_len.min(max_visible);
    let visible_start = start.min(visible_end);
    let visible_stop = end.min(visible_end);
    if visible_start >= visible_stop {
        return;
    }

    let selection_x = caret_x_for_index(line_rect, text_scale, visible_start);
    let selection_width = ((visible_stop - visible_start) as i32 * char_width).max(0) as u32;
    if selection_width == 0 {
        return;
    }
    let selection_height = line_rect
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
                y: line_rect.origin.y,
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
    line_rect: Rect,
    runtime: crate::ui::TextEditRuntimeState,
    text_scale: u32,
    ui: &mut Ui<'_>,
    tokens: &ThemeTokens,
) {
    if line_rect.size.width == 0 || line_rect.size.height == 0 {
        return;
    }
    let scale = text_scale.max(1);
    let max_visible = visible_char_capacity_for_line(line_rect, text_scale);
    if max_visible == 0 {
        return;
    }
    let caret_index = runtime.cursor.min(max_visible);
    let caret_x = caret_x_for_index(line_rect, text_scale, caret_index);
    let caret_width = (scale / 2).max(1);
    let caret_height = line_rect
        .size
        .height
        .min(8u32.saturating_mul(scale))
        .max(1);
    ui.canvas().fill_rect(
        Rect {
            origin: Point {
                x: caret_x,
                y: line_rect.origin.y,
            },
            size: Size {
                width: caret_width,
                height: caret_height,
            },
        },
        tokens.colors.text,
    );
}

/// Resolve the x-position of one caret index inside a textbox line.
fn caret_x_for_index(line_rect: Rect, text_scale: u32, index: usize) -> i32 {
    let char_width = 6i32.saturating_mul(text_scale.max(1) as i32).max(1);
    line_rect
        .origin
        .x
        .saturating_add(i32::try_from(index).unwrap_or(i32::MAX).saturating_mul(char_width))
}
