#[test]
fn measure_cache_invalidates_when_theme_tokens_change() {
    let content = panel("cached", textbox("cache")).pad_all(0);
    let mut base_tokens = ThemeTokens::main();
    base_tokens.typography.text_scale = 2;
    let spec_a = UiSpec::new(RootFrameSpec::new("root", content.clone()).tokens(base_tokens));

    let mut variant_tokens = base_tokens;
    variant_tokens.typography.text_scale = 3;
    let spec_b = UiSpec::new(RootFrameSpec::new("root", content).tokens(variant_tokens));

    let mut engine = LayoutEngineState::default();
    let input = InputState {
        window_size: Size {
            width: 240,
            height: 160,
        },
        ..InputState::default()
    };
    let theme = Theme::default();
    let mut ui_state = UiState::default();

    let mut canvas = Canvas::new(240, 160);
    let mut layout = Layout::default();
    let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
    let _ = render_checked_with_engine(&spec_a, &mut ui, Point { x: 0, y: 0 }, &mut engine)
        .expect("first render should succeed");
    let first_stats = engine.measure_cache_stats();

    let mut canvas = Canvas::new(240, 160);
    let mut layout = Layout::default();
    let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
    let _ = render_checked_with_engine(&spec_b, &mut ui, Point { x: 0, y: 0 }, &mut engine)
        .expect("second render should succeed");
    let second_stats = engine.measure_cache_stats();

    assert!(
        second_stats.misses > first_stats.misses,
        "changing theme tokens should invalidate cached subtree measurements"
    );
}

#[test]
fn canonical_slot_tree_invariants_hold_for_nested_slot_layouts() {
    let controls = row_slots(vec![
        weighted_slot(panel("knobs", knob("mix", 0.4, (0.0, 1.0))).pad_all(0), 70),
        weighted_slot(panel("dropdowns", dropdown("mode", 3, 0)).pad_all(0), 30),
    ]);
    let content = column_slots(vec![
        weighted_slot(panel("header", textbox("Header")).pad_all(0), 7),
        weighted_slot(panel("curve", textbox("Curve")).pad_all(0), 63),
        weighted_slot(panel("controls", controls).pad_all(0), 30),
    ]);
    let spec = UiSpec::new(
        root_frame_sized(
            "root",
            content,
            Size {
                width: 420,
                height: 258,
            },
        )
        .padding(0),
    );

    measure_checked(&spec).expect("canonical strict slot layout should measure");
    assert_root_slot_tree_invariants(spec.root.content());
}

fn assert_root_slot_tree_invariants(root: &Node) {
    let Node::Slot(slot) = root else {
        panic!("root content must be slot-wrapped");
    };
    assert!(
        is_container(slot.child()),
        "root slot child must be a container node"
    );
    assert_slot_tree_node(slot.child());
}

fn assert_slot_tree_node(node: &Node) {
    match node {
        Node::Slot(slot) => {
            assert!(
                !matches!(slot.child(), Node::Slot(_)),
                "slot child must not be another slot"
            );
            assert_slot_tree_node(slot.child());
        }
        Node::Panel(panel) => assert_slot_tree_node(panel.content()),
        Node::PaddingBox(padding_box) => assert_slot_tree_node(padding_box.content()),
        Node::AlignBox(align_box) => assert_slot_tree_node(align_box.content()),
        Node::AspectBox(aspect_box) => assert_slot_tree_node(aspect_box.content()),
        Node::Row(flex) | Node::Column(flex) => {
            for child in flex.children() {
                assert!(matches!(child, Node::Slot(_)));
                assert_slot_tree_node(child);
            }
        }
        Node::Grid(grid) => {
            for child in grid.children() {
                assert!(matches!(child, Node::Slot(_)));
                assert_slot_tree_node(child);
            }
        }
        Node::Absolute(absolute) => {
            for child in absolute.children() {
                assert!(matches!(child.node(), Node::Slot(_)));
                assert_slot_tree_node(child.node());
            }
        }
        Node::Stack(stack) => {
            for child in stack.children() {
                assert!(matches!(child, Node::Slot(_)));
                assert_slot_tree_node(child);
            }
        }
        Node::ScrollView(scroll_view) => assert_slot_tree_node(scroll_view.content()),
        Node::Wrap(wrap) => {
            for child in wrap.children() {
                assert!(matches!(child, Node::Slot(_)));
                assert_slot_tree_node(child);
            }
        }
        Node::SwitchLayout(switch_layout) => {
            for case_entry in switch_layout.cases() {
                assert!(matches!(case_entry.child(), Node::Slot(_)));
                assert_slot_tree_node(case_entry.child());
            }
            assert!(matches!(switch_layout.fallback(), Node::Slot(_)));
            assert_slot_tree_node(switch_layout.fallback());
        }
        Node::TextBox(_)
        | Node::Spacer(_)
        | Node::Knob(_)
        | Node::Slider(_)
        | Node::Toggle(_)
        | Node::Button(_)
        | Node::Dropdown(_)
        | Node::TabBar(_)
        | Node::CurveEditor(_)
        | Node::EqAttractorSurface(_)
        | Node::Region(_)
        | Node::Indicator(_) => {}
    }
}

fn is_container(node: &Node) -> bool {
    matches!(
        node,
        Node::Panel(_)
            | Node::PaddingBox(_)
            | Node::AlignBox(_)
            | Node::AspectBox(_)
            | Node::Row(_)
            | Node::Column(_)
            | Node::Grid(_)
            | Node::Absolute(_)
            | Node::Stack(_)
            | Node::ScrollView(_)
            | Node::Wrap(_)
            | Node::SwitchLayout(_)
    )
}
