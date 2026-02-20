/// Return a textbox content rectangle inset from its outer bounds.
fn inset_text_box_rect(rect: Rect, text_scale: u32) -> Rect {
    const TEXTBOX_INSET_PX: i32 = 2;
    const TEXT_LINE_HEIGHT_BASE_PX: u32 = 8;
    let inset = TEXTBOX_INSET_PX.max(0);
    let max_horizontal = (rect.size.width / 2) as i32;
    let max_vertical = (rect.size.height / 2) as i32;
    let x_inset = inset.min(max_horizontal);
    let line_height = TEXT_LINE_HEIGHT_BASE_PX.saturating_mul(text_scale.max(1));
    let max_vertical_with_text = rect
        .size
        .height
        .saturating_sub(line_height)
        .saturating_div(2) as i32;
    let y_inset = inset.min(max_vertical).min(max_vertical_with_text.max(0));
    Rect {
        origin: Point {
            x: rect.origin.x + x_inset,
            y: rect.origin.y + y_inset,
        },
        size: Size {
            width: rect
                .size
                .width
                .saturating_sub((x_inset as u32).saturating_mul(2)),
            height: rect
                .size
                .height
                .saturating_sub((y_inset as u32).saturating_mul(2)),
        },
    }
}

/// Return the single-line draw rectangle centered vertically in `rect`.
///
/// The output keeps the input width/origin-x so horizontal clipping and
/// selection math remain stable. Height clamps to one text line at
/// `text_scale`.
fn resolve_text_line_rect(rect: Rect, text_scale: u32) -> Rect {
    const TEXT_LINE_HEIGHT_BASE_PX: u32 = 8;
    let line_height = TEXT_LINE_HEIGHT_BASE_PX
        .saturating_mul(text_scale.max(1))
        .max(1);
    if rect.size.height <= line_height {
        return Rect {
            origin: rect.origin,
            size: Size {
                width: rect.size.width,
                height: rect.size.height.max(1),
            },
        };
    }
    let y_offset = rect.size.height.saturating_sub(line_height) / 2;
    Rect {
        origin: Point {
            x: rect.origin.x,
            y: rect
                .origin
                .y
                .saturating_add(i32::try_from(y_offset).unwrap_or(i32::MAX)),
        },
        size: Size {
            width: rect.size.width,
            height: line_height,
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

    #[test]
    fn resolve_text_line_rect_centers_single_line_vertically() {
        let line_rect = resolve_text_line_rect(
            Rect {
                origin: Point { x: 10, y: 20 },
                size: Size {
                    width: 40,
                    height: 12,
                },
            },
            1,
        );
        assert_eq!(line_rect.origin.x, 10);
        assert_eq!(line_rect.origin.y, 22);
        assert_eq!(line_rect.size.width, 40);
        assert_eq!(line_rect.size.height, 8);
    }

    #[test]
    fn resolve_text_line_rect_keeps_origin_when_height_matches_line() {
        let line_rect = resolve_text_line_rect(
            Rect {
                origin: Point { x: 4, y: 6 },
                size: Size {
                    width: 32,
                    height: 8,
                },
            },
            1,
        );
        assert_eq!(line_rect.origin.x, 4);
        assert_eq!(line_rect.origin.y, 6);
        assert_eq!(line_rect.size.width, 32);
        assert_eq!(line_rect.size.height, 8);
    }
}
