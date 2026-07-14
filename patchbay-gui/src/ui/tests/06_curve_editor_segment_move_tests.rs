use super::*;
use crate::canvas::Canvas;
use crate::host::InputState;
use crate::vector::scene::VectorCommand;

const CURVE_ID: WidgetId = WidgetId::new(1169);

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
            crate::declarative::CurvePoint::new(0.0, 0.1),
            crate::declarative::CurvePoint::new(0.3, 0.3),
            crate::declarative::CurvePoint::new(0.6, 0.5),
            crate::declarative::CurvePoint::new(1.0, 0.8),
        ],
        vec![crate::declarative::CurveSegment::new(0.0); 3],
    )
}

fn command_segment_move_options() -> crate::declarative::CurveInteractionOptions {
    crate::declarative::CurveInteractionOptions {
        drag_start_threshold_px: 0,
        ..crate::declarative::CurveInteractionOptions::default()
    }
}

fn command_segment_move(
    highlight: Color,
) -> crate::declarative::CurveSegmentMoveOptions {
    crate::declarative::CurveSegmentMoveOptions::new(
        crate::declarative::CurveEditorModifier::Command,
        highlight,
    )
}

fn point_for_curve(point: crate::declarative::CurvePoint) -> Point {
    let rect = rect();
    Point {
        x: rect.origin.x + (point.x * (rect.size.width - 1) as f32).round() as i32,
        y: rect.origin.y + ((1.0 - point.y) * (rect.size.height - 1) as f32).round() as i32,
    }
}

fn offset(point: Point, dx: i32, dy: i32) -> Point {
    Point {
        x: point.x + dx,
        y: point.y + dy,
    }
}

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 1.0e-6,
        "expected {expected}, got {actual}"
    );
}

fn segment_midpoint(model: &crate::declarative::CurveModel, index: usize) -> Point {
    let left = model.points[index];
    let right = model.points[index + 1];
    point_for_curve(crate::declarative::CurvePoint::new(
        (left.x + right.x) * 0.5,
        (left.y + right.y) * 0.5,
    ))
}

fn render_frame(
    model: &mut crate::declarative::CurveModel,
    ui_state: &mut UiState,
    input: InputState,
    interaction: crate::declarative::CurveInteractionOptions,
    style: crate::declarative::CurveEditorStyle,
) -> (CurveEditorResponse, Vec<VectorCommand>) {
    render_frame_with_segment_move(
        model,
        ui_state,
        input,
        interaction,
        style,
        Some(crate::declarative::CurveSegmentMoveOptions::default()),
    )
}

fn render_frame_with_segment_move(
    model: &mut crate::declarative::CurveModel,
    ui_state: &mut UiState,
    input: InputState,
    interaction: crate::declarative::CurveInteractionOptions,
    style: crate::declarative::CurveEditorStyle,
    segment_move: Option<crate::declarative::CurveSegmentMoveOptions>,
) -> (CurveEditorResponse, Vec<VectorCommand>) {
    let mut canvas = Canvas::new(240, 170);
    let mut layout = Layout::default();
    let theme = Theme::default();
    let mut ui = Ui::new(&mut canvas, &input, ui_state, &mut layout, &theme);
    ui.reset_input_consumption();
    ui.set_vector_shapes_enabled(true);
    let request = CurveEditorRectRenderRequest::new(
            CURVE_ID,
            rect(),
            style,
            crate::declarative::CurveGridConfig::default(),
            interaction,
            None,
        );
    let request = if let Some(segment_move) = segment_move {
        request.segment_move(segment_move)
    } else {
        request
    };
    let response = ui.curve_editor_in_rect(model, request);
    (response, ui.take_vector_commands())
}

fn press_input(pointer_pos: Point, command_down: bool) -> InputState {
    InputState {
        pointer_pos,
        mouse_pressed: true,
        mouse_down: true,
        command_down,
        ..InputState::default()
    }
}

fn runtime_drag_mode(ui_state: &UiState) -> Option<&CurveEditorDragMode> {
    ui_state
        .curve_editor_runtime
        .get(&CURVE_ID)
        .and_then(|runtime| runtime.drag_mode.as_ref())
}

#[test]
fn command_hover_uses_dedicated_segment_move_color_and_suppresses_preview() {
    let mut model = model();
    let mut ui_state = UiState::default();
    let style = crate::declarative::CurveEditorStyle {
        line_highlight: Color::rgb(1, 2, 3),
        ..crate::declarative::CurveEditorStyle::default()
    };
    let preview_fill = style.preview_fill;
    let segment_move_highlight = Color::rgb(4, 5, 6);
    let input = InputState {
        pointer_pos: segment_midpoint(&model, 1),
        command_down: true,
        ..InputState::default()
    };

    let (response, commands) = render_frame_with_segment_move(
        &mut model,
        &mut ui_state,
        input,
        command_segment_move_options(),
        style,
        Some(command_segment_move(segment_move_highlight)),
    );

    assert!(!response.changed);
    assert!(commands.iter().any(
        |command| matches!(command, VectorCommand::Polyline(line) if line.color == segment_move_highlight)
    ));
    assert!(commands.iter().any(
        |command| matches!(command, VectorCommand::CircleFill(circle) if circle.color == segment_move_highlight)
    ));
    assert!(!commands.iter().any(
        |command| matches!(command, VectorCommand::CircleFill(circle) if circle.color == preview_fill)
    ));
}

#[test]
fn command_direct_line_point_and_empty_canvas_keep_their_required_precedence() {
    let interaction = command_segment_move_options();

    let mut direct_model = model();
    let direct_len = direct_model.points.len();
    let direct_pointer = segment_midpoint(&direct_model, 1);
    let mut direct_state = UiState::default();
    let (direct_response, _) = render_frame(
        &mut direct_model,
        &mut direct_state,
        press_input(direct_pointer, true),
        interaction.clone(),
        crate::declarative::CurveEditorStyle::default(),
    );
    assert!(!direct_response.changed);
    assert_eq!(direct_model.points.len(), direct_len);
    assert!(matches!(
        runtime_drag_mode(&direct_state),
        Some(CurveEditorDragMode::MoveSegment { index: 1, .. })
    ));

    let mut point_model = model();
    let point_pointer = point_for_curve(point_model.points[1]);
    let mut point_state = UiState::default();
    let (point_response, _) = render_frame(
        &mut point_model,
        &mut point_state,
        press_input(point_pointer, true),
        interaction.clone(),
        crate::declarative::CurveEditorStyle::default(),
    );
    assert!(!point_response.changed);
    assert!(matches!(
        runtime_drag_mode(&point_state),
        Some(CurveEditorDragMode::MovePoint {
            origin_index: 1,
            ..
        })
    ));

    let mut empty_model = model();
    let empty_len = empty_model.points.len();
    let mut empty_state = UiState::default();
    let (empty_response, _) = render_frame(
        &mut empty_model,
        &mut empty_state,
        press_input(
            point_for_curve(crate::declarative::CurvePoint::new(0.45, 0.95)),
            true,
        ),
        interaction,
        crate::declarative::CurveEditorStyle::default(),
    );
    assert!(empty_response.changed);
    assert_eq!(empty_model.points.len(), empty_len + 1);
    assert!(matches!(
        runtime_drag_mode(&empty_state),
        Some(CurveEditorDragMode::MovePoint { .. })
    ));
}

#[test]
fn unmodified_direct_line_keeps_insertion_and_default_keeps_legacy_segment_drag() {
    let mut gated_model = model();
    let pointer = segment_midpoint(&gated_model, 1);
    let original_len = gated_model.points.len();
    let mut gated_state = UiState::default();
    let (insert_response, _) = render_frame(
        &mut gated_model,
        &mut gated_state,
        press_input(pointer, false),
        command_segment_move_options(),
        crate::declarative::CurveEditorStyle::default(),
    );
    assert!(insert_response.changed);
    assert_eq!(gated_model.points.len(), original_len + 1);

    let mut legacy_model = model();
    let near_pointer = offset(segment_midpoint(&legacy_model, 1), 0, -5);
    let mut legacy_state = UiState::default();
    let (legacy_response, _) = render_frame_with_segment_move(
        &mut legacy_model,
        &mut legacy_state,
        press_input(near_pointer, false),
        crate::declarative::CurveInteractionOptions::default(),
        crate::declarative::CurveEditorStyle::default(),
        None,
    );
    assert!(!legacy_response.changed);
    assert!(matches!(
        runtime_drag_mode(&legacy_state),
        Some(CurveEditorDragMode::MoveSegment { index: 1, .. })
    ));
}

#[test]
fn command_drag_translates_one_pair_and_commit_cancel_and_next_gesture_clear_state() {
    let interaction = command_segment_move_options();
    let style = crate::declarative::CurveEditorStyle::default();
    let mut model = model();
    let mut ui_state = UiState::default();
    let start = segment_midpoint(&model, 1);
    let before = model.points.clone();

    let (press_response, _) = render_frame(
        &mut model,
        &mut ui_state,
        press_input(start, true),
        interaction.clone(),
        style.clone(),
    );
    assert!(!press_response.changed);

    let drag_input = InputState {
        pointer_pos: offset(start, 20, -12),
        mouse_down: true,
        command_down: true,
        ..InputState::default()
    };
    let (drag_response, _) = render_frame(
        &mut model,
        &mut ui_state,
        drag_input,
        interaction.clone(),
        style.clone(),
    );
    assert!(drag_response.changed, "one pair update reports one logical change");
    let left_delta = (
        model.points[1].x - before[1].x,
        model.points[1].y - before[1].y,
    );
    let right_delta = (
        model.points[2].x - before[2].x,
        model.points[2].y - before[2].y,
    );
    assert!((left_delta.0 - right_delta.0).abs() < 1.0e-6);
    assert!((left_delta.1 - right_delta.1).abs() < 1.0e-6);
    assert!(
        ((model.points[2].y - model.points[1].y) - (before[2].y - before[1].y)).abs()
            < 1.0e-6
    );

    let committed = model.clone();
    let release_input = InputState {
        pointer_pos: offset(start, 20, -12),
        mouse_released: true,
        ..InputState::default()
    };
    let _ = render_frame(
        &mut model,
        &mut ui_state,
        release_input,
        interaction.clone(),
        style.clone(),
    );
    assert_eq!(model, committed);
    assert!(runtime_drag_mode(&ui_state).is_none());

    let next_start = segment_midpoint(&model, 0);
    let _ = render_frame(
        &mut model,
        &mut ui_state,
        press_input(next_start, true),
        interaction.clone(),
        style.clone(),
    );
    assert!(matches!(
        runtime_drag_mode(&ui_state),
        Some(CurveEditorDragMode::MoveSegment { index: 0, .. })
    ));

    let focus_loss_input = InputState {
        pointer_pos: next_start,
        mouse_down: false,
        ..InputState::default()
    };
    let _ = render_frame(
        &mut model,
        &mut ui_state,
        focus_loss_input,
        interaction,
        style,
    );
    assert!(runtime_drag_mode(&ui_state).is_none());
}

#[test]
fn gated_segment_move_cancels_on_modifier_release_and_pointer_exit() {
    let interaction = command_segment_move_options();
    let style = crate::declarative::CurveEditorStyle::default();
    let move_color = Color::rgb(7, 8, 9);
    let preview_color = style.preview_fill;
    let mut model = model();
    let mut ui_state = UiState::default();
    let start = segment_midpoint(&model, 1);
    let _ = render_frame_with_segment_move(
        &mut model,
        &mut ui_state,
        press_input(start, true),
        interaction.clone(),
        style.clone(),
        Some(command_segment_move(move_color)),
    );

    let before_modifier_release = model.clone();
    let (modifier_release_response, modifier_released_commands) = render_frame_with_segment_move(
        &mut model,
        &mut ui_state,
        InputState {
            pointer_pos: offset(start, 20, -12),
            mouse_down: true,
            command_down: false,
            ..InputState::default()
        },
        interaction.clone(),
        style.clone(),
        Some(command_segment_move(move_color)),
    );
    assert!(!modifier_release_response.changed);
    assert_eq!(model, before_modifier_release);
    assert!(runtime_drag_mode(&ui_state).is_none());
    assert!(!modifier_released_commands.iter().any(|command| {
        matches!(command, VectorCommand::Polyline(line) if line.color == move_color)
            || matches!(command, VectorCommand::CircleFill(circle) if circle.color == move_color)
    }));
    assert!(!modifier_released_commands.iter().any(
        |command| matches!(command, VectorCommand::CircleFill(circle) if circle.color == preview_color)
    ));

    let _ = render_frame_with_segment_move(
        &mut model,
        &mut ui_state,
        InputState {
            pointer_pos: start,
            mouse_released: true,
            ..InputState::default()
        },
        interaction.clone(),
        style.clone(),
        Some(command_segment_move(move_color)),
    );
    let _ = render_frame_with_segment_move(
        &mut model,
        &mut ui_state,
        press_input(start, true),
        interaction.clone(),
        style.clone(),
        Some(command_segment_move(move_color)),
    );
    let before_pointer_exit = model.clone();
    let (pointer_exit_response, pointer_exit_commands) = render_frame_with_segment_move(
        &mut model,
        &mut ui_state,
        InputState {
            pointer_pos: Point { x: 400, y: 400 },
            pointer_in_window: false,
            mouse_down: true,
            command_down: true,
            ..InputState::default()
        },
        interaction,
        style,
        Some(command_segment_move(move_color)),
    );
    assert!(!pointer_exit_response.changed);
    assert_eq!(model, before_pointer_exit);
    assert!(runtime_drag_mode(&ui_state).is_none());
    assert!(!pointer_exit_commands.iter().any(|command| {
        matches!(command, VectorCommand::Polyline(line) if line.color == move_color)
            || matches!(command, VectorCommand::CircleFill(circle) if circle.color == move_color)
    }));
}

#[test]
fn group_clamping_preserves_pair_offsets_and_all_curve_constraints() {
    let snap = crate::declarative::CurveSnapConfig::default();
    let mut vertical = model();
    move_segment_translated(
        &mut vertical,
        1,
        (0.3, 0.3),
        (0.6, 0.5),
        (0.0, 1.0),
        0.05,
        crate::declarative::EndpointMode::Independent,
        &snap,
    );
    assert_close(vertical.points[1].y, 0.8);
    assert_close(vertical.points[2].y, 1.0);
    assert!((vertical.points[2].y - vertical.points[1].y - 0.2).abs() < 1.0e-6);
    move_segment_translated(
        &mut vertical,
        1,
        (0.3, 0.3),
        (0.6, 0.5),
        (0.0, -1.0),
        0.05,
        crate::declarative::EndpointMode::Independent,
        &snap,
    );
    assert_close(vertical.points[1].y, 0.0);
    assert_close(vertical.points[2].y, 0.2);

    let mut horizontal = crate::declarative::CurveModel::new(
        vec![
            crate::declarative::CurvePoint::new(0.0, 0.2),
            crate::declarative::CurvePoint::new(0.25, 0.3),
            crate::declarative::CurvePoint::new(0.5, 0.5),
            crate::declarative::CurvePoint::new(0.75, 0.7),
            crate::declarative::CurvePoint::new(1.0, 0.8),
        ],
        vec![crate::declarative::CurveSegment::new(0.0); 4],
    );
    move_segment_translated(
        &mut horizontal,
        1,
        (0.25, 0.3),
        (0.5, 0.5),
        (1.0, 0.0),
        0.1,
        crate::declarative::EndpointMode::Independent,
        &snap,
    );
    assert_close(horizontal.points[1].x, 0.4);
    assert_close(horizontal.points[2].x, 0.65);
    move_segment_translated(
        &mut horizontal,
        1,
        (0.25, 0.3),
        (0.5, 0.5),
        (-1.0, 0.0),
        0.1,
        crate::declarative::EndpointMode::Independent,
        &snap,
    );
    assert_close(horizontal.points[1].x, 0.1);
    assert_close(horizontal.points[2].x, 0.35);

    let mut endpoint = model();
    move_segment_translated(
        &mut endpoint,
        0,
        (0.0, 0.1),
        (0.3, 0.3),
        (0.5, 0.2),
        0.05,
        crate::declarative::EndpointMode::Independent,
        &snap,
    );
    assert_close(endpoint.points[0].x, 0.0);
    assert_close(endpoint.points[0].y, 0.3);
    assert_close(endpoint.points[1].x, 0.3);
    assert_close(endpoint.points[1].y, 0.5);

    let mut coupled = model();
    move_segment_translated(
        &mut coupled,
        0,
        (0.0, 0.1),
        (0.3, 0.3),
        (0.4, 0.2),
        0.05,
        crate::declarative::EndpointMode::CoupledY,
        &snap,
    );
    assert_close(coupled.points[0].y, 0.3);
    assert_close(coupled.points[1].y, 0.5);
    assert_close(coupled.points.last().map_or(f32::NAN, |point| point.y), 0.3);
}
