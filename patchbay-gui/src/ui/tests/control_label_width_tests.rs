use super::super::*;
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
