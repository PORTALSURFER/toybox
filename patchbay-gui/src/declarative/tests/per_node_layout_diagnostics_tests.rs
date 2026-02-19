fn render_for_size(spec: &UiSpec, size: Size) -> RenderResult {
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

#[test]
fn events_only_mode_keeps_per_node_diagnostics_empty() {
    let spec = UiSpec::new(
        RootFrameSpec::new("root", panel("main", textbox("x")))
            .layout(LayoutBox::fixed(200, 100).max(200, 100))
            .padding(0),
    );
    let result = render_for_size(
        &spec,
        Size {
            width: 200,
            height: 100,
        },
    );
    assert!(result.node_layout_diagnostics.is_empty());
}

#[test]
fn per_node_mode_emits_deterministic_geometry_entries() {
    let spec = UiSpec::new(
        RootFrameSpec::new(
            "root",
            column(vec![
                panel("header", textbox("Header")).pad_all(0),
                region(
                    "plot",
                    Size {
                        width: 80,
                        height: 30,
                    },
                ),
            ]),
        )
        .layout(LayoutBox::fixed(240, 120).max(240, 120))
        .padding(0)
        .layout_diagnostics_mode(LayoutDiagnosticsMode::PerNode),
    );

    let size = Size {
        width: 240,
        height: 120,
    };
    let first = render_for_size(&spec, size);
    let second = render_for_size(&spec, size);
    assert_eq!(first.node_layout_diagnostics, second.node_layout_diagnostics);
    assert!(!first.node_layout_diagnostics.is_empty());
    assert!(
        first
            .node_layout_diagnostics
            .iter()
            .any(|entry| entry.node_kind == LayoutNodeKind::Panel)
    );
    assert!(
        first.node_layout_diagnostics.iter().all(|entry| {
            entry
                .reasons
                .contains(&LayoutNodeDiagnosticReason::Measured)
                && entry
                    .reasons
                    .contains(&LayoutNodeDiagnosticReason::Resolved)
        })
    );
}

#[test]
fn switch_layout_records_case_and_fallback_reasons() {
    let content = switch_layout(
        vec![when_width_ge(500, panel("wide", textbox("Wide")).fill())],
        panel("compact", textbox("Compact")).fill(),
    )
    .fill();

    let compact_spec = UiSpec::new(
        root_frame_sized(
            "root",
            content.clone(),
            Size {
                width: 420,
                height: 120,
            },
        )
        .padding(0)
        .layout_diagnostics_mode(LayoutDiagnosticsMode::PerNode),
    );
    let compact = render_for_size(
        &compact_spec,
        Size {
            width: 420,
            height: 120,
        },
    );
    assert!(compact.node_layout_diagnostics.iter().any(|entry| {
        entry
            .reasons
            .contains(&LayoutNodeDiagnosticReason::FallbackSelected)
    }));

    let wide_spec = UiSpec::new(
        root_frame_sized(
            "root",
            content,
            Size {
                width: 700,
                height: 120,
            },
        )
        .padding(0)
        .layout_diagnostics_mode(LayoutDiagnosticsMode::PerNode),
    );
    let wide = render_for_size(
        &wide_spec,
        Size {
            width: 700,
            height: 120,
        },
    );
    assert!(wide.node_layout_diagnostics.iter().any(|entry| {
        entry
            .reasons
            .contains(&LayoutNodeDiagnosticReason::SwitchCaseSelected)
    }));
}

#[test]
fn per_node_reasons_include_overflow_policy_adjustments() {
    let make_spec = |overflow_policy| {
        UiSpec::new(
            RootFrameSpec::new(
                "root",
                Node::Absolute(
                    AbsoluteSpec::new(vec![AbsoluteChild::new(
                        Point { x: 90, y: 4 },
                        region(
                            "edge",
                            Size {
                                width: 30,
                                height: 20,
                            },
                        ),
                    )])
                    .layout(ContainerLayout::fill())
                    .overflow(overflow_policy),
                ),
            )
            .layout(LayoutBox::fixed(100, 40).max(100, 40))
            .padding(0)
            .layout_diagnostics_mode(LayoutDiagnosticsMode::PerNode),
        )
    };

    let clip = render_for_size(
        &make_spec(OverflowPolicy::Clip),
        Size {
            width: 100,
            height: 40,
        },
    );
    assert!(clip.node_layout_diagnostics.iter().any(|entry| entry
        .reasons
        .contains(&LayoutNodeDiagnosticReason::OverflowClipped)));

    let compress = render_for_size(
        &make_spec(OverflowPolicy::Compress),
        Size {
            width: 100,
            height: 40,
        },
    );
    assert!(compress.node_layout_diagnostics.iter().any(|entry| entry
        .reasons
        .contains(&LayoutNodeDiagnosticReason::OverflowCompressed)));
}
