#[test]
fn stack_measure_uses_largest_child_extent_plus_padding() {
    let content = stack(vec![
        textbox("small").widget_layout(LayoutBox::fixed(20, 10).max(20, 10)),
        textbox("large").widget_layout(LayoutBox::fixed(80, 40).max(80, 40)),
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
            textbox("line-1").widget_layout(LayoutBox::fixed(30, 12).max(30, 12)),
            textbox("line-2").widget_layout(LayoutBox::fixed(30, 12).max(30, 12)),
            textbox("line-3").widget_layout(LayoutBox::fixed(30, 12).max(30, 12)),
        ]),
    )
    .pad_xy(2, 1);
    let spec = UiSpec::new(RootFrameSpec::new("root", content).padding(0));
    let measured = measure_checked(&spec).expect("scroll-view measure should succeed");
    assert_eq!(
        measured,
        Size {
            width: 34,
            height: 38,
        }
    );
}

#[test]
fn wrap_measure_accumulates_children_without_gap_spacing() {
    let content = wrap(vec![
        textbox("a").widget_layout(LayoutBox::fixed(10, 10).max(10, 10)),
        textbox("b").widget_layout(LayoutBox::fixed(20, 15).max(20, 15)),
        textbox("c").widget_layout(LayoutBox::fixed(30, 5).max(30, 5)),
    ])
    .pad_xy(1, 2);
    let spec = UiSpec::new(RootFrameSpec::new("root", content).padding(0));
    let measured = measure_checked(&spec).expect("wrap measure should succeed");
    assert_eq!(
        measured,
        Size {
            width: 62,
            height: 19,
        }
    );
}

#[test]
fn render_checked_with_engine_reuses_measure_cache_for_stable_inputs() {
    let spec = UiSpec::new(RootFrameSpec::new(
        "root",
        panel("p", textbox("cached").widget_layout(LayoutBox::fixed(40, 16).max(40, 16))).pad_all(0),
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
fn ui_action_invalidation_scope_matches_variant_contract() {
    assert_eq!(
        UiAction::KnobChanged {
            key: "k".to_string(),
            value: 0.5,
        }
        .invalidation_scope(),
        UiInvalidationScope::MeasureSubtree
    );
    assert_eq!(
        UiAction::DropdownSelected {
            key: "d".to_string(),
            index: 1,
        }
        .invalidation_scope(),
        UiInvalidationScope::MeasureSubtree
    );
    assert_eq!(
        UiAction::TabSelected {
            key: "tabs".to_string(),
            index: 1,
        }
        .invalidation_scope(),
        UiInvalidationScope::MeasureSubtree
    );
    assert_eq!(
        UiAction::RegionHover {
            key: "r".to_string(),
            hovered: true,
            local_pointer: Point { x: 1, y: 2 },
        }
        .invalidation_scope(),
        UiInvalidationScope::LayoutSubtree
    );
    assert_eq!(
        UiAction::EqAttractorSurfaceChanged {
            key: "eq".to_string(),
            action: EqAttractorSurfaceAction::Select { id: 1 },
        }
        .invalidation_scope(),
        UiInvalidationScope::LayoutSubtree
    );
}

#[test]
fn runtime_style_targeted_invalidation_is_narrower_than_full_invalidation() {
    let spec = UiSpec::new(RootFrameSpec::new(
        "root",
        row_slots(vec![
            weighted_slot(
                panel(
                    "left-panel",
                    button("left-btn").control_size(Size {
                        width: 80,
                        height: 24,
                    }),
                )
                .pad_all(0),
                1,
            ),
            weighted_slot(
                panel(
                    "right-panel",
                    button("right-btn").control_size(Size {
                        width: 80,
                        height: 24,
                    }),
                )
                .pad_all(0),
                1,
            ),
        ])
        .pad_all(0),
    ));
    let mut engine = LayoutEngineState::default();
    let theme = Theme::default();
    let mut ui_state = UiState::default();

    let mut canvas = Canvas::new(200, 120);
    let mut layout = Layout::default();
    let input_press = InputState {
        pointer_pos: Point { x: 20, y: 20 },
        mouse_pressed: true,
        window_size: Size {
            width: 240,
            height: 120,
        },
        ..InputState::default()
    };
    let mut ui = Ui::new(&mut canvas, &input_press, &mut ui_state, &mut layout, &theme);
    let first = render_checked_with_engine(&spec, &mut ui, Point { x: 0, y: 0 }, &mut engine)
        .expect("first render should succeed");
    assert!(
        first
            .actions
            .iter()
            .any(|action| matches!(action, UiAction::ButtonPressed { key } if key == "left-btn")),
        "left button press should emit keyed action"
    );
    let baseline = engine.measure_cache_stats();
    let mut targeted_engine = engine.clone();
    for action in &first.actions {
        let key = match action {
            UiAction::ButtonPressed { key } => key,
            _ => continue,
        };
        let node_id = targeted_engine
            .node_id_for_key(key)
            .expect("action key should resolve to node id");
        targeted_engine.invalidate_measure_subtree(node_id);
    }
    let mut full_engine = engine.clone();
    full_engine.invalidate_all_measure();

    let mut canvas = Canvas::new(240, 120);
    let mut layout = Layout::default();
    let input_idle = InputState {
        window_size: Size {
            width: 240,
            height: 120,
        },
        ..InputState::default()
    };

    let mut ui = Ui::new(&mut canvas, &input_idle, &mut ui_state, &mut layout, &theme);
    let _ = render_checked_with_engine(
        &spec,
        &mut ui,
        Point { x: 0, y: 0 },
        &mut targeted_engine,
    )
    .expect("targeted re-render should succeed");
    let targeted_after = targeted_engine.measure_cache_stats();

    let mut canvas = Canvas::new(240, 120);
    let mut layout = Layout::default();
    let mut ui = Ui::new(&mut canvas, &input_idle, &mut ui_state, &mut layout, &theme);
    let _ = render_checked_with_engine(
        &spec,
        &mut ui,
        Point { x: 0, y: 0 },
        &mut full_engine,
    )
    .expect("full re-render should succeed");
    let full_after = full_engine.measure_cache_stats();

    let targeted_miss_delta = targeted_after.misses.saturating_sub(baseline.misses);
    let full_miss_delta = full_after.misses.saturating_sub(baseline.misses);
    assert!(
        targeted_miss_delta < full_miss_delta,
        "targeted subtree invalidation should cause fewer misses than full invalidation"
    );
}

#[test]
fn layout_subtree_invalidation_reuses_measure_cache() {
    let spec = UiSpec::new(RootFrameSpec::new(
        "root",
        row_slots(vec![
            weighted_slot(
                panel(
                    "left-panel",
                    button("left-btn").control_size(Size {
                        width: 80,
                        height: 24,
                    }),
                )
                .pad_all(0),
                1,
            ),
            weighted_slot(
                panel(
                    "right-panel",
                    button("right-btn").control_size(Size {
                        width: 80,
                        height: 24,
                    }),
                )
                .pad_all(0),
                1,
            ),
        ])
        .pad_all(0),
    ));
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let idle_input = InputState {
        window_size: Size {
            width: 240,
            height: 120,
        },
        ..InputState::default()
    };
    let mut engine = LayoutEngineState::default();
    let mut canvas = Canvas::new(240, 120);
    let mut layout = Layout::default();
    let mut ui = Ui::new(&mut canvas, &idle_input, &mut ui_state, &mut layout, &theme);
    render_checked_with_engine(&spec, &mut ui, Point { x: 0, y: 0 }, &mut engine)
        .expect("initial render should succeed");
    let baseline = engine.measure_cache_stats();

    let left_id = engine
        .node_id_for_key("left-btn")
        .expect("left button id should resolve");

    let mut layout_only_engine = engine.clone();
    layout_only_engine.invalidate_layout_subtree(left_id);
    let mut canvas = Canvas::new(240, 120);
    let mut layout = Layout::default();
    let mut ui = Ui::new(&mut canvas, &idle_input, &mut ui_state, &mut layout, &theme);
    render_checked_with_engine(
        &spec,
        &mut ui,
        Point { x: 0, y: 0 },
        &mut layout_only_engine,
    )
    .expect("layout-only re-render should succeed");
    let layout_only_stats = layout_only_engine.measure_cache_stats();
    assert_eq!(
        layout_only_stats.misses, baseline.misses,
        "layout-only invalidation should not force measure cache misses"
    );
    assert!(layout_only_stats.hits > baseline.hits);

    let mut measure_engine = engine.clone();
    measure_engine.invalidate_measure_subtree(left_id);
    let mut canvas = Canvas::new(240, 120);
    let mut layout = Layout::default();
    let mut ui = Ui::new(&mut canvas, &idle_input, &mut ui_state, &mut layout, &theme);
    render_checked_with_engine(&spec, &mut ui, Point { x: 0, y: 0 }, &mut measure_engine)
        .expect("measure re-render should succeed");
    let measure_stats = measure_engine.measure_cache_stats();
    assert!(
        measure_stats.misses > baseline.misses,
        "measure invalidation should force at least one recompute"
    );
}

#[test]
fn layout_engine_resolves_node_ids_and_supports_subtree_measure_invalidation() {
    let spec = UiSpec::new(RootFrameSpec::new(
        "root",
        row_slots(vec![
            weighted_slot(panel("left", textbox("left")).pad_all(0), 1),
            weighted_slot(panel("right", textbox("right")).pad_all(0), 1),
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
    std::thread::Builder::new()
        .name("deep-layout-stress".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let mut node = textbox("leaf");
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
            ));
            let measured = measure_checked(&spec).expect("deep tree should measure");
            assert!(measured.width > 0 && measured.height > 0);
        })
        .expect("deep layout stress thread should start")
        .join()
        .expect("deep layout stress thread should finish");
}

#[test]
fn stress_large_slot_list_measures_without_gaps_or_panics() {
    let mut slots = Vec::new();
    for index in 0..1200 {
        slots.push(weighted_slot(
            panel(format!("item-{index}"), textbox("x")).pad_all(0),
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
            panel("a", textbox("A")).pad_all(0),
            panel("b", textbox("B")).pad_all(0),
            panel("c", textbox("C")).pad_all(0),
        ])
        .pad_all(4),
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
