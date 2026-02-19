mod single_slot_container_tests {
    use super::super::*;
    use crate::canvas::Canvas;
    use crate::host::InputState;
    use crate::ui::{Layout, Theme, UiState};

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

    fn panel_rect_by_key(result: &RenderResult, key: &str) -> Rect {
        let needle = format!("panel:{key}[");
        result
            .node_layout_diagnostics
            .iter()
            .find(|entry| entry.node_kind == LayoutNodeKind::Panel && entry.node_path.contains(&needle))
            .map(|entry| entry.resolved_rect)
            .unwrap_or_else(|| panic!("missing panel diagnostic for key `{key}`"))
    }

    #[test]
    fn padding_box_measure_adds_padding_around_content() {
        let spec = UiSpec::new(
            RootFrameSpec::new(
                "root",
                padding_box(spacer(Size {
                    width: 30,
                    height: 10,
                }))
                .pad_xy(7, 9),
            )
            .padding(0),
        );
        let measured = measure_checked(&spec).expect("padding-box measure should succeed");
        assert_eq!(
            measured,
            Size {
                width: 44,
                height: 28,
            }
        );
    }

    #[test]
    fn align_box_measure_tracks_intrinsic_child_size() {
        let spec = UiSpec::new(
            RootFrameSpec::new(
                "root",
                align_box(spacer(Size {
                    width: 24,
                    height: 12,
                }))
                .slot_align(SlotAlign::End, SlotAlign::Center),
            )
            .padding(0),
        );
        let measured = measure_checked(&spec).expect("align-box measure should succeed");
        assert_eq!(
            measured,
            Size {
                width: 24,
                height: 12,
            }
        );
    }

    #[test]
    fn padding_box_places_child_at_inset_origin() {
        let size = Size {
            width: 120,
            height: 80,
        };
        let spec = UiSpec::new(
            RootFrameSpec::new(
                "root",
                padding_box(
                    panel(
                        "inset",
                        spacer(Size {
                            width: 30,
                            height: 10,
                        }),
                    )
                    .pad_all(0),
                )
                .pad_xy(7, 9)
                .fill(),
            )
            .padding(0)
            .layout(LayoutBox::fixed(size.width, size.height).max(size.width, size.height))
            .layout_diagnostics_mode(LayoutDiagnosticsMode::PerNode),
        );
        let result = render_for_size(&spec, size);
        assert_eq!(
            panel_rect_by_key(&result, "inset"),
            Rect {
                origin: Point { x: 7, y: 9 },
                size: Size {
                    width: 30,
                    height: 10,
                },
            }
        );
    }

    #[test]
    fn align_box_places_child_using_slot_alignment() {
        let size = Size {
            width: 120,
            height: 80,
        };
        let spec = UiSpec::new(
            RootFrameSpec::new(
                "root",
                align_box(
                    panel(
                        "aligned",
                        spacer(Size {
                            width: 20,
                            height: 8,
                        }),
                    )
                    .pad_all(0),
                )
                .slot_align(SlotAlign::End, SlotAlign::Center)
                .fill(),
            )
            .padding(0)
            .layout(LayoutBox::fixed(size.width, size.height).max(size.width, size.height))
            .layout_diagnostics_mode(LayoutDiagnosticsMode::PerNode),
        );
        let result = render_for_size(&spec, size);
        assert_eq!(
            panel_rect_by_key(&result, "aligned"),
            Rect {
                origin: Point { x: 100, y: 36 },
                size: Size {
                    width: 20,
                    height: 8,
                },
            }
        );
    }
}
