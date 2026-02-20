use super::super::*;
use crate::canvas::Canvas;
use crate::host::InputState;
use crate::ui::{Layout, Theme, UiState};

#[test]
fn render_unlabeled_knob_uses_reduced_hit_region() {
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
    let theme = Theme::default();
    let changed_at = |pointer_pos: Point| {
        let mut canvas = Canvas::new(360, 220);
        let mut layout = Layout::default();
        let mut ui_state = UiState::default();
        let input = InputState {
            pointer_pos,
            wheel_delta: 1.0,
            ..InputState::default()
        };
        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let result =
            render_checked(&spec, &mut ui, Point { x: 0, y: 0 }).expect("render should succeed");
        result
            .actions
            .iter()
            .any(|action| matches!(action, UiAction::KnobChanged { key, .. } if key == "k"))
    };

    assert!(
        changed_at(Point { x: 80, y: 60 }),
        "wheel interaction should still apply inside reduced unlabeled hit-region"
    );
    assert!(
        !changed_at(Point { x: 100, y: 60 }),
        "wheel interaction should be rejected near the old oversized unlabeled edge"
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
fn knob_double_click_emits_changed_action_at_default_value() {
    let mut canvas = Canvas::new(220, 180);
    let mut layout = Layout::default();
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let input = InputState {
        pointer_pos: Point { x: 40, y: 60 },
        mouse_pressed: true,
        mouse_double_clicked: true,
        ..InputState::default()
    };
    let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);

    let spec = UiSpec::new(RootFrameSpec::new(
        "root",
        Node::Absolute(
            AbsoluteSpec::new(vec![AbsoluteChild::new(
                Point { x: 0, y: 0 },
                Node::Knob(KnobSpec::new("mix", 0.8, (0.0, 1.0)).default_value(0.3)),
            )])
            .layout(ContainerLayout::fill()),
        ),
    ));
    let result = render_checked(&spec, &mut ui, Point { x: 0, y: 0 }).expect("render should succeed");
    assert!(result.actions.iter().any(
        |action| matches!(
            action,
            UiAction::KnobChanged { key, value }
                if key == "mix" && (*value - 0.3).abs() <= f32::EPSILON
        )
    ));
}

#[test]
fn slider_double_click_emits_changed_action_at_default_value() {
    let mut canvas = Canvas::new(240, 120);
    let mut layout = Layout::default();
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let input = InputState {
        pointer_pos: Point { x: 24, y: 24 },
        mouse_pressed: true,
        mouse_double_clicked: true,
        ..InputState::default()
    };
    let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);

    let spec = UiSpec::new(RootFrameSpec::new(
        "root",
        panel(
            "panel",
            Node::Slider(
                SliderSpec::new("mix", 0.75, (0.0, 1.0))
                    .default_value(0.2)
                    .control_size(Size {
                        width: 120,
                        height: 16,
                    }),
            ),
        )
        .pad_all(0),
    ));
    let result = render_checked(&spec, &mut ui, Point { x: 0, y: 0 }).expect("render should succeed");
    assert!(result.actions.iter().any(
        |action| matches!(
            action,
            UiAction::SliderChanged { key, value }
                if key == "mix" && (*value - 0.2).abs() <= f32::EPSILON
        )
    ));
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

#[test]
fn editable_text_box_left_arrow_moves_cursor_for_next_insert() {
    let mut ui_state = UiState::default();
    let theme = Theme::default();

    let mut canvas = Canvas::new(200, 80);
    let mut layout = Layout::default();
    let input_move_left = InputState {
        key_pressed: Some('\u{1c}'),
        ..InputState::default()
    };
    let mut ui = Ui::new(
        &mut canvas,
        &input_move_left,
        &mut ui_state,
        &mut layout,
        &theme,
    );
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
    assert!(
        result.actions.is_empty(),
        "moving cursor should not emit edit actions by itself"
    );

    let mut canvas = Canvas::new(200, 80);
    let mut layout = Layout::default();
    let input_insert = InputState {
        key_pressed: Some('X'),
        ..InputState::default()
    };
    let mut ui = Ui::new(&mut canvas, &input_insert, &mut ui_state, &mut layout, &theme);
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
            UiAction::TextBoxEdited { key, text } if key == "preset-title" && text == "IniXt"
        )
    ));
}

#[test]
fn editable_text_box_shift_arrow_selects_text_and_replaces_it() {
    let mut ui_state = UiState::default();
    let theme = Theme::default();

    let mut canvas = Canvas::new(200, 80);
    let mut layout = Layout::default();
    let input_select_last = InputState {
        key_pressed: Some('\u{1c}'),
        shift_down: true,
        ..InputState::default()
    };
    let mut ui = Ui::new(
        &mut canvas,
        &input_select_last,
        &mut ui_state,
        &mut layout,
        &theme,
    );
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
    assert!(
        result.actions.is_empty(),
        "selection movement should not emit edit actions by itself"
    );

    let mut canvas = Canvas::new(200, 80);
    let mut layout = Layout::default();
    let input_replace = InputState {
        key_pressed: Some('Z'),
        ..InputState::default()
    };
    let mut ui = Ui::new(&mut canvas, &input_replace, &mut ui_state, &mut layout, &theme);
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
            UiAction::TextBoxEdited { key, text } if key == "preset-title" && text == "IniZ"
        )
    ));
}

#[test]
fn editable_text_box_click_places_caret_before_insert() {
    let mut ui_state = UiState::default();
    let theme = Theme::default();
    let size = Size {
        width: 120,
        height: 16,
    };
    let spec = || {
        UiSpec::new(
            root_frame_sized(
                "root",
                Node::Absolute(
                    AbsoluteSpec::new(vec![AbsoluteChild::new(
                        Point { x: 0, y: 0 },
                        textbox("Init")
                            .text_editable("preset-title", true)
                            .text_edit_max_chars(24),
                    )])
                    .layout(ContainerLayout::fill()),
                ),
                size,
            )
            .padding(0),
        )
    };

    let mut canvas = Canvas::new(size.width, size.height);
    let mut layout = Layout::default();
    let click_input = InputState {
        pointer_pos: Point { x: 26, y: 8 },
        mouse_pressed: true,
        mouse_down: true,
        ..InputState::default()
    };
    let mut ui = Ui::new(&mut canvas, &click_input, &mut ui_state, &mut layout, &theme);
    let result = render_checked(&spec(), &mut ui, Point { x: 0, y: 0 }).expect("render should succeed");
    assert!(
        result.actions.is_empty(),
        "clicking to place caret should not emit edit actions by itself"
    );

    let mut canvas = Canvas::new(size.width, size.height);
    let mut layout = Layout::default();
    let insert_input = InputState {
        key_pressed: Some('X'),
        ..InputState::default()
    };
    let mut ui = Ui::new(&mut canvas, &insert_input, &mut ui_state, &mut layout, &theme);
    let result = render_checked(&spec(), &mut ui, Point { x: 0, y: 0 }).expect("render should succeed");
    assert!(result.actions.iter().any(
        |action| matches!(
            action,
            UiAction::TextBoxEdited { key, text } if key == "preset-title" && text == "InXit"
        )
    ));
}

#[test]
fn editable_text_box_drag_select_replaces_selected_range() {
    let mut ui_state = UiState::default();
    let theme = Theme::default();
    let size = Size {
        width: 120,
        height: 16,
    };
    let spec = || {
        UiSpec::new(
            root_frame_sized(
                "root",
                Node::Absolute(
                    AbsoluteSpec::new(vec![AbsoluteChild::new(
                        Point { x: 0, y: 0 },
                        textbox("Init")
                            .text_editable("preset-title", true)
                            .text_edit_max_chars(24),
                    )])
                    .layout(ContainerLayout::fill()),
                ),
                size,
            )
            .padding(0),
        )
    };

    let mut canvas = Canvas::new(size.width, size.height);
    let mut layout = Layout::default();
    let press_input = InputState {
        pointer_pos: Point { x: 14, y: 8 },
        mouse_pressed: true,
        mouse_down: true,
        ..InputState::default()
    };
    let mut ui = Ui::new(&mut canvas, &press_input, &mut ui_state, &mut layout, &theme);
    let _ = render_checked(&spec(), &mut ui, Point { x: 0, y: 0 }).expect("press frame should render");

    let mut canvas = Canvas::new(size.width, size.height);
    let mut layout = Layout::default();
    let drag_input = InputState {
        pointer_pos: Point { x: 38, y: 8 },
        mouse_down: true,
        ..InputState::default()
    };
    let mut ui = Ui::new(&mut canvas, &drag_input, &mut ui_state, &mut layout, &theme);
    let _ = render_checked(&spec(), &mut ui, Point { x: 0, y: 0 }).expect("drag frame should render");

    let mut canvas = Canvas::new(size.width, size.height);
    let mut layout = Layout::default();
    let replace_input = InputState {
        key_pressed: Some('Z'),
        ..InputState::default()
    };
    let mut ui = Ui::new(&mut canvas, &replace_input, &mut ui_state, &mut layout, &theme);
    let result = render_checked(&spec(), &mut ui, Point { x: 0, y: 0 }).expect("replace frame should render");
    assert!(result.actions.iter().any(
        |action| matches!(
            action,
            UiAction::TextBoxEdited { key, text } if key == "preset-title" && text == "IZt"
        )
    ));
}

#[test]
fn editable_text_box_shift_click_extends_selection_from_existing_anchor() {
    let mut ui_state = UiState::default();
    let theme = Theme::default();
    let size = Size {
        width: 120,
        height: 16,
    };
    let spec = || {
        UiSpec::new(
            root_frame_sized(
                "root",
                Node::Absolute(
                    AbsoluteSpec::new(vec![AbsoluteChild::new(
                        Point { x: 0, y: 0 },
                        textbox("Init")
                            .text_editable("preset-title", true)
                            .text_edit_max_chars(24),
                    )])
                    .layout(ContainerLayout::fill()),
                ),
                size,
            )
            .padding(0),
        )
    };

    let mut canvas = Canvas::new(size.width, size.height);
    let mut layout = Layout::default();
    let shift_click_input = InputState {
        pointer_pos: Point { x: 14, y: 8 },
        mouse_pressed: true,
        mouse_down: true,
        shift_down: true,
        ..InputState::default()
    };
    let mut ui = Ui::new(
        &mut canvas,
        &shift_click_input,
        &mut ui_state,
        &mut layout,
        &theme,
    );
    let result = render_checked(&spec(), &mut ui, Point { x: 0, y: 0 }).expect("render should succeed");
    assert!(
        result.actions.is_empty(),
        "shift-click selection should not emit edit actions by itself"
    );

    let mut canvas = Canvas::new(size.width, size.height);
    let mut layout = Layout::default();
    let replace_input = InputState {
        key_pressed: Some('Z'),
        ..InputState::default()
    };
    let mut ui = Ui::new(&mut canvas, &replace_input, &mut ui_state, &mut layout, &theme);
    let result = render_checked(&spec(), &mut ui, Point { x: 0, y: 0 }).expect("render should succeed");
    assert!(result.actions.iter().any(
        |action| matches!(
            action,
            UiAction::TextBoxEdited { key, text } if key == "preset-title" && text == "IZ"
        )
    ));
}
