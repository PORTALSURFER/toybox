
/// Measure monospaced bitmap text bounds at the given scale.
fn text_size(text: &str, scale: u32) -> Size {
    let scale = scale.max(1) as i32;
    let mut max_cols = 0i32;
    let mut lines = 1i32;
    let mut current = 0i32;
    for ch in text.chars() {
        if ch == '\n' {
            max_cols = max_cols.max(current);
            current = 0;
            lines += 1;
        } else {
            current += 1;
        }
    }
    max_cols = max_cols.max(current);
    Size {
        width: (max_cols * 6 * scale).max(0) as u32,
        height: (lines * 8 * scale).max(0) as u32,
    }
}

/// Fit a string to a single line, appending ellipsis when truncated.
fn fit_text_single_line_ellipsis(text: &str, max_width: u32, scale: u32) -> String {
    if max_width == 0 {
        return String::new();
    }

    let single_line: String = text
        .chars()
        .map(|ch| if ch == '\n' || ch == '\r' { ' ' } else { ch })
        .collect();
    if text_size(&single_line, scale).width <= max_width {
        return single_line;
    }

    let char_width = 6 * scale.max(1);
    if char_width == 0 {
        return String::new();
    }
    let max_chars = (max_width / char_width) as usize;
    if max_chars == 0 {
        return String::new();
    }
    if max_chars <= 3 {
        return ".".repeat(max_chars);
    }

    let mut fitted: String = single_line.chars().take(max_chars - 3).collect();
    fitted.push_str("...");
    fitted
}

/// Fit a string to a single line by hard-clamping to visible width.
///
/// Unlike [`fit_text_single_line_ellipsis`], this does not append ellipsis.
fn fit_text_single_line_hard_clamp(text: &str, max_width: u32, scale: u32) -> String {
    if max_width == 0 {
        return String::new();
    }

    let single_line: String = text
        .chars()
        .map(|ch| if ch == '\n' || ch == '\r' { ' ' } else { ch })
        .collect();
    if text_size(&single_line, scale).width <= max_width {
        return single_line;
    }

    let char_width = 6 * scale.max(1);
    if char_width == 0 {
        return String::new();
    }
    let max_chars = (max_width / char_width) as usize;
    if max_chars == 0 {
        return String::new();
    }

    single_line.chars().take(max_chars).collect()
}

/// Normalize knob name labels to uppercase for consistent visual style.
fn normalize_knob_name_label(label: &str) -> String {
    label.to_uppercase()
}

/// Normalize knob value labels to lowercase when they contain alphabetic text.
fn normalize_knob_value_label(label: &str) -> String {
    if label.chars().any(char::is_alphabetic) {
        label.to_lowercase()
    } else {
        label.to_string()
    }
}

/// Return a text origin centered on a target x and clamped to bounds.
fn centered_text_origin_on_x(
    left_bound: i32,
    max_width: u32,
    text_width: u32,
    target_center_x: i32,
) -> i32 {
    let raw = target_center_x - text_width as i32 / 2;
    let min_x = left_bound;
    let max_x = left_bound + (max_width as i32 - text_width as i32).max(0);
    raw.clamp(min_x, max_x)
}
