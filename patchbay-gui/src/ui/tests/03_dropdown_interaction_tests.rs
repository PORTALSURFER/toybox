use super::*;
use crate::canvas::Canvas;
use crate::host::InputState;

#[test]
fn dropdown_opens_and_closes_on_primary_press() {
    let mut canvas = Canvas::new(200, 200);
    let mut layout = Layout::default();
    let layout_origin = layout.cursor;
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let options = ["Off", "Mono", "Poly"];
    let mut selected = 0;
    let id = WidgetId::new(17);

    let open_input = InputState {
        pointer_pos: Point { x: 20, y: 36 },
        mouse_pressed: true,
        ..InputState::default()
    };
    {
        let mut ui = Ui::new(&mut canvas, &open_input, &mut ui_state, &mut layout, &theme);
        let response = ui.dropdown(id, "Mode", &options, &mut selected, 80, 16);
        assert!(response.open);
        assert_eq!(ui.state.open_dropdown, Some(id));
    }

    layout.cursor = layout_origin;
    {
        let mut ui = Ui::new(&mut canvas, &open_input, &mut ui_state, &mut layout, &theme);
        ui.reset_input_consumption();
        let response = ui.dropdown(id, "Mode", &options, &mut selected, 80, 16);
        assert!(!response.open);
        assert_eq!(ui.state.open_dropdown, None);
    }
}

#[test]
fn dropdown_closes_on_outside_press_when_open() {
    let mut canvas = Canvas::new(200, 200);
    let mut layout = Layout::default();
    let layout_origin = layout.cursor;
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let options = ["Off", "Mono", "Poly"];
    let mut selected = 0;
    let id = WidgetId::new(18);

    let open_input = InputState {
        pointer_pos: Point { x: 20, y: 36 },
        mouse_pressed: true,
        ..InputState::default()
    };
    {
        let mut ui = Ui::new(&mut canvas, &open_input, &mut ui_state, &mut layout, &theme);
        let _ = ui.dropdown(id, "Mode", &options, &mut selected, 80, 16);
    }

    layout.cursor = layout_origin;
    let outside_input = InputState {
        pointer_pos: Point { x: 180, y: 180 },
        mouse_pressed: true,
        ..InputState::default()
    };
    {
        let mut ui = Ui::new(&mut canvas, &outside_input, &mut ui_state, &mut layout, &theme);
        ui.reset_input_consumption();
        let response = ui.dropdown(id, "Mode", &options, &mut selected, 80, 16);
        assert!(!response.open);
        assert!(!response.changed);
        assert_eq!(ui.state.open_dropdown, None);
    }
}

#[test]
fn dropdown_open_up_selection_uses_rows_above_control() {
    let mut canvas = Canvas::new(120, 80);
    let mut layout = Layout {
        cursor: Point { x: 16, y: 50 },
        ..Layout::default()
    };
    let layout_origin = layout.cursor;
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let options = ["Off", "Mono", "Poly"];
    let mut selected = 2;
    let id = WidgetId::new(19);

    let open_input = InputState {
        pointer_pos: Point { x: 20, y: 70 },
        mouse_pressed: true,
        ..InputState::default()
    };
    {
        let mut ui = Ui::new(&mut canvas, &open_input, &mut ui_state, &mut layout, &theme);
        let response = ui.dropdown(id, "Mode", &options, &mut selected, 80, 16);
        assert!(response.open);
    }

    layout.cursor = layout_origin;
    let select_first_above_input = InputState {
        pointer_pos: Point { x: 20, y: 58 },
        mouse_pressed: true,
        ..InputState::default()
    };
    {
        let mut ui = Ui::new(
            &mut canvas,
            &select_first_above_input,
            &mut ui_state,
            &mut layout,
            &theme,
        );
        ui.reset_input_consumption();
        let response = ui.dropdown(id, "Mode", &options, &mut selected, 80, 16);
        assert!(response.changed);
        assert!(!response.open);
        assert_eq!(selected, 0);
        assert_eq!(ui.state.open_dropdown, None);
    }
}

#[test]
fn dropdown_near_top_prefers_downward_open_direction() {
    let mut canvas = Canvas::new(120, 90);
    let mut layout = Layout {
        cursor: Point { x: 16, y: 4 },
        ..Layout::default()
    };
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let options = ["Off", "Mono", "Poly"];
    let mut selected = 0;
    let id = WidgetId::new(20);

    let open_input = InputState {
        pointer_pos: Point { x: 20, y: 24 },
        mouse_pressed: true,
        ..InputState::default()
    };
    let mut ui = Ui::new(&mut canvas, &open_input, &mut ui_state, &mut layout, &theme);
    let response = ui.dropdown(id, "Mode", &options, &mut selected, 80, 16);
    assert!(response.open);
    let overlay = ui
        .state
        .overlays
        .last()
        .expect("dropdown overlay should be queued");
    assert!(
        !overlay.open_up,
        "dropdown near top should open downward when there is more space below"
    );
}

#[test]
fn dropdown_clamped_menu_allows_wheel_scroll() {
    let mut canvas = Canvas::new(120, 70);
    let mut layout = Layout {
        cursor: Point { x: 16, y: 8 },
        ..Layout::default()
    };
    let layout_origin = layout.cursor;
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let options = ["0", "1", "2", "3", "4", "5"];
    let mut selected = 0;
    let id = WidgetId::new(21);

    let open_input = InputState {
        pointer_pos: Point { x: 20, y: 30 },
        mouse_pressed: true,
        ..InputState::default()
    };
    {
        let mut ui = Ui::new(&mut canvas, &open_input, &mut ui_state, &mut layout, &theme);
        let response = ui.dropdown(id, "Mode", &options, &mut selected, 80, 16);
        assert!(response.open);
    }

    layout.cursor = layout_origin;
    let scroll_input = InputState {
        pointer_pos: Point { x: 20, y: 50 },
        wheel_delta: -1.0,
        ..InputState::default()
    };
    {
        let mut ui = Ui::new(&mut canvas, &scroll_input, &mut ui_state, &mut layout, &theme);
        ui.reset_input_consumption();
        let response = ui.dropdown(id, "Mode", &options, &mut selected, 80, 16);
        assert!(response.open);
        assert!(
            ui.state.open_dropdown_scroll_px > 0,
            "wheel input over open menu should advance dropdown scroll"
        );
    }
}
