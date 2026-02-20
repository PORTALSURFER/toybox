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
