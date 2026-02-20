use super::super::*;
use crate::canvas::Canvas;
use crate::host::InputState;
use crate::vector::scene::VectorCommand;

fn color_ink_bounds_in_y_band(
    canvas: &Canvas,
    color: Color,
    y_start: i32,
    y_end: i32,
) -> Option<(u32, u32)> {
    let size = canvas.size();
    if size.width == 0 || size.height == 0 {
        return None;
    }
    let start = y_start.max(0) as u32;
    let end = y_end.max(y_start).max(0) as u32;
    let start = start.min(size.height);
    let end = end.min(size.height);
    if start >= end {
        return None;
    }

    let mut min_x = u32::MAX;
    let mut max_x_exclusive = 0u32;
    let pixels = canvas.pixels();
    for y in start..end {
        for x in 0..size.width {
            let idx = ((y * size.width + x) * 4) as usize;
            let r = pixels[idx];
            let g = pixels[idx + 1];
            let b = pixels[idx + 2];
            let a = pixels[idx + 3];
            if a != 0 && r == color.r && g == color.g && b == color.b {
                min_x = min_x.min(x);
                max_x_exclusive = max_x_exclusive.max(x.saturating_add(1));
            }
        }
    }

    if min_x == u32::MAX || max_x_exclusive <= min_x {
        None
    } else {
        Some((min_x, max_x_exclusive))
    }
}

#[test]
fn knob_indicator_point_uses_arc_coordinate_convention() {
    let center = Point { x: 100, y: 100 };
    let radius = 20;

    let start = 7.0 * std::f32::consts::PI / 4.0;
    let top = std::f32::consts::PI / 2.0;

    let start_point = knob_indicator_point(center, radius, start);
    let top_point = knob_indicator_point(center, radius, top);

    assert!(start_point.x > center.x);
    assert!(start_point.y > center.y);
    assert!(top_point.y < center.y);
}

#[test]
fn knob_in_rect_does_not_expand_beyond_default_diameter() {
    let mut canvas = Canvas::new(260, 260);
    let mut layout = Layout::default();
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let mut value = 0.5;
    let input = InputState {
        pointer_pos: Point { x: 24, y: 150 },
        ..InputState::default()
    };

    let rect = Rect {
        origin: Point { x: 0, y: 0 },
        size: Size {
            width: 200,
            height: 220,
        },
    };
    let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
    let request = KnobRectRenderRequest::new(
        WidgetId::new(77),
        "GAIN",
        "50%",
        (0.0, 1.0),
        DEFAULT_KNOB_DIAMETER as u32,
        rect,
    );
    let response = ui.knob_with_labels_in_rect(&mut value, request);

    assert!(
        !response.hovered,
        "pointer should be below a default-sized knob, even in a tall rect"
    );
}

#[test]
fn knob_in_rect_without_labels_expands_dial_and_hit_area_only() {
    let desired_diameter = 72u32;
    let rect = Rect {
        origin: Point { x: 10, y: 20 },
        size: Size {
            width: 72,
            height: 54,
        },
    };
    let pointer = Point { x: 46, y: 66 };
    let mut value_with_labels = 0.5;
    let mut value_without_labels = 0.5;

    let mut canvas_with_labels = Canvas::new(200, 140);
    let mut layout_with_labels = Layout::default();
    let theme = Theme::default();
    let mut ui_state_with_labels = UiState::default();
    let input = InputState {
        pointer_pos: pointer,
        ..InputState::default()
    };
    let mut ui_with_labels = Ui::new(
        &mut canvas_with_labels,
        &input,
        &mut ui_state_with_labels,
        &mut layout_with_labels,
        &theme,
    );
    let with_labels_request =
        KnobRectRenderRequest::new(
            WidgetId::new(901),
            "MIX",
            "50%",
            (0.0, 1.0),
            desired_diameter,
            rect,
        )
            .with_text_scale(2);
    let with_labels_response =
        ui_with_labels.knob_with_labels_in_rect(&mut value_with_labels, with_labels_request);
    let with_labels_radius = ui_with_labels
        .take_vector_commands()
        .into_iter()
        .find_map(|command| match command {
            VectorCommand::Knob(knob) => Some(knob.radius),
            _ => None,
        })
        .expect("knob render should emit a vector knob command");

    let mut canvas_without_labels = Canvas::new(200, 140);
    let mut layout_without_labels = Layout::default();
    let mut ui_state_without_labels = UiState::default();
    let mut ui_without_labels = Ui::new(
        &mut canvas_without_labels,
        &input,
        &mut ui_state_without_labels,
        &mut layout_without_labels,
        &theme,
    );
    let without_labels_request =
        KnobRectRenderRequest::new(
            WidgetId::new(902),
            "",
            "",
            (0.0, 1.0),
            desired_diameter,
            rect,
        )
            .with_text_scale(2);
    let without_labels_response = ui_without_labels
        .knob_with_labels_in_rect(&mut value_without_labels, without_labels_request);
    let without_labels_radius = ui_without_labels
        .take_vector_commands()
        .into_iter()
        .find_map(|command| match command {
            VectorCommand::Knob(knob) => Some(knob.radius),
            _ => None,
        })
        .expect("knob render should emit a vector knob command");
    let side_padding = KNOB_BLOCK_SIDE_PADDING.max(0);
    let unclamped_unlabeled = (desired_diameter as i32)
        .min((rect.size.width as i32 - side_padding * 2).max(1))
        .min(rect.size.height as i32)
        .max(1);
    let expected_unlabeled_size = unclamped_unlabeled.saturating_mul(3).saturating_div(4).max(1);
    let expected_unlabeled_radius = (expected_unlabeled_size / 2 - 1).max(1);

    assert_eq!(
        without_labels_radius, expected_unlabeled_radius,
        "unlabeled knob radius should be reduced to 75% of unclamped rect fit"
    );

    assert!(
        without_labels_radius >= with_labels_radius.saturating_mul(3),
        "dial radius should grow when labels are omitted: with_labels={} without_labels={}",
        with_labels_radius,
        without_labels_radius
    );
    assert!(
        !with_labels_response.hovered,
        "pointer should miss the compact labeled hit-area"
    );
    assert!(
        without_labels_response.hovered,
        "pointer should hit the expanded unlabeled dial hit-area"
    );
}

#[test]
fn knob_labels_are_clamped_to_knob_width() {
    let mut canvas = Canvas::new(320, 240);
    let mut layout = Layout::default();
    let knob_diameter = layout.knob_size.max(1) as u32;
    let dial_square_width = knob_diameter + (KNOB_BLOCK_SIDE_PADDING.max(0) * 2) as u32;
    let expected_width = dial_square_width;
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let input = InputState::default();
    let mut value = 0.5;

    let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
    let response = ui.panel_with_key(
        "knob-clamp",
        PanelStyle {
            padding: 0,
            ..PanelStyle::default()
        },
        None,
        |ui, _| {
            let _ = ui.knob_with_key_labels(
                "knob",
                "PITCH DEPTHPITCH CURVE",
                "100.000000000 HZ",
                &mut value,
                (0.0, 1.0),
            );
        },
    );

    assert_eq!(response.measured_size.width, expected_width);
}

#[test]
fn knob_block_width_matches_dial_hit_width_for_tight_tiling() {
    let knob_diameter = DEFAULT_KNOB_DIAMETER.max(1) as u32;
    let block = knob_block_size_for_diameter(knob_diameter, Theme::default().text_scale);
    let dial_hit_width = knob_diameter + (KNOB_BLOCK_SIDE_PADDING.max(0) * 2) as u32;

    assert_eq!(block.width, dial_hit_width);
}

#[test]
fn hard_clamp_fitter_truncates_without_ellipsis() {
    let fitted = fit_text_single_line_hard_clamp("ABCDEFGHIJ", 24, 1);
    assert_eq!(fitted, "ABCD");
    assert!(!fitted.contains("..."));
}

#[test]
fn knob_name_labels_are_normalized_to_uppercase() {
    assert_eq!(normalize_knob_name_label("Mix dB"), "MIX DB");
    assert_eq!(normalize_knob_name_label("phase"), "PHASE");
}

#[test]
fn knob_value_labels_lowercase_only_when_textual() {
    assert_eq!(normalize_knob_value_label("+2.3 dB"), "+2.3 db");
    assert_eq!(normalize_knob_value_label("23dB"), "23db");
    assert_eq!(normalize_knob_value_label("42.0%"), "42.0%");
}

#[test]
fn centered_text_origin_on_axis_clamps_to_bounds() {
    assert_eq!(centered_text_origin_on_x(10, 40, 20, 30), 20);
    assert_eq!(centered_text_origin_on_x(10, 40, 20, 8), 10);
    assert_eq!(centered_text_origin_on_x(10, 40, 20, 80), 30);
}

#[test]
fn knob_label_ink_centers_match_knob_center_in_canvas_output() {
    let mut canvas = Canvas::new(220, 220);
    let mut layout = Layout::default();
    let theme = Theme {
        text_scale: 1,
        ..Theme::default()
    };
    let text_color = theme.text;
    let mut ui_state = UiState::default();
    let input = InputState::default();
    let mut value = 0.5;
    let knob_diameter = DEFAULT_KNOB_DIAMETER.max(1) as u32;
    let label_h = knob_label_height(theme.text_scale) as i32;
    let label_gap = knob_label_gap(theme.text_scale) as i32;
    let block_size = knob_block_size_for_diameter(knob_diameter, theme.text_scale);
    let origin = Point { x: 30, y: 24 };
    let rect = Rect {
        origin,
        size: block_size,
    };

    let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
    let request = KnobRectRenderRequest::new(
        WidgetId::new(991),
        "I",
        "+2.3 dB",
        (0.0, 1.0),
        knob_diameter,
        rect,
    );
    let _ = ui.knob_with_labels_in_rect(&mut value, request);

    let knob_diameter_i32 = knob_diameter as i32;
    let knob_x_offset = ((block_size.width as i32 - knob_diameter_i32) / 2).max(0);
    let knob_center_x = origin.x + knob_x_offset + knob_diameter_i32 / 2;
    let knob_origin_y = origin.y + label_h + label_gap;
    let value_label_y = knob_origin_y + knob_diameter_i32 + label_gap;

    let name_bounds =
        color_ink_bounds_in_y_band(&canvas, text_color, origin.y, origin.y + label_h)
            .expect("expected top knob name label ink pixels");
    let value_bounds = color_ink_bounds_in_y_band(
        &canvas,
        text_color,
        value_label_y,
        value_label_y + label_h,
    )
    .expect("expected bottom knob value label ink pixels");

    let target_twice = i64::from(knob_center_x).saturating_mul(2);
    let name_width = name_bounds.1.saturating_sub(name_bounds.0);
    let value_width = value_bounds.1.saturating_sub(value_bounds.0);
    let name_center_twice = i64::from(name_bounds.0)
        .saturating_mul(2)
        .saturating_add(i64::from(name_width));
    let value_center_twice = i64::from(value_bounds.0)
        .saturating_mul(2)
        .saturating_add(i64::from(value_width));
    let name_delta = (name_center_twice - target_twice).abs();
    let value_delta = (value_center_twice - target_twice).abs();

    assert!(
        name_delta <= 1,
        "name label center mismatch: knob_center_x={} ink_left={} ink_right_exclusive={} ink_center_x2={} target_x2={} delta={}",
        knob_center_x,
        name_bounds.0,
        name_bounds.1,
        name_center_twice,
        target_twice,
        name_delta
    );
    assert!(
        value_delta <= 1,
        "value label center mismatch: knob_center_x={} ink_left={} ink_right_exclusive={} ink_center_x2={} target_x2={} delta={}",
        knob_center_x,
        value_bounds.0,
        value_bounds.1,
        value_center_twice,
        target_twice,
        value_delta
    );
}

#[test]
fn vector_text_knob_labels_emit_centered_text_commands() {
    let mut canvas = Canvas::new(220, 220);
    let mut layout = Layout::default();
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let input = InputState::default();
    let mut value = 0.5;
    let rect = Rect {
        origin: Point { x: 30, y: 24 },
        size: knob_block_size_for_diameter(DEFAULT_KNOB_DIAMETER as u32, theme.text_scale),
    };

    let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
    ui.set_vector_text_enabled(true);
    let request = KnobRectRenderRequest::new(
        WidgetId::new(992),
        "I",
        "+2.3 dB",
        (0.0, 1.0),
        DEFAULT_KNOB_DIAMETER as u32,
        rect,
    );
    let _ = ui.knob_with_labels_in_rect(&mut value, request);
    let commands = ui.take_vector_commands();
    let centered_count = commands
        .iter()
        .filter(|command| matches!(command, VectorCommand::CenteredText { .. }))
        .count();

    assert_eq!(centered_count, 2);
}

#[test]
fn knob_in_rect_centers_visual_when_cell_is_oversized() {
    let mut canvas = Canvas::new(260, 220);
    let mut layout = Layout::default();
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let input = InputState::default();
    let mut value = 0.5;
    let block = knob_block_size_for_diameter(DEFAULT_KNOB_DIAMETER as u32, theme.text_scale);
    let rect = Rect {
        origin: Point { x: 20, y: 24 },
        size: Size {
            width: block.width + 40,
            height: block.height + 20,
        },
    };

    let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
    let request = KnobRectRenderRequest::new(
        WidgetId::new(993),
        "GAIN",
        "50%",
        (0.0, 1.0),
        DEFAULT_KNOB_DIAMETER as u32,
        rect,
    );
    let _ = ui.knob_with_labels_in_rect(&mut value, request);
    let commands = ui.take_vector_commands();
    let knob = commands
        .into_iter()
        .find_map(|command| match command {
            VectorCommand::Knob(knob) => Some(knob),
            _ => None,
        })
        .expect("knob render should emit a vector knob command");

    let target_center_x = rect.origin.x + rect.size.width as i32 / 2;
    let rect_bottom = rect.origin.y + rect.size.height as i32;
    let knob_bottom = knob.center.y + knob.radius;
    let knob_top = knob.center.y - knob.radius;

    assert_eq!(knob.center.x, target_center_x);
    assert!(
        knob_top >= rect.origin.y && knob_bottom <= rect_bottom,
        "knob body should remain within rect bounds: rect_y=[{}, {}] knob_y=[{}, {}]",
        rect.origin.y,
        rect_bottom,
        knob_top,
        knob_bottom
    );
}

#[test]
fn knob_in_rect_uses_half_minus_one_radius_regression() {
    let mut canvas = Canvas::new(240, 200);
    let mut layout = Layout::default();
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let input = InputState::default();
    let mut value = 0.5;
    let requested_diameter = 40u32;
    let rect = Rect {
        origin: Point { x: 20, y: 24 },
        size: knob_block_size_for_diameter(requested_diameter, theme.text_scale),
    };

    let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
    let request = KnobRectRenderRequest::new(
        WidgetId::new(994),
        "GAIN",
        "50%",
        (0.0, 1.0),
        requested_diameter,
        rect,
    );
    let _ = ui.knob_with_labels_in_rect(&mut value, request);
    let commands = ui.take_vector_commands();
    let knob = commands
        .into_iter()
        .find_map(|command| match command {
            VectorCommand::Knob(knob) => Some(knob),
            _ => None,
        })
        .expect("knob render should emit a vector knob command");

    let expected_radius = (requested_diameter as i32 / 2 - 1).max(1);
    assert_eq!(knob.radius, expected_radius);
}

#[test]
fn hard_clamped_text_respects_rect_height() {
    let mut canvas = Canvas::new(200, 120);
    let mut layout = Layout::default();
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let input = InputState::default();

    let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
    let clipped = ui.text_single_line_hard_clamped_in_rect(
        Rect {
            origin: Point { x: 0, y: 0 },
            size: Size {
                width: 120,
                height: 8,
            },
        },
        "BOUND",
        Color::rgb(255, 255, 255),
    );
    assert_eq!(clipped.width, 0);
    assert_eq!(clipped.height, 0);

    let visible = ui.text_single_line_hard_clamped_in_rect(
        Rect {
            origin: Point { x: 0, y: 20 },
            size: Size {
                width: 18,
                height: 16,
            },
        },
        "ABCDEFGHIJ",
        Color::rgb(255, 255, 255),
    );
    assert_eq!(visible.width, 12);
    assert_eq!(visible.height, 16);
}

#[test]
fn slider_labels_are_clamped_to_control_width() {
    let mut canvas = Canvas::new(320, 240);
    let mut layout = Layout::default();
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let input = InputState::default();
    let mut value = 0.5;
    let width = 90;
    let height = 18;

    let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
    let response = ui.panel_with_key(
        "slider-clamp",
        PanelStyle {
            padding: 0,
            ..PanelStyle::default()
        },
        None,
        |ui, _| {
            let _ = ui.slider(
                WidgetId::new(201),
                "VERY LONG SLIDER LABEL FOR DENSE LAYOUTS",
                &mut value,
                SliderConfig {
                    range: (0.0, 1.0),
                    size: Size {
                        width: width as u32,
                        height: height as u32,
                    },
                },
            );
        },
    );

    assert_eq!(response.measured_size.width, width as u32);
}
