fn render_with_engine(spec: &UiSpec, engine: &mut LayoutEngineState, size: Size) -> RenderResult {
    let input = InputState {
        window_size: size,
        ..InputState::default()
    };
    let mut canvas = Canvas::new(size.width, size.height);
    let mut layout = Layout::default();
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
    render_checked_with_engine(spec, &mut ui, Point { x: 0, y: 0 }, engine)
        .expect("render should succeed")
}

fn two_panel_spec() -> UiSpec {
    UiSpec::new(RootFrameSpec::new(
        "root",
        row_slots(vec![
            weighted_slot(panel("left", label("left")).pad_all(0), 1),
            weighted_slot(panel("right", label("right")).pad_all(0), 1),
        ])
        .pad_all(0),
    ))
}

fn left_only_spec() -> UiSpec {
    UiSpec::new(RootFrameSpec::new(
        "root",
        row_slots(vec![weighted_slot(panel("left", label("left")).pad_all(0), 1)]).pad_all(0),
    ))
}

fn right_only_spec() -> UiSpec {
    UiSpec::new(RootFrameSpec::new(
        "root",
        row_slots(vec![weighted_slot(panel("right", label("right")).pad_all(0), 1)]).pad_all(0),
    ))
}

#[test]
fn keyed_node_ids_do_not_alias_after_sibling_reindex() {
    let size = Size {
        width: 220,
        height: 120,
    };
    let initial = two_panel_spec();
    let updated = right_only_spec();
    let mut engine = LayoutEngineState::default();

    let _ = render_with_engine(&initial, &mut engine, size);
    let stale_left_id = engine
        .node_id_for_key("left")
        .expect("left panel must resolve to node id");
    let initial_right_id = engine
        .node_id_for_key("right")
        .expect("right panel must resolve to node id");

    let _ = render_with_engine(&updated, &mut engine, size);
    let reindexed_right_id = engine
        .node_id_for_key("right")
        .expect("right panel must still resolve to node id");
    assert_eq!(
        initial_right_id, reindexed_right_id,
        "keyed identity must be stable across sibling reindex"
    );

    engine.invalidate_layout_subtree(stale_left_id);
    assert_eq!(engine.structural_gaps().len(), 1);
    assert_eq!(
        engine.structural_gaps()[0],
        StructuralGapEntry {
            node_id: stale_left_id,
            reason: StructuralGapReason::MissingLayoutSubtreeInvalidationTarget,
        }
    );
}

#[test]
fn stale_layout_invalidation_records_structural_gap() {
    let size = Size {
        width: 220,
        height: 120,
    };
    let initial = two_panel_spec();
    let updated = left_only_spec();
    let mut engine = LayoutEngineState::default();

    let _ = render_with_engine(&initial, &mut engine, size);
    let stale_id = engine
        .node_id_for_key("right")
        .expect("right panel must resolve to node id");

    let _ = render_with_engine(&updated, &mut engine, size);
    engine.invalidate_layout_subtree(stale_id);

    assert_eq!(engine.structural_gaps().len(), 1);
    assert_eq!(
        engine.structural_gaps()[0],
        StructuralGapEntry {
            node_id: stale_id,
            reason: StructuralGapReason::MissingLayoutSubtreeInvalidationTarget,
        }
    );
}

#[test]
fn stale_measure_invalidation_emits_structural_gap_diagnostic() {
    let size = Size {
        width: 220,
        height: 120,
    };
    let initial = two_panel_spec();
    let updated = left_only_spec();
    let mut engine = LayoutEngineState::default();

    let _ = render_with_engine(&initial, &mut engine, size);
    let stale_id = engine
        .node_id_for_key("right")
        .expect("right panel must resolve to node id");

    let _ = render_with_engine(&updated, &mut engine, size);
    engine.invalidate_measure_subtree(stale_id);
    let result = render_with_engine(&updated, &mut engine, size);

    assert!(result.layout_diagnostics.iter().any(|diagnostic| {
        diagnostic.code == LayoutDiagnosticCode::StructuralGapDetected
            && diagnostic.container == LayoutContainerKind::RootFrame
            && diagnostic.message
                == StructuralGapReason::MissingMeasureSubtreeInvalidationTarget.diagnostic_message()
    }));
    assert_eq!(result.overflow.total, 0);
    assert!(engine.structural_gaps().is_empty());
}
