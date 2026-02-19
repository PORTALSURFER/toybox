fn selected_panel_key(node: &Node) -> &str {
    let Node::Slot(slot) = node else {
        panic!("switch child must be slot-wrapped");
    };
    let Node::Panel(panel) = slot.child() else {
        panic!("switch slot child must be a panel");
    };
    panel.key.as_str()
}

#[test]
fn switch_layout_selects_first_matching_case_and_respects_boundaries() {
    let spec = SwitchLayoutSpec::new(
        vec![
            when_width_lt(500, panel("compact", label("compact"))),
            when_width_ge(500, panel("wide", label("wide"))),
        ],
        panel("fallback", label("fallback")),
    );

    assert_eq!(selected_panel_key(spec.selected_child(320)), "compact");
    assert_eq!(selected_panel_key(spec.selected_child(499)), "compact");
    assert_eq!(selected_panel_key(spec.selected_child(500)), "wide");
    assert_eq!(selected_panel_key(spec.selected_child(900)), "wide");
}

#[test]
fn switch_layout_uses_fallback_when_no_case_matches() {
    let spec = SwitchLayoutSpec::new(
        vec![
            when_width_lt(300, panel("small", label("small"))),
            when_width_ge(700, panel("large", label("large"))),
        ],
        panel("fallback", label("fallback")),
    );

    assert_eq!(selected_panel_key(spec.selected_child(450)), "fallback");
}

#[test]
fn rejects_switch_case_with_invalid_bounds() {
    let invalid = switch_layout(
        vec![when_width_between(500, 500, panel("bad", label("bad")))],
        panel("fallback", label("fallback")),
    );
    let spec = UiSpec::new(root_frame_sized(
        "root",
        invalid,
        Size {
            width: 420,
            height: 258,
        },
        Size {
            width: 420,
            height: 258,
        },
    ));

    let error = measure_checked(&spec).expect_err("expected invalid switch bounds error");
    assert!(matches!(
        error,
        DeclarativeError::InvalidSwitchCaseRange { case_index, .. } if case_index == 0
    ));
}

#[test]
fn rejects_switch_cases_that_overlap_or_are_unsorted() {
    let overlapping = switch_layout(
        vec![
            when_width_between(0, 500, panel("a", label("a"))),
            when_width_between(400, 800, panel("b", label("b"))),
        ],
        panel("fallback", label("fallback")),
    );
    let spec = UiSpec::new(root_frame_sized(
        "root",
        overlapping,
        Size {
            width: 420,
            height: 258,
        },
        Size {
            width: 420,
            height: 258,
        },
    ));

    let error = measure_checked(&spec).expect_err("expected switch ordering error");
    assert!(matches!(
        error,
        DeclarativeError::InvalidSwitchCaseOrder {
            previous_case_index: 0,
            case_index: 1,
            ..
        }
    ));
}
