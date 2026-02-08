    use super::*;
    use crate::canvas::Canvas;
    use crate::host::InputState;
    #[test]
    fn knob_updates_value_on_drag() {
        let mut canvas = Canvas::new(200, 200);
        let mut layout = Layout::default();
        let layout_origin = layout.cursor;
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let mut value = 0.5;
        let mut input = InputState {
            pointer_pos: Point { x: 40, y: 60 },
            mouse_pressed: true,
            mouse_down: true,
            ..InputState::default()
        };

        {
            let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
            ui.knob(WidgetId::new(1), "GAIN", &mut value, (0.0, 1.0));
        }

        input.mouse_pressed = false;
        input.pointer_pos = Point { x: 40, y: 20 };
        layout.cursor = layout_origin;

        {
            let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
            let response = ui.knob(WidgetId::new(1), "GAIN", &mut value, (0.0, 1.0));
            assert!(response.changed);
            assert!(value > 0.5, "dragging up should increase value");
        }
    }

    #[test]
    fn knob_with_key_allows_dynamic_labels() {
        let mut canvas = Canvas::new(200, 200);
        let mut layout = Layout::default();
        let layout_origin = layout.cursor;
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let mut value = 0.5;
        let mut input = InputState {
            pointer_pos: Point { x: 40, y: 60 },
            mouse_pressed: true,
            mouse_down: true,
            ..InputState::default()
        };

        {
            let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
            ui.knob_with_key("attack", "Attack 0.50s", &mut value, (0.0, 1.0));
        }

        input.mouse_pressed = false;
        input.pointer_pos = Point { x: 40, y: 20 };
        layout.cursor = layout_origin;

        {
            let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
            let response = ui.knob_with_key("attack", "Attack 0.60s", &mut value, (0.0, 1.0));
            assert!(response.changed);
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
    fn knob_labels_are_clamped_to_knob_width() {
        let mut canvas = Canvas::new(320, 240);
        let mut layout = Layout::default();
        let knob_diameter = layout.knob_size.max(1) as u32;
        let dial_square_width = knob_diameter + (KNOB_BLOCK_SIDE_PADDING.max(0) * 2) as u32;
        let label_stack_height = knob_diameter
            + knob_label_height(Theme::default().text_scale) * 2
            + knob_label_gap(Theme::default().text_scale) * 2;
        let expected_width = dial_square_width.max(label_stack_height);
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
                    (0.0, 1.0),
                    width,
                    height,
                );
            },
        );

        assert_eq!(response.measured_size.width, width as u32);
    }
