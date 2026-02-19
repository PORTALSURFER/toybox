fn render_extreme_spec_for_size(spec: &UiSpec, size: Size) -> RenderResult {
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

fn panel_slot_series(count: usize, key_prefix: &str, cell_size: Size) -> Vec<Slot> {
    let mut slots = Vec::with_capacity(count);
    for index in 0..count {
        slots.push(weighted_slot(
            panel(
                format!("{key_prefix}-{index}"),
                spacer(Size {
                    width: cell_size.width,
                    height: cell_size.height,
                }),
            )
            .pad_all(0),
            1,
        ));
    }
    slots
}

fn assert_render_is_deterministic(spec: &UiSpec, size: Size) -> RenderResult {
    let first = render_extreme_spec_for_size(spec, size);
    let second = render_extreme_spec_for_size(spec, size);
    assert_eq!(first, second);
    assert!(first.resolved_scale.is_finite());
    assert!(first.content_rect.size.width > 0);
    assert!(first.content_rect.size.height > 0);
    first
}

#[test]
fn stress_depth_500_panel_nesting_fails_fast_with_depth_guard() {
    let mut node = textbox("leaf");
    for index in 0..500 {
        node = panel(format!("layer-{index}"), node).pad_all(0);
    }

    let size = Size {
        width: 480,
        height: 270,
    };
    let spec = UiSpec::new(root_frame_sized("root", node, size));
    let error = measure_checked(&spec).expect_err("deep tree should fail fast");
    assert!(matches!(
        error,
        DeclarativeError::TreeDepthExceeded {
            max_depth,
            actual_depth,
            ..
        } if actual_depth > max_depth
    ));
}

#[test]
fn stress_10k_slot_row_is_deterministic_and_stable() {
    let slots = panel_slot_series(
        10_000,
        "item",
        Size {
            width: 1,
            height: 1,
        },
    );
    let size = Size {
        width: 10_000,
        height: 96,
    };
    let content = row_slots(slots).pad_all(0);
    let spec = UiSpec::new(root_frame_sized("root", content, size));

    let measured = measure_checked(&spec).expect("large slot list should measure");
    assert!(measured.width >= size.width);
    assert!(measured.height >= size.height);

    let _ = assert_render_is_deterministic(&spec, size);
}

#[test]
fn stress_10k_scroll_column_is_deterministic_and_stable() {
    let slots = panel_slot_series(
        10_000,
        "row",
        Size {
            width: 8,
            height: 2,
        },
    );
    let content = scroll_view(column_slots(slots).gap(0).pad_all(0)).fill();
    let size = Size {
        width: 640,
        height: 200,
    };
    let spec = UiSpec::new(root_frame_sized("root", content, size));
    let measured = measure_checked(&spec).expect("scroll content should measure");
    assert!(measured.width > 0);
    assert!(measured.height > 0);
    let result = assert_render_is_deterministic(&spec, size);
    assert!(result.actions.is_empty());
}
