use super::*;
use crate::canvas::Canvas;
use crate::host::InputState;

fn point_for_curve(point: crate::declarative::CurvePoint, rect: Rect) -> Point {
    let width = rect.size.width.max(1) as f32 - 1.0;
    let height = rect.size.height.max(1) as f32 - 1.0;
    Point {
        x: rect.origin.x + (point.x * width).round() as i32,
        y: rect.origin.y + ((1.0 - point.y) * height).round() as i32,
    }
}

fn offset_point(point: Point, dx: i32, dy: i32) -> Point {
    Point {
        x: point.x + dx,
        y: point.y + dy,
    }
}

fn snap_interaction() -> crate::declarative::CurveInteractionOptions {
    crate::declarative::CurveInteractionOptions {
        drag_start_threshold_px: 0,
        snap: crate::declarative::CurveSnapConfig {
            enabled: true,
            vertical_positions: vec![0.0, 0.25, 0.5, 0.75, 1.0],
            horizontal_positions: vec![0.0, 0.25, 0.5, 0.75, 1.0],
        },
        ..crate::declarative::CurveInteractionOptions::default()
    }
}

#[test]
fn curve_editor_inserted_points_snap_to_grid() {
    let mut canvas = Canvas::new(220, 160);
    let mut layout = Layout::default();
    let mut ui_state = UiState::default();
    let theme = Theme::default();
    let rect = Rect {
        origin: Point { x: 8, y: 10 },
        size: Size {
            width: 180,
            height: 100,
        },
    };
    let mut model = crate::declarative::CurveModel::new(
        vec![
            crate::declarative::CurvePoint::new(0.0, 1.0),
            crate::declarative::CurvePoint::new(1.0, 0.0),
        ],
        vec![crate::declarative::CurveSegment::new(0.0)],
    );
    let input = InputState {
        pointer_pos: point_for_curve(crate::declarative::CurvePoint::new(0.44, 0.56), rect),
        mouse_pressed: true,
        mouse_down: true,
        ..InputState::default()
    };

    let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
    let response = ui.curve_editor_in_rect(
        &mut model,
        CurveEditorRectRenderRequest::new(
            WidgetId::new(51),
            rect,
            crate::declarative::CurveEditorStyle::default(),
            crate::declarative::CurveGridConfig::default(),
            snap_interaction(),
            None,
        ),
    );

    assert!(response.changed);
    assert_eq!(model.points[1], crate::declarative::CurvePoint::new(0.5, 0.5));
}

#[test]
fn curve_editor_dragged_point_snaps_to_grid() {
    let mut canvas = Canvas::new(220, 160);
    let mut layout = Layout::default();
    let layout_origin = layout.cursor;
    let mut ui_state = UiState::default();
    let theme = Theme::default();
    let rect = Rect {
        origin: Point { x: 8, y: 10 },
        size: Size {
            width: 180,
            height: 100,
        },
    };
    let mut model = crate::declarative::CurveModel::new(
        vec![
            crate::declarative::CurvePoint::new(0.0, 1.0),
            crate::declarative::CurvePoint::new(0.33, 0.31),
            crate::declarative::CurvePoint::new(1.0, 1.0),
        ],
        vec![crate::declarative::CurveSegment::new(0.0); 2],
    );

    let press_input = InputState {
        pointer_pos: point_for_curve(model.points[1], rect),
        mouse_pressed: true,
        mouse_down: true,
        ..InputState::default()
    };
    {
        let mut ui = Ui::new(&mut canvas, &press_input, &mut ui_state, &mut layout, &theme);
        let _ = ui.curve_editor_in_rect(
            &mut model,
            CurveEditorRectRenderRequest::new(
                WidgetId::new(52),
                rect,
                crate::declarative::CurveEditorStyle::default(),
                crate::declarative::CurveGridConfig::default(),
                snap_interaction(),
                None,
            ),
        );
    }

    layout.cursor = layout_origin;
    let drag_input = InputState {
        pointer_pos: point_for_curve(crate::declarative::CurvePoint::new(0.41, 0.61), rect),
        mouse_down: true,
        ..InputState::default()
    };
    let mut ui = Ui::new(&mut canvas, &drag_input, &mut ui_state, &mut layout, &theme);
    let response = ui.curve_editor_in_rect(
        &mut model,
        CurveEditorRectRenderRequest::new(
            WidgetId::new(52),
            rect,
            crate::declarative::CurveEditorStyle::default(),
            crate::declarative::CurveGridConfig::default(),
            snap_interaction(),
            None,
        ),
    );

    assert!(response.changed);
    assert_eq!(model.points[1], crate::declarative::CurvePoint::new(0.5, 0.5));
}

#[test]
fn curve_editor_segment_translation_snaps_midpoint_to_grid() {
    let mut canvas = Canvas::new(240, 180);
    let mut layout = Layout::default();
    let layout_origin = layout.cursor;
    let mut ui_state = UiState::default();
    let theme = Theme::default();
    let rect = Rect {
        origin: Point { x: 8, y: 10 },
        size: Size {
            width: 180,
            height: 100,
        },
    };
    let mut model = crate::declarative::CurveModel::new(
        vec![
            crate::declarative::CurvePoint::new(0.0, 0.0),
            crate::declarative::CurvePoint::new(0.25, 0.25),
            crate::declarative::CurvePoint::new(0.5, 0.5),
            crate::declarative::CurvePoint::new(1.0, 1.0),
        ],
        vec![crate::declarative::CurveSegment::new(0.0); 3],
    );
    let midpoint = crate::declarative::CurvePoint::new(0.375, 0.375);

    let press_input = InputState {
        pointer_pos: offset_point(point_for_curve(midpoint, rect), 0, -4),
        mouse_pressed: true,
        mouse_down: true,
        ..InputState::default()
    };
    {
        let mut ui = Ui::new(&mut canvas, &press_input, &mut ui_state, &mut layout, &theme);
        let _ = ui.curve_editor_in_rect(
            &mut model,
            CurveEditorRectRenderRequest::new(
                WidgetId::new(53),
                rect,
                crate::declarative::CurveEditorStyle::default(),
                crate::declarative::CurveGridConfig::default(),
                snap_interaction(),
                None,
            ),
        );
    }

    layout.cursor = layout_origin;
    let drag_input = InputState {
        pointer_pos: point_for_curve(crate::declarative::CurvePoint::new(0.58, 0.62), rect),
        mouse_down: true,
        ..InputState::default()
    };
    let mut ui = Ui::new(&mut canvas, &drag_input, &mut ui_state, &mut layout, &theme);
    let response = ui.curve_editor_in_rect(
        &mut model,
        CurveEditorRectRenderRequest::new(
            WidgetId::new(53),
            rect,
            crate::declarative::CurveEditorStyle::default(),
            crate::declarative::CurveGridConfig::default(),
            snap_interaction(),
            None,
        ),
    );

    assert!(response.changed);
    assert_eq!(model.points[1], crate::declarative::CurvePoint::new(0.375, 0.375));
    assert_eq!(model.points[2], crate::declarative::CurvePoint::new(0.625, 0.625));
}
