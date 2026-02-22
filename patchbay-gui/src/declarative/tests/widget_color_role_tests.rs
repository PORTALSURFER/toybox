mod widget_color_role_tests {
    use super::super::*;
    use crate::canvas::Canvas;
    use crate::host::InputState;
    use crate::ui::{Layout, Theme, Ui, UiState};
    use crate::vector::scene::{KnobVisual, VectorCommand};

    #[test]
    fn knob_color_role_uses_resolved_base_variant() {
        let role = WidgetColorRole::Accent(AccentKey::Entity(17));
        let spec = UiSpec::new(RootFrameSpec::new(
            "root",
            Node::Absolute(
                AbsoluteSpec::new(vec![AbsoluteChild::new(
                    Point { x: 0, y: 0 },
                    Node::Knob(KnobSpec::new("k", 0.5, (0.0, 1.0)).color_role(role)),
                )])
                .layout(ContainerLayout::fill()),
            ),
        ));

        let mut canvas = Canvas::new(220, 160);
        let mut layout = Layout::default();
        let mut ui_state = UiState::default();
        let theme = Theme::default();
        let input = InputState::default();
        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let _ = render_checked(&spec, &mut ui, Point { x: 0, y: 0 }).expect("render should succeed");

        let visual = take_first_knob_visual(&mut ui);
        let resolver = DefaultWidgetColorResolver::new();
        let variants = resolver.resolve(
            role,
            WidgetColorContext {
                tokens: ThemeTokens::default(),
                disabled: false,
                focused: false,
            },
        );
        assert_eq!(visual.fill, variants.base);
    }

    #[test]
    fn knob_without_color_role_uses_legacy_theme_fill() {
        let spec = UiSpec::new(RootFrameSpec::new(
            "root",
            Node::Absolute(
                AbsoluteSpec::new(vec![AbsoluteChild::new(
                    Point { x: 0, y: 0 },
                    Node::Knob(KnobSpec::new("k", 0.5, (0.0, 1.0))),
                )])
                .layout(ContainerLayout::fill()),
            ),
        ));

        let mut canvas = Canvas::new(220, 160);
        let mut layout = Layout::default();
        let mut ui_state = UiState::default();
        let theme = Theme::default();
        let input = InputState::default();
        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let _ = render_checked(&spec, &mut ui, Point { x: 0, y: 0 }).expect("render should succeed");

        let visual = take_first_knob_visual(&mut ui);
        assert_eq!(visual.fill, theme.knob_fill);
    }

    #[test]
    fn focused_knob_uses_resolved_focus_ring_color() {
        let role = WidgetColorRole::Accent(AccentKey::Entity(3));
        let spec = UiSpec::new(RootFrameSpec::new(
            "root",
            Node::Absolute(
                AbsoluteSpec::new(vec![AbsoluteChild::new(
                    Point { x: 0, y: 0 },
                    Node::Knob(
                        KnobSpec::new("k", 0.5, (0.0, 1.0))
                            .color_role(role)
                            .focused(true),
                    ),
                )])
                .layout(ContainerLayout::fill()),
            ),
        ));

        let mut canvas = Canvas::new(220, 160);
        let mut layout = Layout::default();
        let mut ui_state = UiState::default();
        let theme = Theme::default();
        let input = InputState::default();
        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let _ = render_checked(&spec, &mut ui, Point { x: 0, y: 0 }).expect("render should succeed");

        let visual = take_first_knob_visual(&mut ui);
        let resolver = DefaultWidgetColorResolver::new();
        let variants = resolver.resolve(
            role,
            WidgetColorContext {
                tokens: ThemeTokens::default(),
                disabled: false,
                focused: true,
            },
        );
        assert_eq!(visual.outline, variants.focus_ring);
    }

    #[test]
    fn disabled_slider_does_not_emit_changed_actions() {
        let spec = UiSpec::new(RootFrameSpec::new(
            "root",
            panel(
                "panel",
                Node::Slider(
                    SliderSpec::new("mix", 0.25, (0.0, 1.0))
                        .disabled(true)
                        .color_role(WidgetColorRole::Accent(AccentKey::Entity(9)))
                        .control_size(Size {
                            width: 140,
                            height: 24,
                        }),
                ),
            )
            .pad_all(0),
        ));

        let mut canvas = Canvas::new(260, 140);
        let mut layout = Layout::default();
        let mut ui_state = UiState::default();
        let theme = Theme::default();
        let input = InputState {
            pointer_pos: Point { x: 40, y: 20 },
            wheel_delta: 1.0,
            ..InputState::default()
        };
        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let result =
            render_checked(&spec, &mut ui, Point { x: 0, y: 0 }).expect("render should succeed");

        assert!(
            !result
                .actions
                .iter()
                .any(|action| matches!(action, UiAction::SliderChanged { key, .. } if key == "mix")),
            "disabled slider should not emit change actions"
        );
    }

    #[test]
    fn identical_color_role_inputs_produce_identical_vector_commands() {
        let spec = UiSpec::new(RootFrameSpec::new(
            "root",
            Node::Absolute(
                AbsoluteSpec::new(vec![AbsoluteChild::new(
                    Point { x: 0, y: 0 },
                    Node::Knob(
                        KnobSpec::new("k", 0.5, (0.0, 1.0))
                            .color_role(WidgetColorRole::Accent(AccentKey::Entity(11)))
                            .focused(true),
                    ),
                )])
                .layout(ContainerLayout::fill()),
            ),
        ));

        let first = render_and_dump_commands(&spec);
        let second = render_and_dump_commands(&spec);
        assert_eq!(first, second);
    }

    fn render_and_dump_commands(spec: &UiSpec) -> String {
        let mut canvas = Canvas::new(220, 160);
        let mut layout = Layout::default();
        let mut ui_state = UiState::default();
        let theme = Theme::default();
        let input = InputState::default();
        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let _ = render_checked(spec, &mut ui, Point { x: 0, y: 0 }).expect("render should succeed");
        format!("{:?}", ui.take_vector_commands())
    }

    fn take_first_knob_visual(ui: &mut Ui<'_>) -> KnobVisual {
        ui.take_vector_commands()
            .into_iter()
            .find_map(|command| match command {
                VectorCommand::Knob(visual) => Some(visual),
                _ => None,
            })
            .expect("expected knob vector command")
    }
}
