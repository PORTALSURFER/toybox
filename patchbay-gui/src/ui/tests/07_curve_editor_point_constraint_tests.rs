use super::*;
use crate::canvas::Canvas;
use crate::host::InputState;

const CURVE_ID: WidgetId = WidgetId::new(1173);

fn rect() -> Rect {
    Rect {
        origin: Point { x: 10, y: 12 },
        size: Size {
            width: 201,
            height: 121,
        },
    }
}

fn model() -> crate::declarative::CurveModel {
    crate::declarative::CurveModel::new(
        vec![
            crate::declarative::CurvePoint::new(0.0, 0.2),
            crate::declarative::CurvePoint::new(0.3, 0.4),
            crate::declarative::CurvePoint::new(0.6, 0.6),
            crate::declarative::CurvePoint::new(1.0, 0.2),
        ],
        vec![crate::declarative::CurveSegment::new(0.0); 3],
    )
}

fn point_for_curve(point: crate::declarative::CurvePoint) -> Point {
    let rect = rect();
    Point {
        x: rect.origin.x + (point.x * (rect.size.width - 1) as f32).round() as i32,
        y: rect.origin.y + ((1.0 - point.y) * (rect.size.height - 1) as f32).round() as i32,
    }
}

fn input(point: crate::declarative::CurvePoint, pressed: bool, shift: bool, command: bool) -> InputState {
    InputState {
        pointer_pos: point_for_curve(point),
        mouse_pressed: pressed,
        mouse_down: true,
        shift_down: shift,
        command_down: command,
        ..InputState::default()
    }
}

fn render_frame(
    model: &mut crate::declarative::CurveModel,
    ui_state: &mut UiState,
    input: InputState,
    interaction: crate::declarative::CurveInteractionOptions,
    constrained: bool,
) -> CurveEditorResponse {
    let mut canvas = Canvas::new(240, 170);
    let mut layout = Layout::default();
    let theme = Theme::default();
    let mut ui = Ui::new(&mut canvas, &input, ui_state, &mut layout, &theme);
    ui.reset_input_consumption();
    let request = CurveEditorRectRenderRequest::new(
        CURVE_ID,
        rect(),
        crate::declarative::CurveEditorStyle::default(),
        crate::declarative::CurveGridConfig::default(),
        interaction,
        None,
    );
    let request = if constrained {
        request.point_horizontal_constraint(crate::declarative::CurveEditorModifier::Shift)
    } else {
        request
    };
    ui.curve_editor_in_rect(model, request)
}

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 0.006,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn shift_from_press_locks_origin_y_while_opt_out_keeps_legacy_two_axis_drag() {
    let interaction = crate::declarative::CurveInteractionOptions {
        drag_start_threshold_px: 0,
        ..crate::declarative::CurveInteractionOptions::default()
    };
    let mut constrained_model = model();
    let mut constrained_state = UiState::default();
    let start = constrained_model.points[1];
    render_frame(
        &mut constrained_model,
        &mut constrained_state,
        input(start, true, true, false),
        interaction.clone(),
        true,
    );
    render_frame(
        &mut constrained_model,
        &mut constrained_state,
        input(crate::declarative::CurvePoint::new(0.45, 0.9), false, true, false),
        interaction.clone(),
        true,
    );
    assert_close(constrained_model.points[1].x, 0.45);
    assert_close(constrained_model.points[1].y, start.y);

    let mut legacy_model = model();
    let mut legacy_state = UiState::default();
    render_frame(
        &mut legacy_model,
        &mut legacy_state,
        input(start, true, true, false),
        interaction.clone(),
        false,
    );
    render_frame(
        &mut legacy_model,
        &mut legacy_state,
        input(crate::declarative::CurvePoint::new(0.45, 0.9), false, true, false),
        interaction,
        false,
    );
    assert_close(legacy_model.points[1].y, 0.9);
}

#[test]
fn mid_drag_shift_toggles_capture_visible_y_and_release_without_jump() {
    let interaction = crate::declarative::CurveInteractionOptions {
        drag_start_threshold_px: 0,
        ..crate::declarative::CurveInteractionOptions::default()
    };
    let mut model = model();
    let mut ui_state = UiState::default();
    let start = model.points[1];
    render_frame(
        &mut model,
        &mut ui_state,
        input(start, true, false, false),
        interaction.clone(),
        true,
    );
    render_frame(
        &mut model,
        &mut ui_state,
        input(crate::declarative::CurvePoint::new(0.38, 0.7), false, false, false),
        interaction.clone(),
        true,
    );
    let first_anchor = model.points[1].y;

    render_frame(
        &mut model,
        &mut ui_state,
        input(crate::declarative::CurvePoint::new(0.44, 0.1), false, true, false),
        interaction.clone(),
        true,
    );
    render_frame(
        &mut model,
        &mut ui_state,
        input(crate::declarative::CurvePoint::new(0.48, 0.95), false, true, false),
        interaction.clone(),
        true,
    );
    assert_close(model.points[1].y, first_anchor);

    render_frame(
        &mut model,
        &mut ui_state,
        input(crate::declarative::CurvePoint::new(0.48, 0.95), false, false, false),
        interaction.clone(),
        true,
    );
    assert_close(model.points[1].y, first_anchor);
    render_frame(
        &mut model,
        &mut ui_state,
        input(crate::declarative::CurvePoint::new(0.49, 0.85), false, false, false),
        interaction.clone(),
        true,
    );
    assert_close(model.points[1].y, first_anchor - 0.1);

    let second_anchor = model.points[1].y;
    render_frame(
        &mut model,
        &mut ui_state,
        input(crate::declarative::CurvePoint::new(0.5, 0.2), false, true, false),
        interaction.clone(),
        true,
    );
    render_frame(
        &mut model,
        &mut ui_state,
        input(crate::declarative::CurvePoint::new(0.51, 0.8), false, true, false),
        interaction.clone(),
        true,
    );
    assert_close(model.points[1].y, second_anchor);
    render_frame(
        &mut model,
        &mut ui_state,
        input(crate::declarative::CurvePoint::new(0.51, 0.8), false, false, false),
        interaction,
        true,
    );
    assert_close(model.points[1].y, second_anchor);
}

#[test]
fn shift_and_command_keep_y_anchored_while_x_uses_snap_targets() {
    let interaction = crate::declarative::CurveInteractionOptions {
        drag_start_threshold_px: 0,
        snap: crate::declarative::CurveSnapConfig {
            enabled: true,
            vertical_positions: vec![0.25, 0.5, 0.75],
            horizontal_positions: vec![0.0, 1.0],
        },
        ..crate::declarative::CurveInteractionOptions::default()
    };
    let mut model = model();
    let mut ui_state = UiState::default();
    let start = model.points[1];
    render_frame(
        &mut model,
        &mut ui_state,
        input(start, true, true, true),
        interaction.clone(),
        true,
    );
    render_frame(
        &mut model,
        &mut ui_state,
        input(crate::declarative::CurvePoint::new(0.52, 0.95), false, true, true),
        interaction,
        true,
    );
    assert_close(model.points[1].x, 0.5);
    assert_close(model.points[1].y, start.y);
}

#[test]
fn horizontal_constraint_preserves_neighbor_and_coupled_endpoint_rules() {
    let interaction = crate::declarative::CurveInteractionOptions {
        min_point_spacing_x: 0.05,
        drag_start_threshold_px: 0,
        push_through_threshold_px: 10_000,
        ..crate::declarative::CurveInteractionOptions::default()
    };
    let mut interior = model();
    let mut interior_state = UiState::default();
    let start = interior.points[1];
    render_frame(
        &mut interior,
        &mut interior_state,
        input(start, true, true, false),
        interaction.clone(),
        true,
    );
    render_frame(
        &mut interior,
        &mut interior_state,
        input(crate::declarative::CurvePoint::new(0.95, 0.95), false, true, false),
        interaction,
        true,
    );
    assert_close(interior.points[1].x, interior.points[2].x - 0.05);
    assert_close(interior.points[1].y, start.y);

    let coupled_interaction = crate::declarative::CurveInteractionOptions {
        drag_start_threshold_px: 0,
        endpoint_mode: crate::declarative::EndpointMode::CoupledY,
        ..crate::declarative::CurveInteractionOptions::default()
    };
    let mut coupled = model();
    let mut coupled_state = UiState::default();
    let endpoint = coupled.points[0];
    render_frame(
        &mut coupled,
        &mut coupled_state,
        input(endpoint, true, true, false),
        coupled_interaction.clone(),
        true,
    );
    render_frame(
        &mut coupled,
        &mut coupled_state,
        input(crate::declarative::CurvePoint::new(0.4, 0.9), false, true, false),
        coupled_interaction,
        true,
    );
    assert_close(coupled.points[0].x, 0.0);
    assert_close(coupled.points[0].y, endpoint.y);
    assert_close(coupled.points.last().unwrap().y, endpoint.y);
}

#[test]
fn sticky_drag_through_removal_restores_from_origin_with_anchor_intact() {
    let interaction = crate::declarative::CurveInteractionOptions {
        drag_start_threshold_px: 0,
        push_through_threshold_px: 2,
        ..crate::declarative::CurveInteractionOptions::default()
    };
    let mut model = crate::declarative::CurveModel::new(
        vec![
            crate::declarative::CurvePoint::new(0.0, 0.2),
            crate::declarative::CurvePoint::new(0.3, 0.4),
            crate::declarative::CurvePoint::new(0.5, 0.6),
            crate::declarative::CurvePoint::new(0.7, 0.8),
            crate::declarative::CurvePoint::new(1.0, 0.2),
        ],
        vec![crate::declarative::CurveSegment::new(0.0); 4],
    );
    let mut ui_state = UiState::default();
    let start = model.points[1];
    render_frame(
        &mut model,
        &mut ui_state,
        input(start, true, true, false),
        interaction.clone(),
        true,
    );
    render_frame(
        &mut model,
        &mut ui_state,
        input(crate::declarative::CurvePoint::new(0.65, 0.95), false, true, false),
        interaction.clone(),
        true,
    );
    assert_eq!(model.points.len(), 4);
    assert_close(model.points[1].y, start.y);

    render_frame(
        &mut model,
        &mut ui_state,
        input(crate::declarative::CurvePoint::new(0.35, 0.05), false, true, false),
        interaction,
        true,
    );
    assert_eq!(model.points.len(), 5);
    assert_close(model.points[1].x, 0.35);
    assert_close(model.points[1].y, start.y);
}

#[test]
fn release_focus_loss_and_consecutive_gestures_clear_constraint_runtime() {
    let interaction = crate::declarative::CurveInteractionOptions {
        drag_start_threshold_px: 0,
        ..crate::declarative::CurveInteractionOptions::default()
    };
    let mut model = model();
    let mut ui_state = UiState::default();
    let start = model.points[1];
    render_frame(
        &mut model,
        &mut ui_state,
        input(start, true, true, false),
        interaction.clone(),
        true,
    );
    render_frame(
        &mut model,
        &mut ui_state,
        input(crate::declarative::CurvePoint::new(0.4, 0.9), false, true, false),
        interaction.clone(),
        true,
    );
    let release_pointer = point_for_curve(model.points[1]);
    render_frame(
        &mut model,
        &mut ui_state,
        InputState {
            pointer_pos: release_pointer,
            mouse_released: true,
            ..InputState::default()
        },
        interaction.clone(),
        true,
    );
    assert!(
        ui_state
            .curve_editor_runtime
            .get(&CURVE_ID)
            .and_then(|runtime| runtime.drag_mode.as_ref())
            .is_none()
    );

    let next_start = model.points[1];
    render_frame(
        &mut model,
        &mut ui_state,
        input(next_start, true, false, false),
        interaction.clone(),
        true,
    );
    render_frame(
        &mut model,
        &mut ui_state,
        input(crate::declarative::CurvePoint::new(0.45, 0.25), false, false, false),
        interaction.clone(),
        true,
    );
    assert_close(model.points[1].y, 0.25);

    let focus_loss_pointer = point_for_curve(model.points[1]);
    render_frame(
        &mut model,
        &mut ui_state,
        InputState {
            pointer_pos: focus_loss_pointer,
            mouse_down: false,
            ..InputState::default()
        },
        interaction,
        true,
    );
    assert!(
        ui_state
            .curve_editor_runtime
            .get(&CURVE_ID)
            .and_then(|runtime| runtime.drag_mode.as_ref())
            .is_none()
    );
}
