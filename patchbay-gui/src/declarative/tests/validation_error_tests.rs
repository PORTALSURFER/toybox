use super::super::*;

#[test]
fn rejects_invalid_knob_range() {
    let spec = UiSpec::new(RootFrameSpec::new(
        "root",
        panel(
            "panel",
            Node::Knob(KnobSpec::new("k", 0.5, (1.0, 1.0))),
        ),
    ));
    let error = measure_checked(&spec).expect_err("expected invalid range error");
    assert!(matches!(
        error,
        DeclarativeError::InvalidValueRange { node_kind, .. } if node_kind == "Knob"
    ));
}

#[test]
fn rejects_invalid_slider_range() {
    let spec = UiSpec::new(RootFrameSpec::new(
        "root",
        panel(
            "panel",
            Node::Slider(SliderSpec::new("s", 0.5, (0.8, 0.2))),
        ),
    ));
    let error = measure_checked(&spec).expect_err("expected invalid range error");
    assert!(matches!(
        error,
        DeclarativeError::InvalidValueRange { node_kind, .. } if node_kind == "Slider"
    ));
}

#[test]
fn rejects_zero_aspect_ratio_components() {
    let spec = UiSpec::new(RootFrameSpec::new(
        "root",
        aspect_box(
            panel("panel", textbox("x")),
            AspectRatio::new(0, 1),
        ),
    ));
    let error = measure_checked(&spec).expect_err("expected invalid aspect ratio");
    assert!(matches!(
        error,
        DeclarativeError::InvalidAspectRatio { width, height } if width == 0 && height == 1
    ));
}

#[test]
fn rejects_out_of_range_control_value() {
    let spec = UiSpec::new(RootFrameSpec::new(
        "root",
        panel(
            "panel",
            Node::Slider(SliderSpec::new("s", 1.5, (0.0, 1.0))),
        ),
    ));
    let error = measure_checked(&spec).expect_err("expected invalid control value");
    assert!(matches!(
        error,
        DeclarativeError::InvalidControlValue { node_kind, key, .. }
            if node_kind == "Slider" && key == "s"
    ));
}

#[test]
fn rejects_dropdown_selection_out_of_bounds() {
    let spec = UiSpec::new(RootFrameSpec::new(
        "root",
        panel(
            "panel",
            Node::Dropdown(DropdownSpec::new(
                "mode",
                2,
                2,
            )),
        ),
    ));
    let error = measure_checked(&spec).expect_err("expected invalid dropdown selection");
    assert!(matches!(
        error,
        DeclarativeError::InvalidDropdownSelection {
            key,
            selected,
            options_len
        } if key == "mode" && selected == 2 && options_len == 2
    ));
}

#[test]
fn rejects_zero_control_size() {
    let spec = UiSpec::new(RootFrameSpec::new(
        "root",
        panel(
            "panel",
            Node::Slider(
                SliderSpec::new("s", 0.5, (0.0, 1.0)).control_size(Size {
                    width: 0,
                    height: 24,
                }),
            ),
        ),
    ));
    let error = measure_checked(&spec).expect_err("expected invalid control size");
    assert!(matches!(
        error,
        DeclarativeError::InvalidControlSize { node_kind, .. } if node_kind == "Slider"
    ));
}

#[test]
fn rejects_inverted_label_layout_bounds() {
    let spec = UiSpec::new(RootFrameSpec::new(
        "root",
        panel(
            "panel",
            textbox("bad-layout").widget_layout(LayoutBox::auto().min(100, 8).max(20, 40)),
        ),
    ));
    let error = measure_checked(&spec).expect_err("expected invalid text box layout bounds");
    assert!(matches!(
        error,
        DeclarativeError::InvalidLayoutBounds {
            node_kind,
            axis,
            min,
            max
        } if node_kind == "TextBox" && axis == "width" && min == 100 && max == 20
    ));
}

#[test]
fn rejects_inverted_root_layout_bounds() {
    let spec = UiSpec::new(
        RootFrameSpec::new("root", panel("panel", textbox("x")))
            .layout(LayoutBox::auto().min(320, 120).max(200, 160)),
    );
    let error = measure_checked(&spec).expect_err("expected invalid root layout bounds");
    assert!(matches!(
        error,
        DeclarativeError::InvalidLayoutBounds {
            node_kind,
            axis,
            min,
            max
        } if node_kind == "RootFrame" && axis == "width" && min == 320 && max == 200
    ));
}

#[test]
fn rejects_inverted_slot_widget_layout_bounds() {
    let spec = UiSpec::new(RootFrameSpec::new(
        "root",
        row_slots(vec![weighted_slot(textbox("x"), 1).width_bounds(Some(72), Some(24))]),
    ));
    let error = measure_checked(&spec).expect_err("expected invalid slot-derived layout bounds");
    assert!(matches!(
        error,
        DeclarativeError::InvalidLayoutBounds {
            node_kind,
            axis,
            min,
            max
        } if node_kind == "TextBox" && axis == "width" && min == 72 && max == 24
    ));
}

#[test]
fn rejects_editable_text_box_with_empty_key() {
    let spec = UiSpec::new(RootFrameSpec::new(
        "root",
        panel("panel", textbox("Init").text_editable("", true)),
    ));
    let error = measure_checked(&spec).expect_err("expected invalid editable text-box key");
    assert!(matches!(
        error,
        DeclarativeError::EmptyNodeKey { node_kind } if node_kind == "TextBox"
    ));
}

#[test]
fn rejects_non_slot_root_content() {
    let spec = UiSpec::new(RootFrameSpec {
        key: "root".to_string(),
        title: None,
        padding: 0,
        layout: LayoutBox::auto(),
        tokens: None,
        design_size: None,
        scale_mode: RootScaleMode::UniformFit,
        zoom_override: None,
        layout_diagnostics_mode: LayoutDiagnosticsMode::EventsOnly,
        content: Box::new(textbox("not slotted")),
    });
    let error = measure_checked(&spec).expect_err("expected invalid root slot error");
    assert!(matches!(
        error,
        DeclarativeError::InvalidRootContent { node_kind } if node_kind == "TextBox"
    ));
}

#[test]
fn rejects_root_slot_child_when_not_container() {
    let spec = UiSpec::new(RootFrameSpec {
        key: "root".to_string(),
        title: None,
        padding: 0,
        layout: LayoutBox::auto(),
        tokens: None,
        design_size: None,
        scale_mode: RootScaleMode::UniformFit,
        zoom_override: None,
        layout_diagnostics_mode: LayoutDiagnosticsMode::EventsOnly,
        content: Box::new(slot(textbox("bad"))),
    });
    let error = measure_checked(&spec).expect_err("expected invalid root slot child error");
    assert!(matches!(
        error,
        DeclarativeError::InvalidRootSlotChild { node_kind } if node_kind == "TextBox"
    ));
}

#[test]
fn rejects_container_children_when_not_slot_wrapped() {
    let invalid_row = Node::Row(FlexSpec {
        layout: ContainerLayout::auto(),
        gap: 0,
        padding: EdgeInsets::default(),
        align: Align::Start,
        justify: Justify::Start,
        children: vec![textbox("direct child")],
    });
    let spec = UiSpec::new(RootFrameSpec::new("root", invalid_row));
    let error = measure_checked(&spec).expect_err("expected invalid container child error");
    assert!(matches!(
        error,
        DeclarativeError::InvalidContainerChild {
            container_kind,
            node_kind
        } if container_kind == "Row" && node_kind == "TextBox"
    ));
}

#[test]
fn rejects_slot_child_when_nested_slot() {
    let spec = UiSpec::new(RootFrameSpec {
        key: "root".to_string(),
        title: None,
        padding: 0,
        layout: LayoutBox::auto(),
        tokens: None,
        design_size: None,
        scale_mode: RootScaleMode::UniformFit,
        zoom_override: None,
        layout_diagnostics_mode: LayoutDiagnosticsMode::EventsOnly,
        content: Box::new(slot(panel(
            "p",
            Node::Slot(SlotSpec::new(Node::Slot(SlotSpec::new(textbox("bad"))))),
        ))),
    });
    let error = measure_checked(&spec).expect_err("expected invalid nested-slot child error");
    assert!(matches!(
        error,
        DeclarativeError::InvalidSlotChild { node_kind } if node_kind == "Slot"
    ));
}

#[test]
fn rejects_slot_grid_with_invalid_percent_total() {
    let invalid_grid = Node::Grid(GridSpec {
        layout: ContainerLayout::fill(),
        template: GridTemplate::new(vec![TrackSize::Percent(70), TrackSize::Percent(60)])
            .rows(vec![TrackSize::Fr(1)]),
        children: vec![slot(textbox("left")), slot(textbox("right"))],
        kind: GridKind::SlotRow,
    });
    let spec = UiSpec::new(RootFrameSpec::new("root", invalid_grid).layout(LayoutBox::fixed(100, 40)));
    let error = measure_checked(&spec).expect_err("expected invalid slot fractions error");
    assert!(matches!(
        error,
        DeclarativeError::InvalidSlotFractions {
            total_percent,
            fill_count
        } if total_percent == 130 && fill_count == 0
    ));
}

#[test]
fn accepts_canonical_root_slot_tree() {
    let content = column_slots(vec![
        weighted_slot(panel("header", textbox("Header")), 20),
        fill_slot(panel(
            "controls",
            row_slots(vec![
                fraction_slot(panel("left", knob("mix", 0.5, (0.0, 1.0))), 50),
                fill_slot(panel(
                    "right",
                    dropdown(
                        "mode",
                        2,
                        0,
                    ),
                )),
            ]),
        )),
    ]);
    let spec = UiSpec::new(root_frame_sized(
        "root",
        content,
        Size {
            width: 320,
            height: 200,
        },
    ));
    let measured = measure_checked(&spec).expect("canonical slot tree should validate");
    assert_eq!(
        measured,
        Size {
            width: 320,
            height: 200,
        }
    );
}

#[test]
fn rejects_excessive_tree_depth_before_measurement() {
    let mut node = textbox("leaf");
    for index in 0..500 {
        node = panel(format!("layer-{index}"), node).pad_all(0);
    }
    let spec = UiSpec::new(root_frame_sized(
        "root",
        node,
        Size {
            width: 480,
            height: 270,
        },
    ));

    let error = measure_checked(&spec).expect_err("expected depth guard error");
    assert!(matches!(
        error,
        DeclarativeError::TreeDepthExceeded { max_depth, actual_depth, .. }
            if actual_depth > max_depth
    ));
}
