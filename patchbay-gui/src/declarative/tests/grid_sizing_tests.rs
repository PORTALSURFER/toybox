use super::super::*;

#[test]
fn clamp_size_to_available_caps_oversized_children() {
    let available = Size {
        width: 60,
        height: 40,
    };
    let resolved = Size {
        width: 80,
        height: 70,
    };
    let clamped = clamp_size_to_available(resolved, available);
    assert_eq!(clamped, available);
}

#[test]
fn rejects_duplicate_widget_keys() {
    let spec = UiSpec::new(RootFrameSpec::new(
        "root",
        Node::column(vec![
            Node::Knob(KnobSpec::new("k", 0.5, (0.0, 1.0))),
            Node::Knob(KnobSpec::new("k", 0.5, (0.0, 1.0))),
        ]),
    ));
    let error = measure_checked(&spec).expect_err("expected duplicate key error");
    assert!(matches!(error, DeclarativeError::DuplicateNodeKey { .. }));
}

#[test]
fn rejects_root_key_collision_with_child() {
    let spec = UiSpec::new(RootFrameSpec::new(
        "dup",
        Node::Panel(PanelSpec::new("dup", textbox("content"))),
    ));
    let error = measure_checked(&spec).expect_err("expected duplicate key error");
    assert!(matches!(error, DeclarativeError::DuplicateNodeKey { key } if key == "dup"));
}

#[test]
fn measures_grid_from_template_and_children() {
    let grid = GridSpec::new(
        GridTemplate::new(vec![TrackSize::Fr(1), TrackSize::Fr(1)]),
        vec![
            Node::Spacer(
                SpacerSpec::new()
                    .layout(LayoutBox::fixed(10, 12).max(10, 12)),
            ),
            Node::Spacer(
                SpacerSpec::new()
                    .layout(LayoutBox::fixed(20, 14).max(20, 14)),
            ),
        ],
    );
    let spec = UiSpec::new(RootFrameSpec::new("root", Node::Grid(grid)));
    let measured = measure_checked(&spec).expect("measurement should succeed");
    assert!(measured.width >= 30);
    assert!(measured.height >= 14);
}

#[test]
fn grid_measure_uses_tight_tracks_without_gap_spacing() {
    let grid = GridSpec::new(
        GridTemplate::columns_fr(2),
        vec![
            spacer(Size {
                width: 10,
                height: 10,
            }),
            spacer(Size {
                width: 10,
                height: 10,
            }),
            spacer(Size {
                width: 10,
                height: 10,
            }),
            spacer(Size {
                width: 10,
                height: 10,
            }),
        ],
    );
    let spec = UiSpec::new(RootFrameSpec::new("root", Node::Grid(grid)).padding(0));
    let measured = measure_checked(&spec).expect("measurement should succeed");
    assert_eq!(measured.width, 20);
    assert_eq!(measured.height, 20);
}
