#[test]
fn stack_measure_uses_largest_child_extent_plus_padding() {
    let content = stack(vec![
        label("small").widget_layout(LayoutBox::fixed(20, 10).max(20, 10)),
        label("large").widget_layout(LayoutBox::fixed(80, 40).max(80, 40)),
    ])
    .pad_xy(3, 5);
    let spec = UiSpec::new(RootFrameSpec::new("root", content).padding(0));
    let measured = measure_checked(&spec).expect("stack measure should succeed");
    assert_eq!(
        measured,
        Size {
            width: 86,
            height: 50,
        }
    );
}

#[test]
fn scroll_view_measure_tracks_content_size() {
    let content = scroll_view(
        column(vec![
            label("line-1").widget_layout(LayoutBox::fixed(30, 12).max(30, 12)),
            label("line-2").widget_layout(LayoutBox::fixed(30, 12).max(30, 12)),
            label("line-3").widget_layout(LayoutBox::fixed(30, 12).max(30, 12)),
        ])
        .gap(2),
    )
    .pad_xy(2, 1);
    let spec = UiSpec::new(RootFrameSpec::new("root", content).padding(0));
    let measured = measure_checked(&spec).expect("scroll-view measure should succeed");
    assert_eq!(
        measured,
        Size {
            width: 34,
            height: 42,
        }
    );
}

#[test]
fn wrap_measure_accumulates_children_and_gaps() {
    let content = wrap(vec![
        label("a").widget_layout(LayoutBox::fixed(10, 10).max(10, 10)),
        label("b").widget_layout(LayoutBox::fixed(20, 15).max(20, 15)),
        label("c").widget_layout(LayoutBox::fixed(30, 5).max(30, 5)),
    ])
    .gap(3)
    .pad_xy(1, 2);
    let spec = UiSpec::new(RootFrameSpec::new("root", content).padding(0));
    let measured = measure_checked(&spec).expect("wrap measure should succeed");
    assert_eq!(
        measured,
        Size {
            width: 68,
            height: 19,
        }
    );
}

#[test]
fn render_checked_with_engine_reuses_measure_cache_for_stable_inputs() {
    let spec = UiSpec::new(RootFrameSpec::new(
        "root",
        panel("p", label("cached").widget_layout(LayoutBox::fixed(40, 16).max(40, 16))).pad_all(0),
    ));

    let mut engine = LayoutEngineState::default();
    engine.invalidate_all_measure();

    let input = InputState {
        window_size: Size {
            width: 100,
            height: 80,
        },
        ..InputState::default()
    };

    let mut canvas = Canvas::new(100, 80);
    let mut layout = Layout::default();
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
    let first = render_checked_with_engine(&spec, &mut ui, Point { x: 0, y: 0 }, &mut engine)
        .expect("first render should succeed");
    let first_stats = engine.measure_cache_stats();
    assert_eq!(first_stats.hits, 0);
    assert!(first_stats.misses > 0);

    let mut canvas = Canvas::new(100, 80);
    let mut layout = Layout::default();
    let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
    let second = render_checked_with_engine(&spec, &mut ui, Point { x: 0, y: 0 }, &mut engine)
        .expect("second render should succeed");
    let second_stats = engine.measure_cache_stats();
    assert!(second_stats.hits > first_stats.hits);
    assert_eq!(second_stats.misses, first_stats.misses);
    assert_eq!(first.measured_size, second.measured_size);
}

#[test]
fn runtime_style_conservative_invalidation_remeasures_after_actions() {
    let spec = UiSpec::new(RootFrameSpec::new(
        "root",
        Node::Panel(PanelSpec::new(
            "panel",
            Node::Button(ButtonSpec::new("ok", "OK").control_size(Size {
                width: 80,
                height: 24,
            })),
        )),
    ));
    let mut engine = LayoutEngineState::default();
    let theme = Theme::default();
    let mut ui_state = UiState::default();

    let mut canvas = Canvas::new(200, 120);
    let mut layout = Layout::default();
    let input_press = InputState {
        pointer_pos: Point { x: 24, y: 24 },
        mouse_pressed: true,
        window_size: Size {
            width: 200,
            height: 120,
        },
        ..InputState::default()
    };
    let mut ui = Ui::new(&mut canvas, &input_press, &mut ui_state, &mut layout, &theme);
    let first = render_checked_with_engine(&spec, &mut ui, Point { x: 0, y: 0 }, &mut engine)
        .expect("first render should succeed");
    assert!(!first.actions.is_empty(), "button press should emit action");
    let baseline = engine.measure_cache_stats();
    engine.invalidate_all_measure();

    let mut canvas = Canvas::new(200, 120);
    let mut layout = Layout::default();
    let input_idle = InputState {
        window_size: Size {
            width: 200,
            height: 120,
        },
        ..InputState::default()
    };
    let mut ui = Ui::new(&mut canvas, &input_idle, &mut ui_state, &mut layout, &theme);
    let _ = render_checked_with_engine(&spec, &mut ui, Point { x: 0, y: 0 }, &mut engine)
        .expect("second render should succeed");
    let after = engine.measure_cache_stats();
    assert!(after.misses > baseline.misses);
}

#[test]
fn layout_engine_resolves_node_ids_and_supports_subtree_measure_invalidation() {
    let spec = UiSpec::new(RootFrameSpec::new(
        "root",
        row_slots(vec![
            weighted_slot(panel("left", label("left")).pad_all(0), 1),
            weighted_slot(panel("right", label("right")).pad_all(0), 1),
        ])
        .pad_all(0),
    ));
    let input = InputState {
        window_size: Size {
            width: 200,
            height: 80,
        },
        ..InputState::default()
    };
    let mut engine = LayoutEngineState::default();
    let mut ui_state = UiState::default();
    let theme = Theme::default();

    let mut canvas = Canvas::new(200, 80);
    let mut layout = Layout::default();
    let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
    render_checked_with_engine(&spec, &mut ui, Point { x: 0, y: 0 }, &mut engine)
        .expect("initial render should succeed");
    let baseline = engine.measure_cache_stats();

    let left_id = engine
        .node_id_for_key("left")
        .expect("left panel id should be registered");
    let right_id = engine
        .node_id_for_key("right")
        .expect("right panel id should be registered");
    assert!(engine.contains_node(left_id));
    assert!(engine.contains_node(right_id));
    assert_ne!(left_id, right_id);

    engine.invalidate_measure_subtree(left_id);
    let mut canvas = Canvas::new(200, 80);
    let mut layout = Layout::default();
    let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
    render_checked_with_engine(&spec, &mut ui, Point { x: 0, y: 0 }, &mut engine)
        .expect("re-render after subtree invalidation should succeed");
    let after = engine.measure_cache_stats();
    assert!(after.hits > baseline.hits);
    assert!(after.misses > baseline.misses);
}

#[test]
fn stress_deeply_nested_panel_tree_measures_without_failure() {
    let mut node = label("leaf");
    for index in 0..300 {
        node = panel(format!("layer-{index}"), node).pad_all(0);
    }
    let spec = UiSpec::new(root_frame_sized(
        "root",
        node,
        Size {
            width: 400,
            height: 240,
        },
        Size {
            width: 400,
            height: 240,
        },
    ));
    let measured = measure_checked(&spec).expect("deep tree should measure");
    assert!(measured.width > 0 && measured.height > 0);
}

#[test]
fn stress_large_slot_list_measures_without_gaps_or_panics() {
    let mut slots = Vec::new();
    for index in 0..1200 {
        slots.push(weighted_slot(
            panel(format!("item-{index}"), label("x")).pad_all(0),
            1,
        ));
    }
    let content = row_slots(slots);
    let spec = UiSpec::new(root_frame_sized(
        "root",
        content,
        Size {
            width: 1200,
            height: 120,
        },
        Size {
            width: 1200,
            height: 120,
        },
    ));
    let measured = measure_checked(&spec).expect("large slot list should measure");
    assert!(measured.width >= 1200);
    assert!(measured.height >= 120);
}

#[test]
fn layout_is_deterministic_across_repeated_renders_for_multiple_root_sizes() {
    let spec = UiSpec::new(root_frame_sized(
        "root",
        wrap(vec![
            panel("a", label("A")).pad_all(0),
            panel("b", label("B")).pad_all(0),
            panel("c", label("C")).pad_all(0),
        ])
        .gap(6)
        .pad_all(4),
        Size {
            width: 420,
            height: 258,
        },
        Size {
            width: 420,
            height: 258,
        },
    ));
    let sizes = [
        Size {
            width: 420,
            height: 258,
        },
        Size {
            width: 840,
            height: 516,
        },
        Size {
            width: 300,
            height: 200,
        },
    ];

    for size in sizes {
        let first = render_spec_for_size(&spec, size);
        let second = render_spec_for_size(&spec, size);
        assert_eq!(first, second, "layout should be deterministic for {size:?}");
        assert!(first.resolved_scale.is_finite());
        assert!(first.content_rect.size.width > 0);
        assert!(first.content_rect.size.height > 0);
    }
}

fn render_spec_for_size(spec: &UiSpec, size: Size) -> RenderResult {
    let input = InputState {
        window_size: size,
        ..InputState::default()
    };
    let mut canvas = Canvas::new(size.width, size.height);
    let mut layout = Layout::default();
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
    render_checked(spec, &mut ui, Point { x: 0, y: 0 }).expect("render should succeed")
}
