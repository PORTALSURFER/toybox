use super::*;
use crate::canvas::Canvas;
use crate::host::InputState;
use crate::vector::scene::VectorCommand;

fn is_subpixel(value: f32) -> bool {
    (value - value.round()).abs() > 1.0e-3
}

#[test]
fn curve_editor_vector_shapes_keep_subpixel_positions() {
    let mut canvas = Canvas::new(260, 180);
    let mut layout = Layout::default();
    let mut ui_state = UiState::default();
    let theme = Theme::default();
    let input = InputState::default();
    let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
    ui.set_vector_shapes_enabled(true);

    let mut model = crate::declarative::CurveModel::new(
        vec![
            crate::declarative::CurvePoint::new(0.0, 0.95),
            crate::declarative::CurvePoint::new(0.37, 0.23),
            crate::declarative::CurvePoint::new(0.74, 0.81),
            crate::declarative::CurvePoint::new(1.0, 0.9),
        ],
        vec![crate::declarative::CurveSegment::new(0.0); 3],
    );

    let rect = Rect {
        origin: Point { x: 7, y: 9 },
        size: Size {
            width: 197,
            height: 121,
        },
    };
    let request = CurveEditorRectRenderRequest::new(
        WidgetId::new(4404),
        rect,
        crate::declarative::CurveEditorStyle::default(),
        crate::declarative::CurveGridConfig::default(),
        crate::declarative::CurveInteractionOptions::default(),
        Some(0.333),
    );
    let response = ui.curve_editor_in_rect(&mut model, request);
    assert!(!response.changed, "render-only frame should not mutate model");

    let commands = ui.take_vector_commands();
    let polyline_has_subpixel = commands.iter().any(|command| {
        if let VectorCommand::Polyline(polyline) = command {
            return polyline
                .points
                .iter()
                .any(|point| is_subpixel(point.x) || is_subpixel(point.y));
        }
        false
    });
    assert!(
        polyline_has_subpixel,
        "curve polyline should preserve subpixel coordinates for smoother anti-aliased rendering"
    );

    let circle_has_subpixel_center = commands.iter().any(|command| match command {
        VectorCommand::CircleFill(circle) => {
            is_subpixel(circle.center.x) || is_subpixel(circle.center.y)
        }
        VectorCommand::CircleStroke(circle) => {
            is_subpixel(circle.center.x) || is_subpixel(circle.center.y)
        }
        _ => false,
    });
    assert!(
        circle_has_subpixel_center,
        "curve points and playhead circles should keep subpixel centers in vector mode"
    );
}
