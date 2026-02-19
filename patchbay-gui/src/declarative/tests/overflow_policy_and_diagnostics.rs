fn render_with_input(spec: &UiSpec, input: InputState) -> RenderResult {
    let mut canvas = Canvas::new(input.window_size.width, input.window_size.height);
    let mut layout = Layout::default();
    let theme = Theme::default();
    let mut ui_state = UiState::default();
    let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
    render_checked(spec, &mut ui, Point { x: 0, y: 0 }).expect("render should succeed")
}

#[test]
fn absolute_clip_policy_drops_disjoint_child_and_emits_diagnostic() {
    let size = Size {
        width: 100,
        height: 40,
    };
    let content = Node::Absolute(
        AbsoluteSpec::new(vec![AbsoluteChild::new(
            Point { x: 150, y: 0 },
            region(
                "out",
                Size {
                    width: 30,
                    height: 20,
                },
            ),
        )])
        .layout(ContainerLayout::fill())
        .overflow(OverflowPolicy::Clip),
    );
    let spec = UiSpec::new(
        RootFrameSpec::new("root", content)
            .layout(LayoutBox::fixed(100, 40).max(100, 40))
            .padding(0),
    );

    let result = render_with_input(
        &spec,
        InputState {
            window_size: size,
            pointer_pos: Point { x: 70, y: 10 },
            ..InputState::default()
        },
    );

    assert!(
        result
            .actions
            .iter()
            .all(|action| !matches!(action, UiAction::RegionHover { key, .. } if key == "out"))
    );
    assert!(result.layout_diagnostics.iter().any(|diagnostic| {
        diagnostic.container == LayoutContainerKind::Absolute
            && diagnostic.message == "layout rect does not intersect container bounds"
    }));
}

#[test]
fn absolute_compress_policy_keeps_disjoint_child_visible_and_emits_diagnostic() {
    let size = Size {
        width: 100,
        height: 40,
    };
    let content = Node::Absolute(
        AbsoluteSpec::new(vec![AbsoluteChild::new(
            Point { x: 150, y: 0 },
            region(
                "out",
                Size {
                    width: 30,
                    height: 20,
                },
            ),
        )])
        .layout(ContainerLayout::fill())
        .overflow(OverflowPolicy::Compress),
    );
    let spec = UiSpec::new(
        RootFrameSpec::new("root", content)
            .layout(LayoutBox::fixed(100, 40).max(100, 40))
            .padding(0),
    );

    let result = render_with_input(
        &spec,
        InputState {
            window_size: size,
            pointer_pos: Point { x: 95, y: 5 },
            ..InputState::default()
        },
    );

    assert!(result.actions.iter().any(|action| {
        matches!(action, UiAction::RegionHover { key, .. } if key == "out")
    }));
    assert!(result.layout_diagnostics.iter().any(|diagnostic| {
        diagnostic.container == LayoutContainerKind::Absolute
            && diagnostic.message == "layout rect compressed to fit container bounds"
    }));
}
