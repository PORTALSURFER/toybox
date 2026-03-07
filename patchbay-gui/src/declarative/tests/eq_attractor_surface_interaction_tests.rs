use super::super::*;
use crate::canvas::Canvas;
use crate::host::InputState;
use crate::ui::{Layout, Theme, Ui, UiState};
use crate::vector::scene::VectorCommand;

fn eq_test_model() -> EqAttractorSurfaceModel {
    EqAttractorSurfaceModel {
        attractors: vec![EqAttractor::new(7, 0.5, 0.5).selected(true)],
        warp: 0.5,
        pull_force: 1.0,
        depths: vec![0.8],
        cycles: vec![1.2],
        rates_hz: vec![0.4],
        reverse_global: false,
        freq_min_hz: 20.0,
        freq_max_hz: 20_000.0,
        eq_bands: 32,
        eq_depth_db: 12.0,
    }
}

fn eq_test_spec(model: EqAttractorSurfaceModel) -> UiSpec {
    let size = Size {
        width: 200,
        height: 120,
    };
    UiSpec::new(
        RootFrameSpec::new(
            "root",
            Node::Absolute(
                AbsoluteSpec::new(vec![AbsoluteChild::new(
                    Point { x: 0, y: 0 },
                    eq_attractor_surface("eq-surface", model, EqAttractorSurfaceStyle::default())
                        .widget_layout(
                            LayoutBox::fixed(size.width, size.height).max(size.width, size.height),
                        ),
                )])
                .layout(ContainerLayout::fill()),
            ),
        )
        .padding(0)
        .layout(LayoutBox::fixed(size.width, size.height)),
    )
}

fn render_actions(spec: &UiSpec, input: InputState, ui_state: &mut UiState) -> Vec<UiAction> {
    let mut canvas = Canvas::new(
        input.window_size.width.max(1),
        input.window_size.height.max(1),
    );
    let mut layout = Layout::default();
    let theme = Theme::default();
    let mut ui = Ui::new(&mut canvas, &input, ui_state, &mut layout, &theme);
    let result =
        render_checked(spec, &mut ui, Point { x: 0, y: 0 }).expect("render should succeed");
    result.actions
}

fn render_pixels_and_actions(spec: &UiSpec, input: InputState) -> (Vec<u8>, Vec<UiAction>) {
    let mut canvas = Canvas::new(
        input.window_size.width.max(1),
        input.window_size.height.max(1),
    );
    let mut layout = Layout::default();
    let mut ui_state = UiState::default();
    let theme = Theme::default();
    let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
    let result =
        render_checked(spec, &mut ui, Point { x: 0, y: 0 }).expect("render should succeed");
    (canvas.pixels().to_vec(), result.actions)
}

fn render_vector_commands(spec: &UiSpec, input: InputState) -> Vec<VectorCommand> {
    let mut canvas = Canvas::new(
        input.window_size.width.max(1),
        input.window_size.height.max(1),
    );
    let mut layout = Layout::default();
    let mut ui_state = UiState::default();
    let theme = Theme::default();
    let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
    ui.set_vector_shapes_enabled(true);
    let _ = render_checked(spec, &mut ui, Point { x: 0, y: 0 }).expect("render should succeed");
    ui.take_vector_commands()
}

#[test]
fn eq_surface_press_emits_select_action() {
    let spec = eq_test_spec(eq_test_model());
    let mut ui_state = UiState::default();
    let actions = render_actions(
        &spec,
        InputState {
            window_size: Size {
                width: 200,
                height: 120,
            },
            pointer_pos: Point { x: 100, y: 60 },
            mouse_pressed: true,
            ..InputState::default()
        },
        &mut ui_state,
    );

    assert!(
        actions.iter().any(|action| matches!(
            action,
            UiAction::EqAttractorSurfaceChanged {
                key,
                action: EqAttractorSurfaceAction::Select { id }
            } if key == "eq-surface" && *id == 7
        )),
        "pressing a handle should emit Select"
    );
}

#[test]
fn eq_surface_drag_emits_move_action_after_press() {
    let spec = eq_test_spec(eq_test_model());
    let mut ui_state = UiState::default();

    let _ = render_actions(
        &spec,
        InputState {
            window_size: Size {
                width: 200,
                height: 120,
            },
            pointer_pos: Point { x: 100, y: 60 },
            mouse_pressed: true,
            ..InputState::default()
        },
        &mut ui_state,
    );

    let drag_actions = render_actions(
        &spec,
        InputState {
            window_size: Size {
                width: 200,
                height: 120,
            },
            pointer_pos: Point { x: 140, y: 38 },
            mouse_down: true,
            ..InputState::default()
        },
        &mut ui_state,
    );

    assert!(
        drag_actions.iter().any(|action| matches!(
            action,
            UiAction::EqAttractorSurfaceChanged {
                key,
                action: EqAttractorSurfaceAction::Move { id, x, y }
            } if key == "eq-surface" && *id == 7 && *x > 0.65 && *y > 0.60
        )),
        "dragging after press should emit Move for active handle"
    );
}

#[test]
fn eq_surface_double_click_empty_emits_add_action() {
    let spec = eq_test_spec(eq_test_model());
    let mut ui_state = UiState::default();
    let actions = render_actions(
        &spec,
        InputState {
            window_size: Size {
                width: 200,
                height: 120,
            },
            pointer_pos: Point { x: 20, y: 20 },
            mouse_double_clicked: true,
            ..InputState::default()
        },
        &mut ui_state,
    );

    assert!(
        actions.iter().any(|action| matches!(
            action,
            UiAction::EqAttractorSurfaceChanged {
                key,
                action: EqAttractorSurfaceAction::Add { x, y }
            } if key == "eq-surface" && *x >= 0.0 && *x <= 1.0 && *y >= 0.0 && *y <= 1.0
        )),
        "double-clicking empty space should emit Add"
    );
}

#[test]
fn eq_surface_secondary_click_emits_remove_action() {
    let spec = eq_test_spec(eq_test_model());
    let mut ui_state = UiState::default();
    let actions = render_actions(
        &spec,
        InputState {
            window_size: Size {
                width: 200,
                height: 120,
            },
            pointer_pos: Point { x: 100, y: 60 },
            mouse_secondary_pressed: true,
            ..InputState::default()
        },
        &mut ui_state,
    );

    assert!(
        actions.iter().any(|action| matches!(
            action,
            UiAction::EqAttractorSurfaceChanged {
                key,
                action: EqAttractorSurfaceAction::Remove { id }
            } if key == "eq-surface" && *id == 7
        )),
        "secondary clicking a handle should emit Remove"
    );
}

#[test]
fn eq_surface_identical_inputs_are_deterministic() {
    let spec = eq_test_spec(eq_test_model());
    let input = InputState {
        window_size: Size {
            width: 200,
            height: 120,
        },
        ..InputState::default()
    };
    let (pixels_a, actions_a) = render_pixels_and_actions(&spec, input.clone());
    let (pixels_b, actions_b) = render_pixels_and_actions(&spec, input);

    assert_eq!(actions_a, actions_b);
    assert_eq!(pixels_a, pixels_b);
}

#[test]
fn eq_surface_nodes_render_after_curve_commands_in_vector_mode() {
    let spec = eq_test_spec(eq_test_model());
    let commands = render_vector_commands(
        &spec,
        InputState {
            window_size: Size {
                width: 200,
                height: 120,
            },
            ..InputState::default()
        },
    );

    let last_curve_like_command = commands
        .iter()
        .rposition(|command| matches!(command, VectorCommand::Line(_) | VectorCommand::Polyline(_)))
        .expect("expected line-based surface commands");
    let first_circle_command = commands
        .iter()
        .position(|command| {
            matches!(
                command,
                VectorCommand::CircleFill(_) | VectorCommand::CircleStroke(_)
            )
        })
        .expect("expected circle node commands");

    assert!(
        first_circle_command > last_curve_like_command,
        "attractor node circles should be appended after curve/grid vector commands"
    );
}
