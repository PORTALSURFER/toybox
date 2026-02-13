use super::super::*;

#[test]
fn resolve_grid_axis_distributes_fr_remainder_without_slack() {
    let row_heights = resolve_grid_axis(GridAxisResolveRequest {
        tracks: &[TrackSize::Fr(7), TrackSize::Fr(63), TrackSize::Fr(30)],
        columns: 1,
        rows: 3,
        gap: 0,
        available: 259,
        is_columns: false,
        intrinsic: &[Size {
            width: 0,
            height: 0,
        }; 3],
    });
    assert_eq!(row_heights, vec![18, 163, 78]);
    assert_eq!(row_heights.iter().sum::<u32>(), 259);

    let column_widths = resolve_grid_axis(GridAxisResolveRequest {
        tracks: &[TrackSize::Fr(70), TrackSize::Fr(30)],
        columns: 2,
        rows: 1,
        gap: 0,
        available: 799,
        is_columns: true,
        intrinsic: &[Size {
            width: 0,
            height: 0,
        }; 2],
    });
    assert_eq!(column_widths, vec![559, 240]);
    assert_eq!(column_widths.iter().sum::<u32>(), 799);
}

#[test]
fn root_frame_sized_uses_window_size_with_minimum_floor() {
    let root = root_frame_sized(
        "root",
        label("x"),
        Size {
            width: 420,
            height: 258,
        },
        Size {
            width: 360,
            height: 400,
        },
    );
    assert_eq!(
        root.layout,
        LayoutBox::fixed(420, 400).max(420, 400),
        "root should clamp to min width and use host-provided height"
    );
    assert_eq!(root.scale_mode, RootScaleMode::None);
    assert_eq!(root.design_size, None);
}

#[test]
fn nested_section_helpers_measure_successfully() {
    let controls = row_sections(vec![
        weighted(panel("left", label("Knobs")).pad_all(0), 70),
        weighted(panel("right", label("Dropdowns")).pad_all(0), 30),
    ]);
    let content = column_sections(vec![
        weighted(panel("header", label("Header")).pad_all(0), 7),
        weighted(panel("curve", label("Curve")).pad_all(0), 63),
        weighted(panel("controls", controls).pad_all(0), 30),
    ]);
    let spec = UiSpec::new(
        root_frame_sized(
            "root",
            content,
            Size {
                width: 420,
                height: 258,
            },
            Size {
                width: 840,
                height: 516,
            },
        )
        .padding(0),
    );

    let measured = measure_checked(&spec).expect("nested section helpers should validate");
    assert!(measured.width >= 420);
    assert!(measured.height >= 258);
}

#[test]
fn justify_weighting_and_distribution_cover_new_modes() {
    let between = justify_space_weights(Justify::SpaceBetween, 3);
    assert_eq!(between, vec![0, 1, 1, 0]);

    let around = justify_space_weights(Justify::SpaceAround, 3);
    assert_eq!(around, vec![1, 2, 2, 1]);

    let evenly = justify_space_weights(Justify::SpaceEvenly, 3);
    assert_eq!(evenly, vec![1, 1, 1, 1]);

    let distributed = distribute_space(7, &[1, 2, 0, 1]);
    assert_eq!(distributed.iter().sum::<i32>(), 7);
    assert_eq!(distributed[2], 0);
}

#[test]
fn node_layout_helpers_apply_constraints_when_supported() {
    let node = panel("main", label("x")).fill_width();
    match node {
        Node::Panel(panel) => {
            assert_eq!(panel.layout.width, Length::Fill(1));
            assert_eq!(panel.layout.height, Length::Auto);
        }
        _ => panic!("expected panel node"),
    }

    let spacer_node = spacer(Size {
        width: 10,
        height: 10,
    })
    .fill();
    assert!(matches!(spacer_node, Node::Spacer(_)));
}
