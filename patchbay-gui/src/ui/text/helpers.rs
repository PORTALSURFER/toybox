
/// Measure monospaced bitmap text bounds at the given scale.
fn text_size(text: &str, scale: u32) -> Size {
    let scale = u64::from(scale.max(1));
    let mut max_cols = 0u64;
    let mut lines = 1u64;
    let mut current = 0u64;
    for ch in text.chars() {
        if ch == '\n' {
            max_cols = max_cols.max(current);
            current = 0;
            lines = lines.saturating_add(1);
        } else {
            current = current.saturating_add(1);
        }
    }
    max_cols = max_cols.max(current);
    let char_width = 6u64.saturating_mul(scale);
    let char_height = 8u64.saturating_mul(scale);
    Size {
        width: u32::try_from(max_cols.saturating_mul(char_width))
            .unwrap_or(u32::MAX),
        height: u32::try_from(lines.saturating_mul(char_height))
            .unwrap_or(u32::MAX),
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

    let char_width = 6u64.saturating_mul(scale.max(1).into());
    let max_chars = (u64::from(max_width) / char_width) as usize;
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

    let char_width = 6u64.saturating_mul(scale.max(1).into());
    let max_chars = (u64::from(max_width) / char_width) as usize;
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
    let raw = (i64::from(target_center_x) - i64::from(text_width) / 2).clamp(
        i64::from(i32::MIN),
        i64::from(i32::MAX),
    );
    let min_x = left_bound;
    let span = max_width.saturating_sub(text_width);
    let max_x = left_bound.saturating_add(span.try_into().unwrap_or(i32::MAX));
    raw.clamp(i64::from(min_x), i64::from(max_x)) as i32
}

#[cfg(test)]
mod text_helpers_tests {
    use super::*;

    #[test]
    fn text_size_saturates_for_extreme_scale() {
        let size = text_size("HELLO", u32::MAX);
        assert_eq!(size.width, u32::MAX);
    }

    #[test]
    fn centered_origin_handles_saturated_span() {
        assert_eq!(centered_text_origin_on_x(-10, u32::MAX, u32::MAX, 40), -10);
        assert_eq!(centered_text_origin_on_x(10, u32::MAX, u32::MAX, -100), 10);
    }
}
