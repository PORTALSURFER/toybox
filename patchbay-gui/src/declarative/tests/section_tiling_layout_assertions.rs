use super::super::super::*;

#[test]
fn root_vertical_sections_tile_parent_without_gaps() {
    let root_size = Size {
        width: 421,
        height: 259,
    };
    let spec = UiSpec::new(
        root_frame_sized(
            "root",
            column_sections(vec![
                weighted(panel("header", label("Header")).pad_all(0), 7),
                weighted(panel("curve", label("Curve")).pad_all(0), 63),
                weighted(panel("controls", label("Controls")).pad_all(0), 30),
            ]),
            root_size,
            root_size,
        )
        .padding(0),
    );

    let measured = measure_checked(&spec).expect("measurement should succeed");
    assert_eq!(measured, root_size);
    let Node::Grid(root_grid) = spec.root.content.as_ref() else {
        panic!("expected grid-backed root sections");
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
    super::section_tiling_layout_helpers::assert_tracks_tile_parent_exactly(
        measured.height,
        root_grid.template.row_gap,
        &row_heights,
    );
}

#[test]
fn root_horizontal_sections_tile_parent_without_gaps() {
    let root_size = Size {
        width: 799,
        height: 301,
    };
    let spec = UiSpec::new(
        root_frame_sized(
            "root",
            row_sections(vec![
                weighted(panel("left", label("Left")).pad_all(0), 17),
                weighted(panel("center", label("Center")).pad_all(0), 55),
                weighted(panel("right", label("Right")).pad_all(0), 28),
            ]),
            root_size,
            root_size,
        )
        .padding(0),
    );

    let measured = measure_checked(&spec).expect("measurement should succeed");
    assert_eq!(measured, root_size);
    let Node::Grid(root_grid) = spec.root.content.as_ref() else {
        panic!("expected grid-backed root sections");
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
    super::section_tiling_layout_helpers::assert_tracks_tile_parent_exactly(
        measured.width,
        root_grid.template.column_gap,
        &column_widths,
    );
}

#[test]
fn nested_section_layouts_tile_each_parent_without_gaps() {
    let right_nested = column_sections(vec![
        weighted(panel("right-top", label("R1")).pad_all(0), 40),
        weighted(panel("right-bottom", label("R2")).pad_all(0), 60),
    ]);
    let controls = row_sections(vec![
        weighted(panel("knobs", label("Knobs")).pad_all(0), 70),
        weighted(panel("dropdowns", right_nested).pad_all(0), 30),
    ]);
    let content = column_sections(vec![
        weighted(panel("header", label("Header")).pad_all(0), 9),
        weighted(panel("curve", label("Curve")).pad_all(0), 61),
        weighted(panel("controls", controls).pad_all(0), 30),
    ]);
    let root_size = Size {
        width: 803,
        height: 511,
    };
    let spec = UiSpec::new(root_frame_sized("root", content, root_size, root_size).padding(0));
    let measured = measure_checked(&spec).expect("measurement should succeed");
    assert_eq!(measured, root_size);

    let Node::Grid(root_grid) = spec.root.content.as_ref() else {
        panic!("expected grid-backed root sections");
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
    super::section_tiling_layout_helpers::assert_tracks_tile_parent_exactly(
        measured.height,
        root_grid.template.row_gap,
        &root_row_heights,
    );

    let controls_panel = match &root_grid.children[2] {
        Node::Panel(panel) => panel,
        other => panic!("expected controls panel, got {other:?}"),
    };
    let Node::Grid(controls_grid) = controls_panel.content.as_ref() else {
        panic!("expected row section grid in controls panel");
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
    super::section_tiling_layout_helpers::assert_tracks_tile_parent_exactly(
        measured.width,
        controls_grid.template.column_gap,
        &controls_column_widths,
    );

    let right_panel = match &controls_grid.children[1] {
        Node::Panel(panel) => panel,
        other => panic!("expected right panel, got {other:?}"),
    };
    let Node::Grid(right_grid) = right_panel.content.as_ref() else {
        panic!("expected nested column section grid in right panel");
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
    super::section_tiling_layout_helpers::assert_tracks_tile_parent_exactly(
        root_row_heights[2],
        right_grid.template.row_gap,
        &nested_row_heights,
    );
}
