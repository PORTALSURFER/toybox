use super::super::*;
use crate::InputState;

fn scene_frame_spec() -> UiSpec {
    UiSpec::new(
        root_frame_sized(
            "root",
            region(
                "content",
                Size {
                    width: 200,
                    height: 100,
                },
            ),
            Size {
                width: 200,
                height: 100,
            },
        )
        .padding(0),
    )
}

#[test]
fn plan_scene_frame_remaps_second_pass_input_from_surface_to_design_space() {
    let surface_input = InputState {
        window_size: Size {
            width: 100,
            height: 100,
        },
        pointer_pos: Point { x: 50, y: 50 },
        pointer_in_window: true,
        ..InputState::default()
    };
    let mut seen_inputs = Vec::new();

    let frame = plan_scene_frame(&surface_input, |input: &InputState| {
        seen_inputs.push(input.clone());
        scene_frame_spec()
    });

    assert_eq!(seen_inputs.len(), 2);
    assert_eq!(seen_inputs[0].window_size, surface_input.window_size);
    assert_eq!(
        seen_inputs[1].window_size,
        Size {
            width: 200,
            height: 100,
        }
    );
    assert_eq!(seen_inputs[1].pointer_pos, Point { x: 100, y: 50 });
    assert_eq!(frame.frame_input.window_size, seen_inputs[1].window_size);
    assert_eq!(frame.frame_input.pointer_pos, seen_inputs[1].pointer_pos);
}

#[test]
fn plan_scene_frame_preserves_unclamped_drag_pointer_mapping() {
    let surface_input = InputState {
        window_size: Size {
            width: 100,
            height: 100,
        },
        pointer_pos: Point { x: -10, y: 10 },
        pointer_in_window: false,
        mouse_down: true,
        ..InputState::default()
    };
    let mut seen_inputs = Vec::new();

    let frame = plan_scene_frame(&surface_input, |input: &InputState| {
        seen_inputs.push(input.clone());
        scene_frame_spec()
    });

    assert_eq!(seen_inputs.len(), 2);
    assert_eq!(seen_inputs[1].pointer_pos, Point { x: -20, y: -30 });
    assert_eq!(frame.frame_input.pointer_pos, Point { x: -20, y: -30 });
}
