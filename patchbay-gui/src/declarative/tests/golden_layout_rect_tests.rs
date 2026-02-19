fn golden_render_spec_for_size(spec: &UiSpec, size: Size) -> RenderResult {
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

fn golden_panel_rect_by_key(result: &RenderResult, key: &str) -> Rect {
    let needle = format!("panel:{key}[");
    result
        .node_layout_diagnostics
        .iter()
        .find(|entry| entry.node_kind == LayoutNodeKind::Panel && entry.node_path.contains(&needle))
        .map(|entry| entry.resolved_rect)
        .unwrap_or_else(|| panic!("missing panel diagnostic for key `{key}`"))
}

fn golden_nested_flex_spec(size: Size) -> UiSpec {
    let top_row = row_slots(vec![
        weighted_slot(panel("left", label("L")).pad_all(0).fill(), 1),
        weighted_slot(panel("right", label("R")).pad_all(0).fill(), 2),
    ])
    .gap(10)
    .fill();

    let content = column_slots(vec![
        weighted_slot(top_row, 2),
        weighted_slot(panel("bottom", label("B")).pad_all(0).fill(), 1),
    ])
    .gap(6)
    .fill();

    UiSpec::new(
        RootFrameSpec::new("root", content)
            .layout(LayoutBox::fixed(size.width, size.height).max(size.width, size.height))
            .padding(0)
            .layout_diagnostics_mode(LayoutDiagnosticsMode::PerNode),
    )
}

fn golden_fixed_grid_spec(size: Size) -> UiSpec {
    let template = GridTemplate::new(vec![TrackSize::Px(60), TrackSize::Px(140)])
        .rows(vec![TrackSize::Px(40), TrackSize::Px(80)])
        .gap_xy(10, 6);
    let content = grid(
        template,
        vec![
            panel("g1", label("1")).pad_all(0).fill(),
            panel("g2", label("2")).pad_all(0).fill(),
            panel("g3", label("3")).pad_all(0).fill(),
            panel("g4", label("4")).pad_all(0).fill(),
        ],
    )
    .fill();
    UiSpec::new(
        RootFrameSpec::new("root", content)
            .layout(LayoutBox::fixed(size.width, size.height).max(size.width, size.height))
            .padding(0)
            .layout_diagnostics_mode(LayoutDiagnosticsMode::PerNode),
    )
}

#[test]
fn golden_nested_flex_rects_match_expected_for_multiple_root_sizes() {
    let cases = [
        (
            Size {
                width: 310,
                height: 186,
            },
            Rect {
                origin: Point { x: 0, y: 0 },
                size: Size {
                    width: 100,
                    height: 120,
                },
            },
            Rect {
                origin: Point { x: 110, y: 0 },
                size: Size {
                    width: 200,
                    height: 120,
                },
            },
            Rect {
                origin: Point { x: 0, y: 126 },
                size: Size {
                    width: 310,
                    height: 60,
                },
            },
        ),
        (
            Size {
                width: 610,
                height: 366,
            },
            Rect {
                origin: Point { x: 0, y: 0 },
                size: Size {
                    width: 200,
                    height: 240,
                },
            },
            Rect {
                origin: Point { x: 210, y: 0 },
                size: Size {
                    width: 400,
                    height: 240,
                },
            },
            Rect {
                origin: Point { x: 0, y: 246 },
                size: Size {
                    width: 610,
                    height: 120,
                },
            },
        ),
    ];

    for (size, left, right, bottom) in cases {
        let spec = golden_nested_flex_spec(size);
        let result = golden_render_spec_for_size(&spec, size);
        assert_eq!(golden_panel_rect_by_key(&result, "left"), left);
        assert_eq!(golden_panel_rect_by_key(&result, "right"), right);
        assert_eq!(golden_panel_rect_by_key(&result, "bottom"), bottom);
        assert!(result.layout_diagnostics.is_empty());
    }
}

#[test]
fn golden_fixed_grid_rects_match_expected_cell_origins() {
    let size = Size {
        width: 210,
        height: 126,
    };
    let spec = golden_fixed_grid_spec(size);
    let result = golden_render_spec_for_size(&spec, size);
    assert_eq!(
        golden_panel_rect_by_key(&result, "g1"),
        Rect {
            origin: Point { x: 0, y: 0 },
            size: Size {
                width: 60,
                height: 40,
            },
        }
    );
    assert_eq!(
        golden_panel_rect_by_key(&result, "g2"),
        Rect {
            origin: Point { x: 70, y: 0 },
            size: Size {
                width: 140,
                height: 40,
            },
        }
    );
    assert_eq!(
        golden_panel_rect_by_key(&result, "g3"),
        Rect {
            origin: Point { x: 0, y: 46 },
            size: Size {
                width: 60,
                height: 80,
            },
        }
    );
    assert_eq!(
        golden_panel_rect_by_key(&result, "g4"),
        Rect {
            origin: Point { x: 70, y: 46 },
            size: Size {
                width: 140,
                height: 80,
            },
        }
    );
    assert!(result.layout_diagnostics.is_empty());
}
