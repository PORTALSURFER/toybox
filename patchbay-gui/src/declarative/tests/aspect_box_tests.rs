mod aspect_box_tests {
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
    fn aspect_box_measure_expands_child_to_ratio_bounds() {
        let spec = UiSpec::new(
            RootFrameSpec::new(
                "root",
                aspect_box(
                    spacer(Size {
                        width: 30,
                        height: 30,
                    }),
                    AspectRatio::new(16, 9),
                ),
            )
            .padding(0),
        );
        let measured = measure_checked(&spec).expect("aspect-box measure should succeed");
        assert_eq!(
            measured,
            Size {
                width: 54,
                height: 30,
            }
        );
    }

    #[test]
    fn aspect_box_places_fill_child_in_fitted_aligned_rect() {
        let size = Size {
            width: 80,
            height: 30,
        };
        let spec = UiSpec::new(
            RootFrameSpec::new(
                "root",
                aspect_box(
                    panel("aspect-panel", label("A")).pad_all(0).fill(),
                    AspectRatio::new(16, 9),
                )
                .slot_align(SlotAlign::End, SlotAlign::Start)
                .fill(),
            )
            .padding(0)
            .layout(LayoutBox::fixed(size.width, size.height).max(size.width, size.height))
            .layout_diagnostics_mode(LayoutDiagnosticsMode::PerNode),
        );
        let result = render_for_size(&spec, size);
        assert_eq!(
            panel_rect_by_key(&result, "aspect-panel"),
            Rect {
                origin: Point { x: 27, y: 0 },
                size: Size {
                    width: 53,
                    height: 30,
                },
            }
        );
        assert!(result.layout_diagnostics.is_empty());
    }
}
