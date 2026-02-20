    use super::*;
    use crate::canvas::Canvas;
    use crate::host::InputState;
    #[test]
    fn button_reports_click() {
        let mut canvas = Canvas::new(200, 200);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState {
            pointer_pos: Point { x: 20, y: 20 },
            mouse_pressed: true,
            ..InputState::default()
        };

        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let response = ui.button(WidgetId::new(4), "OK", 40, 16);
        assert!(response.clicked);
    }

    #[test]
    fn dropdown_selects_option() {
        let mut canvas = Canvas::new(200, 200);
        let mut layout = Layout::default();
        let layout_origin = layout.cursor;
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let mut input = InputState {
            pointer_pos: Point { x: 20, y: 40 },
            mouse_pressed: true,
            ..InputState::default()
        };
        let options = ["Off", "Mono", "Poly"];
        let mut selected = 0;
        {
            let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
            ui.dropdown(WidgetId::new(5), "Mode", &options, &mut selected, 80, 16);
        }

        input.mouse_pressed = true;
        input.pointer_pos = Point { x: 20, y: 70 };
        layout.cursor = layout_origin;
        {
            let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
            ui.reset_input_consumption();
            let response = ui.dropdown(WidgetId::new(5), "Mode", &options, &mut selected, 80, 16);
            assert!(response.changed);
            assert_eq!(selected, 1);
        }
    }

    #[test]
    fn dropdown_respects_consumed_mouse_press() {
        let mut canvas = Canvas::new(200, 200);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let options = ["One", "Two"];
        let mut selected = 0;

        ui_state.consume_mouse_pressed = true;
        let input = InputState {
            pointer_pos: Point { x: 20, y: 40 },
            mouse_pressed: true,
            ..InputState::default()
        };

        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let response = ui.dropdown(WidgetId::new(9), "Mode", &options, &mut selected, 80, 16);
        assert!(!response.open);
        assert!(ui.state.open_dropdown.is_none());
    }

    #[test]
    fn panel_auto_sizes_after_draw() {
        let mut canvas = Canvas::new(400, 200);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState::default();

        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let response = ui.panel_with_key(
            "panel",
            PanelStyle {
                title: Some("Panel"),
                ..PanelStyle::default()
            },
            None,
            |ui, _rect| {
                let mut value = 0.5;
                ui.knob_with_key("gain", "GAIN", &mut value, (0.0, 1.0));
            },
        );

        assert!(response.measured_size.width > 0);
        assert!(response.measured_size.height > 0);
    }

    #[test]
    fn panel_auto_size_advances_layout_by_measured_height() {
        let mut canvas = Canvas::new(400, 200);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState::default();

        let start = layout.cursor;
        let spacing = layout.spacing;

        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let response = ui.panel_with_key(
            "panel-advance",
            PanelStyle {
                title: Some("Panel"),
                ..PanelStyle::default()
            },
            None,
            |ui, _rect| {
                let mut value = 0.5;
                ui.knob_with_key("gain", "GAIN", &mut value, (0.0, 1.0));
            },
        );

        let expected_y = start.y + response.measured_size.height as i32 + spacing;
        assert_eq!(ui.layout.cursor.y, expected_y);
    }

    #[test]
    fn panel_clamps_explicit_size_to_content() {
        let mut canvas = Canvas::new(400, 200);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState::default();
        let explicit = Size {
            width: 1,
            height: 1,
        };

        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let response = ui.panel_with_key(
            "panel-explicit",
            PanelStyle {
                title: Some("Panel"),
                ..PanelStyle::default()
            },
            Some(explicit),
            |ui, _rect| {
                let mut value = 0.5;
                ui.knob_with_key("gain", "GAIN", &mut value, (0.0, 1.0));
            },
        );

        assert!(response.measured_size.width > explicit.width);
        assert!(response.measured_size.height > explicit.height);
    }

    #[test]
    fn grid_cell_positions_are_consistent() {
        let mut canvas = Canvas::new(200, 200);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState::default();

        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let origin = Point { x: 10, y: 20 };
        let spec = GridSpec {
            columns: 4,
            cell_size: Size {
                width: 10,
                height: 12,
            },
            gap: 2,
            rows: None,
        };
        let response = ui.grid_with_key("grid", spec, origin, |_ui, grid| {
            let rect = grid.cell_rect(5);
            assert_eq!(rect.origin.x, origin.x + (10 + 2));
            assert_eq!(rect.origin.y, origin.y + (12 + 2));
        });

        assert_eq!(response.rows, 2);
        assert_eq!(response.columns, 4);
    }

    #[test]
    fn grid_rows_are_tracked_when_addressing_cells_by_row_and_column() {
        let mut canvas = Canvas::new(240, 240);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState::default();

        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let origin = Point { x: 8, y: 10 };
        let spec = GridSpec {
            columns: 4,
            cell_size: Size {
                width: 10,
                height: 12,
            },
            gap: 2,
            rows: None,
        };
        let response = ui.grid_with_key("grid-rc", spec, origin, |_ui, grid| {
            let rect = grid.cell_rect_rc(2, 3);
            assert_eq!(rect.origin.x, origin.x + 3 * (10 + 2));
            assert_eq!(rect.origin.y, origin.y + 2 * (12 + 2));
        });

        assert_eq!(response.rows, 3);
        assert_eq!(response.columns, 4);
    }

    #[test]
    fn main_palette_matches_documented_values() {
        let palette = MainPalette::main();
        assert_eq!(palette.accent_focus, Color::rgb(255, 196, 64));
        assert_eq!(palette.syntax_emphasis, Color::rgb(64, 214, 255));
        assert_eq!(palette.identifiers, Color::rgb(128, 255, 128));
        assert_eq!(palette.literals, Color::rgb(255, 128, 128));
        assert_eq!(palette.text_primary, Color::rgb(255, 255, 255));
        assert_eq!(palette.text_muted, Color::rgb(156, 156, 156));
        assert_eq!(palette.ui_secondary, Color::rgb(92, 92, 92));
        assert_eq!(palette.background_primary, Color::rgb(24, 24, 24));
        assert_eq!(palette.background_secondary, Color::rgb(38, 38, 38));
    }

    #[test]
    fn default_theme_is_derived_from_main_palette() {
        let palette = MainPalette::main();
        let theme = Theme::default();
        assert_eq!(theme.background, palette.background_primary);
        assert_eq!(theme.text, palette.text_primary);
        assert_eq!(theme.knob_fill, palette.background_secondary);
        assert_eq!(theme.knob_outline, palette.ui_secondary);
        assert_eq!(theme.knob_active, palette.accent_focus);
        assert_eq!(theme.knob_hover, palette.syntax_emphasis);
        assert_eq!(theme.knob_indicator, palette.text_primary);
    }
