    use super::*;
    use crate::canvas::Canvas;
    use crate::host::InputState;
    #[test]
    fn toggle_labels_are_clamped_to_control_width() {
        let mut canvas = Canvas::new(320, 240);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState::default();
        let mut value = false;
        let width = 96;
        let height = 18;

        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let response = ui.panel_with_key(
            "toggle-clamp",
            PanelStyle {
                padding: 0,
                ..PanelStyle::default()
            },
            None,
            |ui, _| {
                let _ = ui.toggle(
                    WidgetId::new(202),
                    "VERY LONG TOGGLE LABEL FOR DENSE LAYOUTS",
                    &mut value,
                    width,
                    height,
                );
            },
        );

        assert_eq!(response.measured_size.width, width as u32);
    }

    #[test]
    fn dropdown_labels_are_clamped_to_control_width() {
        let mut canvas = Canvas::new(320, 240);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState::default();
        let options = ["One", "Two", "Three"];
        let mut selected = 0usize;
        let width = 92;
        let height = 18;

        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let response = ui.panel_with_key(
            "dropdown-clamp",
            PanelStyle {
                padding: 0,
                ..PanelStyle::default()
            },
            None,
            |ui, _| {
                let _ = ui.dropdown(
                    WidgetId::new(203),
                    "VERY LONG DROPDOWN LABEL FOR DENSE LAYOUTS",
                    &options,
                    &mut selected,
                    width,
                    height,
                );
            },
        );

        assert_eq!(response.measured_size.width, width as u32);
    }

    #[test]
    fn block_size_helpers_match_rendered_width_contracts() {
        let mut canvas = Canvas::new(200, 200);
        let mut layout = Layout::default();
        let knob_diameter = layout.knob_size.max(1) as u32;
        let dial_square_width = knob_diameter + (KNOB_BLOCK_SIDE_PADDING.max(0) * 2) as u32;
        let label_stack_height = knob_diameter
            + knob_label_height(Theme::default().text_scale) * 2
            + knob_label_gap(Theme::default().text_scale) * 2;
        let expected_knob_width = dial_square_width.max(label_stack_height);
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState::default();
        let ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);

        let knob = ui.knob_block_size("Pitch Depth", "100%");
        assert_eq!(knob.width, expected_knob_width);

        let slider = ui.slider_block_size(
            "Drive",
            Size {
                width: 84,
                height: 16,
            },
        );
        assert_eq!(slider.width, 84);

        let toggle = ui.toggle_block_size(
            "Enable",
            Size {
                width: 70,
                height: 18,
            },
        );
        assert_eq!(toggle.width, 70);

        let dropdown = ui.dropdown_block_size(
            "Mode",
            Size {
                width: 112,
                height: 18,
            },
        );
        assert_eq!(dropdown.width, 112);

        let button = ui.button_block_size(
            "Apply",
            Size {
                width: 88,
                height: 22,
            },
        );
        assert_eq!(button.width, 88);
        assert_eq!(button.height, 22);
    }

    #[test]
    fn slider_updates_value_on_drag() {
        let mut canvas = Canvas::new(200, 200);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let mut value = 0.0;
        let mut input = InputState {
            pointer_pos: Point { x: 20, y: 40 },
            mouse_pressed: true,
            mouse_down: true,
            ..InputState::default()
        };

        {
            let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
            ui.slider(WidgetId::new(2), "GAIN", &mut value, (0.0, 1.0), 100, 16);
        }

        input.mouse_pressed = false;
        input.pointer_pos = Point { x: 80, y: 40 };

        {
            let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
            let response = ui.slider(WidgetId::new(2), "GAIN", &mut value, (0.0, 1.0), 100, 16);
            assert!(response.changed);
        }
    }

    #[test]
    fn root_frame_measures_text_content() {
        let mut canvas = Canvas::new(200, 200);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState::default();

        {
            let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
            ui.root_frame_with_key(
                "root",
                RootFrameStyle {
                    padding: 0,
                    ..RootFrameStyle::default()
                },
                None,
                |ui, _rect| {
                    ui.text(Point { x: 0, y: 0 }, "Root");
                },
            );
        }

        let measured = ui_state
            .take_root_frame_size()
            .expect("root frame size missing");
        let expected = text_size("Root", theme.text_scale);
        assert_eq!(measured.width, expected.width);
        assert_eq!(measured.height, expected.height);
    }

    #[test]
    fn root_frame_respects_explicit_size() {
        let mut canvas = Canvas::new(200, 200);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState::default();
        let explicit = Size {
            width: 123,
            height: 77,
        };

        {
            let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
            ui.root_frame_with_key(
                "root",
                RootFrameStyle::default(),
                Some(explicit),
                |_ui, _| {},
            );
        }

        let measured = ui_state
            .take_root_frame_size()
            .expect("root frame size missing");
        assert_eq!(measured, explicit);
    }

    #[test]
    fn root_frame_preserves_explicit_size_even_when_content_is_larger() {
        let mut canvas = Canvas::new(200, 200);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState::default();
        let explicit = Size {
            width: 1,
            height: 1,
        };

        {
            let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
            ui.root_frame_with_key(
                "root",
                RootFrameStyle {
                    padding: 0,
                    ..RootFrameStyle::default()
                },
                Some(explicit),
                |ui, _| {
                    ui.text(Point { x: 0, y: 0 }, "Root");
                },
            );
        }

        let measured = ui_state
            .take_root_frame_size()
            .expect("root frame size missing");
        assert_eq!(measured, explicit);
    }

    #[test]
    fn toggle_flips_on_click() {
        let mut canvas = Canvas::new(200, 200);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let mut value = false;
        let input = InputState {
            pointer_pos: Point { x: 20, y: 40 },
            mouse_pressed: true,
            ..InputState::default()
        };

        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let response = ui.toggle(WidgetId::new(3), "Toggle", &mut value, 40, 16);
        assert!(response.changed);
        assert!(value);
    }
