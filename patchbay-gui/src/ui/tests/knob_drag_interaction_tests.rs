use super::super::*;
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
fn knob_stays_active_when_drag_pointer_leaves_knob_bounds() {
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
    input.pointer_pos = Point { x: -120, y: -140 };
    layout.cursor = layout_origin;

    {
        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let response = ui.knob(WidgetId::new(1), "GAIN", &mut value, (0.0, 1.0));
        assert!(response.active);
        assert!(response.changed);
        assert!(value > 0.5, "dragging continues even when pointer leaves bounds");
    }
}
