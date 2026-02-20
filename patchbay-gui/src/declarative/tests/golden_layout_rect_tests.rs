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
    golden_panel_rect_by_key_optional(result, key)
        .unwrap_or_else(|| panic!("missing panel diagnostic for key `{key}`"))
}

fn golden_panel_rect_by_key_optional(result: &RenderResult, key: &str) -> Option<Rect> {
    let needle = format!("panel:{key}[");
    result
        .node_layout_diagnostics
        .iter()
        .find(|entry| entry.node_kind == LayoutNodeKind::Panel && entry.node_path.contains(&needle))
        .map(|entry| entry.resolved_rect)
}

fn golden_nested_flex_spec(size: Size) -> UiSpec {
    let top_row = row_slots(vec![
        weighted_slot(panel("left", textbox("L")).pad_all(0).fill(), 1),
        weighted_slot(panel("right", textbox("R")).pad_all(0).fill(), 2),
    ])
    .gap(10)
    .fill();

    let content = column_slots(vec![
        weighted_slot(top_row, 2),
        weighted_slot(panel("bottom", textbox("B")).pad_all(0).fill(), 1),
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
            panel("g1", textbox("1")).pad_all(0).fill(),
            panel("g2", textbox("2")).pad_all(0).fill(),
            panel("g3", textbox("3")).pad_all(0).fill(),
            panel("g4", textbox("4")).pad_all(0).fill(),
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

fn golden_stack_alignment_spec(size: Size) -> UiSpec {
    let content = Node::Stack(
        StackSpec::new(vec![
            panel(
                "stack-a",
                spacer(Size {
                    width: 20,
                    height: 10,
                }),
            )
            .pad_all(0),
            panel(
                "stack-b",
                spacer(Size {
                    width: 30,
                    height: 16,
                }),
            )
            .pad_all(0),
        ])
        .layout(ContainerLayout::fill())
        .align(SlotAlign::End, SlotAlign::Center),
    );
    UiSpec::new(
        RootFrameSpec::new("root", content)
            .layout(LayoutBox::fixed(size.width, size.height).max(size.width, size.height))
            .padding(0)
            .layout_diagnostics_mode(LayoutDiagnosticsMode::PerNode),
    )
}

fn golden_scroll_view_spec(size: Size, offset_x: i32, offset_y: i32) -> UiSpec {
    let content = Node::ScrollView(
        ScrollViewSpec::new(
            panel(
                "scroll-content",
                spacer(Size {
                    width: 180,
                    height: 140,
                }),
            )
            .pad_all(0),
        )
        .layout(ContainerLayout::fill())
        .offset_x(offset_x)
        .offset_y(offset_y),
    );
    UiSpec::new(
        RootFrameSpec::new("root", content)
            .layout(LayoutBox::fixed(size.width, size.height).max(size.width, size.height))
            .padding(0)
            .layout_diagnostics_mode(LayoutDiagnosticsMode::PerNode),
    )
}

fn golden_wrap_spec(size: Size) -> UiSpec {
    let content = Node::Wrap(
        WrapSpec::new(vec![
            panel(
                "w1",
                spacer(Size {
                    width: 30,
                    height: 10,
                }),
            )
            .pad_all(0),
            panel(
                "w2",
                spacer(Size {
                    width: 40,
                    height: 20,
                }),
            )
            .pad_all(0),
            panel(
                "w3",
                spacer(Size {
                    width: 50,
                    height: 15,
                }),
            )
            .pad_all(0),
            panel(
                "w4",
                spacer(Size {
                    width: 25,
                    height: 12,
                }),
            )
            .pad_all(0),
        ])
        .layout(ContainerLayout::fill())
        .column_gap(5)
        .row_gap(4)
        .justify(Justify::Start),
    );
    UiSpec::new(
        RootFrameSpec::new("root", content)
            .layout(LayoutBox::fixed(size.width, size.height).max(size.width, size.height))
            .padding(0)
            .layout_diagnostics_mode(LayoutDiagnosticsMode::PerNode),
    )
}

fn golden_switch_layout_spec(size: Size) -> UiSpec {
    let content = Node::SwitchLayout(
        SwitchLayoutSpec::new(
            vec![
                when_width_lt(
                    120,
                    panel(
                        "compact",
                        spacer(Size {
                            width: 20,
                            height: 10,
                        }),
                    )
                    .pad_all(0),
                ),
                when_width_between(
                    120,
                    220,
                    panel(
                        "medium",
                        spacer(Size {
                            width: 60,
                            height: 20,
                        }),
                    )
                    .pad_all(0),
                ),
            ],
            panel(
                "fallback",
                spacer(Size {
                    width: 90,
                    height: 30,
                }),
            )
            .pad_all(0),
        )
        .layout(ContainerLayout::fill()),
    );
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
fn fixed_grid_tracks_are_rejected_by_validation() {
    let size = Size {
        width: 210,
        height: 126,
    };
    let spec = golden_fixed_grid_spec(size);
    let input = InputState {
        window_size: size,
        ..InputState::default()
    };
    let mut canvas = Canvas::new(size.width, size.height);
    let mut layout = Layout::default();
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
    let error = render_checked(&spec, &mut ui, Point { x: 0, y: 0 })
        .expect_err("fixed grid tracks should be rejected");
    assert!(matches!(
        error,
        DeclarativeError::InvalidFixedGridTrack { axis } if axis == "columns"
    ));
}

#[test]
fn golden_stack_rects_match_aligned_overlay_positions() {
    let size = Size {
        width: 120,
        height: 80,
    };
    let spec = golden_stack_alignment_spec(size);
    let result = golden_render_spec_for_size(&spec, size);
    assert_eq!(
        golden_panel_rect_by_key(&result, "stack-a"),
        Rect {
            origin: Point { x: 100, y: 35 },
            size: Size {
                width: 20,
                height: 10,
            },
        }
    );
    assert_eq!(
        golden_panel_rect_by_key(&result, "stack-b"),
        Rect {
            origin: Point { x: 90, y: 32 },
            size: Size {
                width: 30,
                height: 16,
            },
        }
    );
    assert!(result.layout_diagnostics.is_empty());
}

#[test]
fn golden_scroll_view_rects_match_offset_and_clamp() {
    let cases = [
        (
            Size {
                width: 120,
                height: 60,
            },
            10,
            25,
            Rect {
                origin: Point { x: -10, y: -25 },
                size: Size {
                    width: 180,
                    height: 140,
                },
            },
        ),
        (
            Size {
                width: 120,
                height: 60,
            },
            999,
            999,
            Rect {
                origin: Point { x: -60, y: -80 },
                size: Size {
                    width: 180,
                    height: 140,
                },
            },
        ),
    ];
    for (size, offset_x, offset_y, expected) in cases {
        let spec = golden_scroll_view_spec(size, offset_x, offset_y);
        let result = golden_render_spec_for_size(&spec, size);
        assert_eq!(golden_panel_rect_by_key(&result, "scroll-content"), expected);
        assert!(result.layout_diagnostics.is_empty());
    }
}

#[test]
fn golden_wrap_rects_match_deterministic_row_breaks() {
    let size = Size {
        width: 100,
        height: 90,
    };
    let spec = golden_wrap_spec(size);
    let result = golden_render_spec_for_size(&spec, size);
    assert_eq!(
        golden_panel_rect_by_key(&result, "w1"),
        Rect {
            origin: Point { x: 0, y: 0 },
            size: Size {
                width: 30,
                height: 10,
            },
        }
    );
    assert_eq!(
        golden_panel_rect_by_key(&result, "w2"),
        Rect {
            origin: Point { x: 35, y: 0 },
            size: Size {
                width: 40,
                height: 20,
            },
        }
    );
    assert_eq!(
        golden_panel_rect_by_key(&result, "w3"),
        Rect {
            origin: Point { x: 0, y: 24 },
            size: Size {
                width: 50,
                height: 15,
            },
        }
    );
    assert_eq!(
        golden_panel_rect_by_key(&result, "w4"),
        Rect {
            origin: Point { x: 55, y: 24 },
            size: Size {
                width: 25,
                height: 12,
            },
        }
    );
    assert!(result.layout_diagnostics.is_empty());
}

#[test]
fn golden_switch_layout_rects_match_selected_case_for_breakpoints() {
    let cases = [
        (
            Size {
                width: 100,
                height: 60,
            },
            "compact",
            Rect {
                origin: Point { x: 0, y: 0 },
                size: Size {
                    width: 20,
                    height: 10,
                },
            },
        ),
        (
            Size {
                width: 150,
                height: 60,
            },
            "medium",
            Rect {
                origin: Point { x: 0, y: 0 },
                size: Size {
                    width: 60,
                    height: 20,
                },
            },
        ),
        (
            Size {
                width: 260,
                height: 60,
            },
            "fallback",
            Rect {
                origin: Point { x: 0, y: 0 },
                size: Size {
                    width: 90,
                    height: 30,
                },
            },
        ),
    ];
    for (size, expected_key, expected_rect) in cases {
        let spec = golden_switch_layout_spec(size);
        let result = golden_render_spec_for_size(&spec, size);
        assert_eq!(
            golden_panel_rect_by_key(&result, expected_key),
            expected_rect
        );
        for key in ["compact", "medium", "fallback"] {
            if key != expected_key {
                assert!(
                    golden_panel_rect_by_key_optional(&result, key).is_none(),
                    "unexpected panel `{key}` should not be rendered at width {}",
                    size.width
                );
            }
        }
        assert!(result.layout_diagnostics.is_empty());
    }
}
