use super::super::*;

#[test]
fn rejects_invalid_knob_range() {
    let spec = UiSpec::new(RootFrameSpec::new(
        "root",
        panel(
            "panel",
            Node::Knob(KnobSpec::new("k", "Drive", 0.5, (1.0, 1.0))),
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
            Node::Slider(SliderSpec::new("s", "Shape", 0.5, (0.8, 0.2))),
        ),
    ));
    let error = measure_checked(&spec).expect_err("expected invalid range error");
    assert!(matches!(
        error,
        DeclarativeError::InvalidValueRange { node_kind, .. } if node_kind == "Slider"
    ));
}

#[test]
fn rejects_out_of_range_control_value() {
    let spec = UiSpec::new(RootFrameSpec::new(
        "root",
        panel(
            "panel",
            Node::Slider(SliderSpec::new("s", "Shape", 1.5, (0.0, 1.0))),
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
                "Mode",
                vec!["A".to_string(), "B".to_string()],
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
                SliderSpec::new("s", "Shape", 0.5, (0.0, 1.0)).control_size(Size {
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
fn rejects_non_slot_root_content() {
    let spec = UiSpec::new(RootFrameSpec {
        key: "root".to_string(),
        title: None,
        padding: 0,
        layout: LayoutBox::auto(),
        tokens: None,
        design_size: None,
        scale_mode: RootScaleMode::None,
        zoom_override: None,
        content: Box::new(label("not slotted")),
    });
    let error = measure_checked(&spec).expect_err("expected invalid root slot error");
    assert!(matches!(
        error,
        DeclarativeError::InvalidRootContent { node_kind } if node_kind == "Label"
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
        scale_mode: RootScaleMode::None,
        zoom_override: None,
        content: Box::new(slot(label("bad"))),
    });
    let error = measure_checked(&spec).expect_err("expected invalid root slot child error");
    assert!(matches!(
        error,
        DeclarativeError::InvalidRootSlotChild { node_kind } if node_kind == "Label"
    ));
}

#[test]
fn rejects_container_children_when_not_slot_wrapped() {
    let invalid_row = Node::Row(FlexSpec {
        layout: LayoutBox::auto(),
        gap: 0,
        padding: EdgeInsets::default(),
        align: Align::Start,
        justify: Justify::Start,
        children: vec![label("direct child")],
    });
    let spec = UiSpec::new(RootFrameSpec::new("root", invalid_row));
    let error = measure_checked(&spec).expect_err("expected invalid container child error");
    assert!(matches!(
        error,
        DeclarativeError::InvalidContainerChild {
            container_kind,
            node_kind
        } if container_kind == "Row" && node_kind == "Label"
    ));
}
