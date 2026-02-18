
use crate::canvas::glyph_bitmap_for_text;

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

/// Estimate the rendered width for a text string using glyph ink bounds.
fn glyph_ink_span(text: &str, scale: u32) -> (u32, u32) {
    let scale = u64::from(scale.max(1));
    let mut first_glyph = true;
    let mut min_col = u64::MAX;
    let mut max_col = 0u64;

    for (i, ch) in text.chars().enumerate() {
        let glyph = glyph_bitmap_for_text(ch);

        let mut glyph_min = 5u8;
        let mut glyph_max = 0u8;
        for col in 0..5 {
            for row in glyph.iter() {
                if (row >> (4 - col)) & 1 == 1 {
                    glyph_min = glyph_min.min(col as u8);
                    glyph_max = glyph_max.max(col as u8);
                    break;
                }
            }
            if glyph_min == 0 && glyph_max == 4 {
                break;
            }
        }
        if glyph_min == 5 {
            continue;
        }

        let base = u64::try_from(i)
            .unwrap_or(u64::MAX)
            .saturating_mul(6);
        let glyph_left = base.saturating_add(u64::from(glyph_min));
        let glyph_right = base
            .saturating_add(u64::from(glyph_max.saturating_add(1)));
        if first_glyph {
            min_col = glyph_left;
            max_col = glyph_right;
            first_glyph = false;
        } else {
            min_col = min_col.min(glyph_left);
            max_col = max_col.max(glyph_right);
        }
    }

    if first_glyph {
        return (0, 0);
    }

    let width_cells = max_col.saturating_sub(min_col);
    let left_offset_cells = min_col;
    let width = width_cells.saturating_mul(scale);
    let left_offset = left_offset_cells.saturating_mul(scale);
    (
        u32::try_from(left_offset).unwrap_or(u32::MAX),
        u32::try_from(width).unwrap_or(u32::MAX),
    )
}

/// Return a text origin centered on a target x and clamped to bounds.
fn centered_text_origin_on_x(
    left_bound: i32,
    max_width: u32,
    text_width: u32,
    target_center_x: i32,
) -> i32 {
    centered_text_origin_on_span(left_bound, max_width, 0, text_width, target_center_x)
}

/// Return a text origin centered on a target x and clamped to bounds.
///
/// `span_left` is the visible-ink offset from the drawn string origin, and
/// `span_width` is the visible-ink span in source-space units.
fn centered_text_origin_on_span(
    left_bound: i32,
    max_width: u32,
    span_left: u32,
    span_width: u32,
    target_center_x: i32,
) -> i32 {
    let span_left = i64::from(span_left);
    let span_width = i64::from(span_width);
    let raw = (i64::from(target_center_x).saturating_sub(span_left)).saturating_sub(span_width / 2);
    let min_x = (i64::from(left_bound)).saturating_sub(span_left);
    let span_span = span_left.saturating_add(span_width);
    let max_x = if span_span >= i64::from(max_width) {
        i64::from(left_bound)
    } else {
        (i64::from(left_bound))
            .saturating_add(i64::from(max_width))
            .saturating_sub(span_span)
    };
    raw.clamp(min_x, max_x) as i32
}

#[cfg(test)]
mod centered_text_helpers_tests {
    use super::*;

    #[test]
    fn centered_text_origin_on_span_with_left_offset() {
        assert_eq!(centered_text_origin_on_span(10, 50, 1, 3, 35), 33);
        assert_eq!(centered_text_origin_on_span(10, 50, 0, 3, 35), 34);
        assert_eq!(centered_text_origin_on_span(10, 10, 1, 3, 35), 16);
    }
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
        assert_eq!(centered_text_origin_on_span(-10, u32::MAX, 0, u32::MAX, 40), -10);
        assert_eq!(centered_text_origin_on_span(10, u32::MAX, 0, u32::MAX, -100), 10);
    }

    #[test]
    fn character_cell_width_tracks_glyph_ink() {
        assert_eq!(glyph_ink_span(" ", 1).1, 0);
        assert_eq!(glyph_ink_span("I", 1).1, 3);
        assert_eq!(glyph_ink_span("I A", 1).1, 16);
        assert_eq!(glyph_ink_span("I", 2).1, 6);
    }

    #[test]
    fn character_cell_bounds_tracks_left_offset() {
        assert_eq!(glyph_ink_span("I", 1), (1, 3));
        assert_eq!(glyph_ink_span("A", 1), (0, 5));
        assert_eq!(glyph_ink_span("I A", 1), (1, 16));
    }
}
