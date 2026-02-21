
    #[test]
    fn render_region_draw_commands_and_emit_interaction_action() {
        let mut canvas = Canvas::new(160, 120);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState {
            pointer_pos: Point { x: 12, y: 12 },
            mouse_pressed: true,
            ..InputState::default()
        };
        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);

        let region = RegionSpec::new("plot").layout(LayoutBox::fixed(64, 48).max(64, 48))
        .draw_commands(vec![
            DrawCommand::FillRect {
                rect: Rect {
                    origin: Point { x: 0, y: 0 },
                    size: Size {
                        width: 64,
                        height: 48,
                    },
                },
                color: Color::rgb(20, 30, 40),
            },
            DrawCommand::Line {
                start: Point { x: 4, y: 4 },
                end: Point { x: 20, y: 20 },
                color: Color::rgb(230, 230, 230),
            },
            DrawCommand::Text {
                origin: Point { x: 2, y: 2 },
                text: "Hi".to_string(),
                color: Color::rgb(200, 200, 210),
                scale: 1,
            },
        ]);
        let spec = UiSpec::new(RootFrameSpec::new(
            "root",
            panel("panel", Node::Region(region)).pad_all(0),
        ));

        let result =
            render_checked(&spec, &mut ui, Point { x: 0, y: 0 }).expect("render should succeed");
        assert!(result.actions.iter().any(|action| {
            matches!(
                action,
                UiAction::RegionHover {
                    key,
                    hovered,
                    local_pointer
                } if key == "plot"
                    && *hovered
                    && local_pointer.x >= 0
                    && local_pointer.y >= 0
                    && local_pointer.x < 64
                    && local_pointer.y < 48
            )
        }));
        assert!(result.actions.iter().any(|action| {
            matches!(
                action,
                UiAction::RegionInteracted {
                    key,
                    kind,
                    local_pointer,
                    ..
                } if key == "plot"
                    && *kind == RegionInteractionKind::Pressed
                    && local_pointer.x >= 0
                    && local_pointer.y >= 0
                    && local_pointer.x < 64
                    && local_pointer.y < 48
            )
        }));
    }

    #[test]
    fn render_region_emits_hover_false_when_pointer_is_outside() {
        let mut canvas = Canvas::new(160, 120);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState {
            pointer_pos: Point { x: 150, y: 110 },
            ..InputState::default()
        };
        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);

        let spec = UiSpec::new(RootFrameSpec::new(
            "root",
            panel(
                "panel",
                Node::Region(RegionSpec::new("plot").layout(LayoutBox::fixed(64, 48).max(64, 48))),
            )
            .pad_all(0),
        ));

        let result =
            render_checked(&spec, &mut ui, Point { x: 0, y: 0 }).expect("render should succeed");
        assert!(result.actions.iter().any(|action| {
            matches!(
                action,
                UiAction::RegionHover { key, hovered, .. } if key == "plot" && !hovered
            )
        }));
    }

    #[test]
    fn render_region_emits_hover_false_when_pointer_is_outside_window() {
        let mut canvas = Canvas::new(160, 120);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState {
            pointer_pos: Point { x: 12, y: 12 },
            pointer_in_window: false,
            ..InputState::default()
        };
        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);

        let spec = UiSpec::new(RootFrameSpec::new(
            "root",
            panel(
                "panel",
                Node::Region(RegionSpec::new("plot").layout(LayoutBox::fixed(64, 48).max(64, 48))),
            )
            .pad_all(0),
        ));

        let result =
            render_checked(&spec, &mut ui, Point { x: 0, y: 0 }).expect("render should succeed");
        assert!(result.actions.iter().any(|action| {
            matches!(
                action,
                UiAction::RegionHover { key, hovered, .. } if key == "plot" && !hovered
            )
        }));
    }

    #[test]
    fn panel_children_respect_grid_cell_origins() {
        let root_size = Size {
            width: 200,
            height: 100,
        };
        let content = row_slots(vec![
            weighted_slot(
                panel(
                    "left-panel",
                    Node::Region(RegionSpec::new("left-region").layout(LayoutBox::fixed(100, 100).max(100, 100))),
                )
                .pad_all(0)
                .background(Color::rgb(15, 20, 25))
                .outline(Color::rgb(15, 20, 25)),
                50,
            ),
            weighted_slot(
                panel(
                    "right-panel",
                    Node::Region(RegionSpec::new("right-region").layout(LayoutBox::fixed(100, 100).max(100, 100))),
                )
                .pad_all(0)
                .background(Color::rgb(30, 35, 40))
                .outline(Color::rgb(30, 35, 40)),
                50,
            ),
        ]);
        let spec = UiSpec::new(root_frame_sized("root", content, root_size).padding(0));

        let mut canvas = Canvas::new(root_size.width, root_size.height);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState {
            pointer_pos: Point { x: 150, y: 50 },
            ..InputState::default()
        };
        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let result =
            render_checked(&spec, &mut ui, Point { x: 0, y: 0 }).expect("render should succeed");

        assert!(result.actions.iter().any(|action| {
            matches!(
                action,
                UiAction::RegionHover { key, hovered, .. }
                    if key == "right-region" && *hovered
            )
        }));
        assert!(result.actions.iter().any(|action| {
            matches!(
                action,
                UiAction::RegionHover { key, hovered, .. }
                    if key == "left-region" && !*hovered
            )
        }));
    }
