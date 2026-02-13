use super::super::*;
use crate::canvas::Canvas;
use crate::host::InputState;

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
        ui.slider(
            WidgetId::new(2),
            "GAIN",
            &mut value,
            SliderConfig {
                range: (0.0, 1.0),
                size: Size {
                    width: 100,
                    height: 16,
                },
            },
        );
    }

    input.mouse_pressed = false;
    input.pointer_pos = Point { x: 80, y: 40 };

    {
        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let response = ui.slider(
            WidgetId::new(2),
            "GAIN",
            &mut value,
            SliderConfig {
                range: (0.0, 1.0),
                size: Size {
                    width: 100,
                    height: 16,
                },
            },
        );
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
