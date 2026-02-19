use super::super::*;
use crate::canvas::Canvas;
use crate::host::InputState;
use crate::ui::{Layout, Theme, UiState};

#[test]
fn render_knob_uses_token_diameter_for_hit_region() {
    let mut canvas = Canvas::new(360, 220);
    let mut layout = Layout::default();
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let input = InputState {
        pointer_pos: Point { x: 100, y: 60 },
        wheel_delta: 1.0,
        ..InputState::default()
    };
    let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);

    let mut tokens = ThemeTokens::default();
    tokens.controls.knob_diameter = 96;
    let spec = UiSpec::new(
        RootFrameSpec::new(
            "root",
            Node::Absolute(
                AbsoluteSpec::new(vec![AbsoluteChild::new(
                    Point { x: 0, y: 0 },
                    Node::Knob(KnobSpec::new("k", 0.5, (0.0, 1.0))),
                )])
                .layout(ContainerLayout::fill()),
            ),
        )
        .tokens(tokens)
        .padding(0)
        .layout(LayoutBox::fixed(320, 200)),
    );

    let result = render_checked(&spec, &mut ui, Point { x: 0, y: 0 }).expect("render should succeed");
    assert!(
        result
            .actions
            .iter()
            .any(|action| matches!(action, UiAction::KnobChanged { key, .. } if key == "k"))
    );
}

#[test]
fn knob_interaction_is_clamped_to_slot_bounds() {
    let root_size = Size {
        width: 140,
        height: 100,
    };
    let content = column_slots(vec![
        weighted_slot(
            panel("tight", knob("k", 0.5, (0.0, 1.0))).pad_all(0),
            20,
        ),
        weighted_slot(panel("rest", textbox("x")).pad_all(0), 80),
    ]);
    let spec = UiSpec::new(root_frame_sized("root", content, root_size).padding(0));

    let mut canvas = Canvas::new(root_size.width, root_size.height);
    let mut layout = Layout::default();
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let input = InputState {
        // Falls inside the knob's historical expanded hit-ring but below
        // the top slot bounds.
        pointer_pos: Point { x: 16, y: 25 },
        wheel_delta: 1.0,
        ..InputState::default()
    };
    let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
    let result = render_checked(&spec, &mut ui, Point { x: 0, y: 0 }).expect("render should succeed");
    assert!(
        !result
            .actions
            .iter()
            .any(|action| matches!(action, UiAction::KnobChanged { key, .. } if key == "k")),
        "knob outside slot clip should not receive wheel interaction"
    );
}

#[test]
fn dropdown_overlay_interaction_can_escape_slot_bounds() {
    let root_size = Size {
        width: 160,
        height: 96,
    };
    let content = column_slots(vec![
        weighted_slot(
            panel(
                "tight",
                dropdown(
                    "mode",
                    3,
                    0,
                ),
            )
            .pad_all(0),
            25,
        ),
        weighted_slot(panel("rest", textbox("x")).pad_all(0), 75),
    ]);
    let spec = UiSpec::new(root_frame_sized("root", content, root_size).padding(0));

    let mut ui_state = UiState::default();
    let theme = Theme::default();

    // Frame 1: open dropdown.
    let mut canvas = Canvas::new(root_size.width, root_size.height);
    let mut layout = Layout::default();
    let input_open = InputState {
        pointer_pos: Point { x: 8, y: 10 },
        mouse_pressed: true,
        ..InputState::default()
    };
    {
        let mut ui = Ui::new(&mut canvas, &input_open, &mut ui_state, &mut layout, &theme);
        let _ = render_checked(&spec, &mut ui, Point { x: 0, y: 0 })
            .expect("open frame should render");
    }

    // Frame 2: click on the second option below the slot bounds.
    let mut canvas = Canvas::new(root_size.width, root_size.height);
    let mut layout = Layout::default();
    let input_select = InputState {
        pointer_pos: Point { x: 8, y: 58 },
        mouse_pressed: true,
        ..InputState::default()
    };
    let mut ui = Ui::new(
        &mut canvas,
        &input_select,
        &mut ui_state,
        &mut layout,
        &theme,
    );
    ui.reset_input_consumption();
    let result = render_checked(&spec, &mut ui, Point { x: 0, y: 0 })
        .expect("selection frame should render");
    assert!(
        result.actions.iter().any(
            |action| matches!(
                action,
                UiAction::DropdownSelected { key, index } if key == "mode" && *index == 1
            )
        ),
        "dropdown option outside slot clip should remain selectable via popup overlay"
    );
}

#[test]
fn render_emits_button_action() {
    let mut canvas = Canvas::new(200, 120);
    let mut layout = Layout::default();
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let input = InputState {
        pointer_pos: Point { x: 24, y: 24 },
        mouse_pressed: true,
        ..InputState::default()
    };
    let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);

    let button = ButtonSpec::new("ok").control_size(Size {
        width: 80,
        height: 24,
    });
    let spec = UiSpec::new(RootFrameSpec::new(
        "root",
        Node::Panel(PanelSpec::new("panel", Node::Button(button))),
    ));

    let result = render_checked(&spec, &mut ui, Point { x: 0, y: 0 }).expect("render should succeed");
    assert!(
        result
            .actions
            .iter()
            .any(|action| matches!(action, UiAction::ButtonPressed { key } if key == "ok"))
    );
}

#[test]
fn dropdown_emits_double_click_action() {
    let mut canvas = Canvas::new(200, 120);
    let mut layout = Layout::default();
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let input = InputState {
        pointer_pos: Point { x: 24, y: 18 },
        mouse_double_clicked: true,
        ..InputState::default()
    };
    let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);

    let spec = UiSpec::new(RootFrameSpec::new(
        "root",
        panel("panel", dropdown("mode", 3, 0)).pad_all(0),
    ));
    let result = render_checked(&spec, &mut ui, Point { x: 0, y: 0 }).expect("render should succeed");
    assert!(
        result
            .actions
            .iter()
            .any(|action| matches!(action, UiAction::DropdownDoubleClicked { key } if key == "mode"))
    );
}

#[test]
fn editable_text_box_emits_edit_and_commit_actions() {
    let mut canvas = Canvas::new(200, 80);
    let mut layout = Layout::default();
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let input = InputState {
        key_pressed: Some('A'),
        ..InputState::default()
    };
    let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);

    let spec = UiSpec::new(RootFrameSpec::new(
        "root",
        panel(
            "panel",
            textbox("Init")
                .text_editable("preset-title", true)
                .text_edit_max_chars(24),
        )
        .pad_all(0),
    ));
    let result = render_checked(&spec, &mut ui, Point { x: 0, y: 0 }).expect("render should succeed");
    assert!(result.actions.iter().any(
        |action| matches!(
            action,
            UiAction::TextBoxEdited { key, text } if key == "preset-title" && text == "InitA"
        )
    ));

    let mut canvas = Canvas::new(200, 80);
    let mut layout = Layout::default();
    let input_commit = InputState {
        key_pressed: Some('\r'),
        ..InputState::default()
    };
    let mut ui = Ui::new(
        &mut canvas,
        &input_commit,
        &mut ui_state,
        &mut layout,
        &theme,
    );
    let spec = UiSpec::new(RootFrameSpec::new(
        "root",
        panel(
            "panel",
            textbox("InitA")
                .text_editable("preset-title", true)
                .text_edit_max_chars(24),
        )
        .pad_all(0),
    ));
    let result = render_checked(&spec, &mut ui, Point { x: 0, y: 0 }).expect("render should succeed");
    assert!(result.actions.iter().any(
        |action| matches!(
            action,
            UiAction::TextBoxEditCommitted { key, text }
                if key == "preset-title" && text == "InitA"
        )
    ));
}
