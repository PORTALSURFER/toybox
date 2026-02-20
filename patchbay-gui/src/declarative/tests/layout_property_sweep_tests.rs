fn render_property_spec_for_size(spec: &UiSpec, size: Size) -> RenderResult {
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

fn property_sweep_sizes() -> [Size; 4] {
    [
        Size {
            width: 240,
            height: 120,
        },
        Size {
            width: 333,
            height: 177,
        },
        Size {
            width: 512,
            height: 288,
        },
        Size {
            width: 803,
            height: 377,
        },
    ]
}

fn property_sweep_spec(size: Size) -> UiSpec {
    let top = row_slots(vec![
        weighted_slot(panel("left", textbox("L")).pad_all(2).fill(), 1),
        weighted_slot(panel("right", textbox("R")).pad_all(2).fill(), 1),
    ])
    .container_overflow(OverflowPolicy::Compress)
    .fill();

    let absolute = Node::Absolute(
        AbsoluteSpec::new(vec![AbsoluteChild::new(
            Point {
                x: size.width as i32 - 24,
                y: 8,
            },
            textbox("edge").widget_layout(LayoutBox::fixed(60, 20).max(60, 20)),
        )])
        .layout(ContainerLayout::fill())
        .overflow(OverflowPolicy::Compress),
    )
    .fill();

    let content = column_slots(vec![weighted_slot(top, 2), weighted_slot(absolute, 1)])
        .container_overflow(OverflowPolicy::Compress)
        .fill();

    UiSpec::new(
        RootFrameSpec::new("root", content)
            .layout(LayoutBox::fixed(size.width, size.height).max(size.width, size.height))
            .padding(0)
            .layout_diagnostics_mode(LayoutDiagnosticsMode::PerNode),
    )
}

fn summary_from_codes(diagnostics: &[LayoutDiagnostic]) -> LayoutOverflowSummary {
    let mut summary = LayoutOverflowSummary::default();
    for diagnostic in diagnostics {
        match diagnostic.code {
            LayoutDiagnosticCode::OverflowClipped => {
                summary.clipped += 1;
                summary.total += 1;
            }
            LayoutDiagnosticCode::OverflowSkippedDisjoint
            | LayoutDiagnosticCode::OverflowSkippedCollapsedBounds => {
                summary.skipped += 1;
                summary.total += 1;
            }
            LayoutDiagnosticCode::OverflowCompressed
            | LayoutDiagnosticCode::ScrollViewContentCompressed => {
                summary.compressed += 1;
                summary.total += 1;
            }
            LayoutDiagnosticCode::StructuralGapDetected => {}
        }
    }
    summary
}

fn rect_fits_within_bounds(rect: Rect, bounds: Rect) -> bool {
    let left = i64::from(rect.origin.x);
    let top = i64::from(rect.origin.y);
    let right = left + i64::from(rect.size.width);
    let bottom = top + i64::from(rect.size.height);

    let min_x = i64::from(bounds.origin.x);
    let min_y = i64::from(bounds.origin.y);
    let max_x = min_x + i64::from(bounds.size.width);
    let max_y = min_y + i64::from(bounds.size.height);

    left >= min_x && top >= min_y && right <= max_x && bottom <= max_y
}

#[test]
fn property_sweep_render_results_are_deterministic_for_same_input() {
    for size in property_sweep_sizes() {
        let spec = property_sweep_spec(size);
        let first = render_property_spec_for_size(&spec, size);
        let second = render_property_spec_for_size(&spec, size);
        assert_eq!(first, second, "render must be deterministic for {size:?}");
        assert!(first.resolved_scale.is_finite());
    }
}

#[test]
fn property_sweep_overflow_summary_matches_structured_diagnostics() {
    for size in property_sweep_sizes() {
        let spec = property_sweep_spec(size);
        let result = render_property_spec_for_size(&spec, size);
        let expected = summary_from_codes(&result.layout_diagnostics);
        assert!(result.overflow.total > 0);
        assert_eq!(
            result.overflow, expected,
            "overflow summary should match diagnostic codes for {size:?}"
        );
    }
}

#[test]
fn property_sweep_resolved_rects_stay_within_root_content_bounds() {
    for size in property_sweep_sizes() {
        let spec = property_sweep_spec(size);
        let result = render_property_spec_for_size(&spec, size);
        for entry in &result.node_layout_diagnostics {
            assert!(
                rect_fits_within_bounds(entry.resolved_rect, result.content_rect),
                "node {} escaped root content bounds for {size:?}",
                entry.node_path
            );
        }
    }
}

#[test]
fn property_sweep_rejects_inverted_slot_widget_bounds() {
    for size in property_sweep_sizes() {
        let spec = UiSpec::new(
            RootFrameSpec::new(
                "root",
                row_slots(vec![weighted_slot(textbox("x"), 1).width_bounds(Some(80), Some(16))]),
            )
            .layout(LayoutBox::fixed(size.width, size.height)),
        );
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
            .expect_err("inverted slot bounds must fail validation");
        assert!(matches!(
            error,
            DeclarativeError::InvalidLayoutBounds {
                node_kind,
                axis,
                min,
                max
            } if node_kind == "TextBox" && axis == "width" && min == 80 && max == 16
        ));
    }
}
