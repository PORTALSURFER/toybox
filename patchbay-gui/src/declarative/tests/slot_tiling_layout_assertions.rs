use super::super::super::*;

fn expect_slot_child<'a>(node: &'a Node, label: &str) -> &'a Node {
    match node {
        Node::Slot(slot) => slot.child.as_ref(),
        other => panic!("expected {label} slot wrapper, got {other:?}"),
    }
}

fn expect_slot_wrapped_panel<'a>(node: &'a Node, label: &str) -> &'a PanelSpec {
    match expect_slot_child(node, label) {
        Node::Row(row) => match row.children.as_slice() {
            [child] => match expect_slot_child(child, label) {
                Node::Panel(panel) => panel,
                other => panic!("expected {label} row to contain panel, got {other:?}"),
            },
            _ => panic!("expected {label} row to contain exactly one child"),
        },
        Node::Panel(panel) => panel,
        other => panic!("expected {label} panel (or row wrapper), got {other:?}"),
    }
}

#[test]
fn root_vertical_slots_tile_parent_without_gaps() {
    let root_size = Size {
        width: 421,
        height: 259,
    };
    let spec = UiSpec::new(
        root_frame_sized(
            "root",
            column_slots(vec![
                weighted_slot(panel("header", label("Header")).pad_all(0), 7),
                weighted_slot(panel("curve", label("Curve")).pad_all(0), 63),
                weighted_slot(panel("controls", label("Controls")).pad_all(0), 30),
            ]),
            root_size,
            root_size,
        )
        .padding(0),
    );

    let measured = measure_checked(&spec).expect("measurement should succeed");
    assert_eq!(measured, root_size);
    let Node::Grid(root_grid) = expect_slot_child(spec.root.content.as_ref(), "root") else {
        panic!("expected grid-backed root slots");
    };
    let row_heights = resolve_grid_axis(GridAxisResolveRequest {
        tracks: &root_grid.template.rows,
        columns: 1,
        rows: root_grid.children.len(),
        gap: root_grid.template.row_gap,
        available: measured.height,
        is_columns: false,
        intrinsic: &vec![
            Size {
                width: 0,
                height: 0,
            };
            root_grid.children.len()
        ],
    });
    super::slot_tiling_layout_helpers::assert_tracks_tile_parent_exactly(
        measured.height,
        root_grid.template.row_gap,
        &row_heights,
    );
}

#[test]
fn root_horizontal_slots_tile_parent_without_gaps() {
    let root_size = Size {
        width: 799,
        height: 301,
    };
    let spec = UiSpec::new(
        root_frame_sized(
            "root",
            row_slots(vec![
                weighted_slot(panel("left", label("Left")).pad_all(0), 17),
                weighted_slot(panel("center", label("Center")).pad_all(0), 55),
                weighted_slot(panel("right", label("Right")).pad_all(0), 28),
            ]),
            root_size,
            root_size,
        )
        .padding(0),
    );

    let measured = measure_checked(&spec).expect("measurement should succeed");
    assert_eq!(measured, root_size);
    let Node::Grid(root_grid) = expect_slot_child(spec.root.content.as_ref(), "root") else {
        panic!("expected grid-backed root slots");
    };
    let column_widths = resolve_grid_axis(GridAxisResolveRequest {
        tracks: &root_grid.template.columns,
        columns: root_grid.template.columns.len(),
        rows: 1,
        gap: root_grid.template.column_gap,
        available: measured.width,
        is_columns: true,
        intrinsic: &vec![
            Size {
                width: 0,
                height: 0,
            };
            root_grid.children.len()
        ],
    });
    super::slot_tiling_layout_helpers::assert_tracks_tile_parent_exactly(
        measured.width,
        root_grid.template.column_gap,
        &column_widths,
    );
}

#[test]
fn nested_slot_layouts_tile_each_parent_without_gaps() {
    let right_nested = column_slots(vec![
        weighted_slot(panel("right-top", label("R1")).pad_all(0), 40),
        weighted_slot(panel("right-bottom", label("R2")).pad_all(0), 60),
    ]);
    let controls = row_slots(vec![
        weighted_slot(panel("knobs", label("Knobs")).pad_all(0), 70),
        weighted_slot(panel("dropdowns", right_nested).pad_all(0), 30),
    ]);
    let content = column_slots(vec![
        weighted_slot(panel("header", label("Header")).pad_all(0), 9),
        weighted_slot(panel("curve", label("Curve")).pad_all(0), 61),
        weighted_slot(panel("controls", controls).pad_all(0), 30),
    ]);
    let root_size = Size {
        width: 803,
        height: 511,
    };
    let spec = UiSpec::new(root_frame_sized("root", content, root_size, root_size).padding(0));
    let measured = measure_checked(&spec).expect("measurement should succeed");
    assert_eq!(measured, root_size);

    let Node::Grid(root_grid) = expect_slot_child(spec.root.content.as_ref(), "root") else {
        panic!("expected grid-backed root slots");
    };
    let root_row_heights = resolve_grid_axis(GridAxisResolveRequest {
        tracks: &root_grid.template.rows,
        columns: 1,
        rows: root_grid.children.len(),
        gap: root_grid.template.row_gap,
        available: measured.height,
        is_columns: false,
        intrinsic: &vec![
            Size {
                width: 0,
                height: 0,
            };
            root_grid.children.len()
        ],
    });
    super::slot_tiling_layout_helpers::assert_tracks_tile_parent_exactly(
        measured.height,
        root_grid.template.row_gap,
        &root_row_heights,
    );

    let controls_panel = expect_slot_wrapped_panel(&root_grid.children[2], "controls");
    let Node::Grid(controls_grid) = expect_slot_child(controls_panel.content.as_ref(), "controls") else {
        panic!("expected row slot grid in controls panel");
    };
    let controls_column_widths = resolve_grid_axis(GridAxisResolveRequest {
        tracks: &controls_grid.template.columns,
        columns: controls_grid.template.columns.len(),
        rows: 1,
        gap: controls_grid.template.column_gap,
        available: measured.width,
        is_columns: true,
        intrinsic: &vec![
            Size {
                width: 0,
                height: 0,
            };
            controls_grid.children.len()
        ],
    });
    super::slot_tiling_layout_helpers::assert_tracks_tile_parent_exactly(
        measured.width,
        controls_grid.template.column_gap,
        &controls_column_widths,
    );

    let right_panel = expect_slot_wrapped_panel(&controls_grid.children[1], "right");
    let Node::Grid(right_grid) = expect_slot_child(right_panel.content.as_ref(), "right") else {
        panic!("expected nested column slot grid in right panel");
    };
    let nested_row_heights = resolve_grid_axis(GridAxisResolveRequest {
        tracks: &right_grid.template.rows,
        columns: 1,
        rows: right_grid.children.len(),
        gap: right_grid.template.row_gap,
        available: root_row_heights[2],
        is_columns: false,
        intrinsic: &vec![
            Size {
                width: 0,
                height: 0,
            };
            right_grid.children.len()
        ],
    });
    super::slot_tiling_layout_helpers::assert_tracks_tile_parent_exactly(
        root_row_heights[2],
        right_grid.template.row_gap,
        &nested_row_heights,
    );
}
