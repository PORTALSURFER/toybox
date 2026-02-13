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
                    Node::Knob(KnobSpec::new("k", "Drive", 0.5, (0.0, 1.0))),
                )])
                .layout(LayoutBox::fixed(320, 200)),
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
fn knob_interaction_is_clamped_to_section_bounds() {
    let root_size = Size {
        width: 140,
        height: 100,
    };
    let content = column_sections(vec![
        weighted(
            panel("tight", knob("k", "K", 0.5, (0.0, 1.0))).pad_all(0),
            20,
        ),
        weighted(panel("rest", label("x")).pad_all(0), 80),
    ]);
    let spec = UiSpec::new(root_frame_sized("root", content, root_size, root_size).padding(0));

    let mut canvas = Canvas::new(root_size.width, root_size.height);
    let mut layout = Layout::default();
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let input = InputState {
        // Falls inside the knob's historical expanded hit-ring but below
        // the top section bounds.
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
        "knob outside section clip should not receive wheel interaction"
    );
}

#[test]
fn dropdown_overlay_interaction_is_clamped_to_section_bounds() {
    let root_size = Size {
        width: 160,
        height: 96,
    };
    let content = column_sections(vec![
        weighted(
            panel(
                "tight",
                dropdown(
                    "mode",
                    "",
                    vec!["A".to_string(), "B".to_string(), "C".to_string()],
                    0,
                ),
            )
            .pad_all(0),
            25,
        ),
        weighted(panel("rest", label("x")).pad_all(0), 75),
    ]);
    let spec = UiSpec::new(root_frame_sized("root", content, root_size, root_size).padding(0));

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

    // Frame 2: click where second option would be if overflow were allowed.
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
        !result.actions.iter().any(
            |action| matches!(action, UiAction::DropdownSelected { key, .. } if key == "mode")
        ),
        "dropdown option outside section clip should not be selectable"
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

    let button = ButtonSpec::new("ok", "OK").control_size(Size {
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
