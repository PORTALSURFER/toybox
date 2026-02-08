    use super::*;
    use crate::canvas::Canvas;
    use crate::host::InputState;
    use crate::ui::{Layout, MainPalette, Theme, UiState};

    #[cfg(feature = "layout-debug-borders")]
    #[test]
    fn debug_border_palette_is_available_when_feature_enabled() {
        assert_eq!(
            container_debug_border_color(ContainerKind::RootFrame, 0),
            None
        );
        let expected = Some(Color::rgb(245, 98, 98));
        for kind in [
            ContainerKind::Panel,
            ContainerKind::Flex,
            ContainerKind::Grid,
            ContainerKind::Absolute,
        ] {
            assert_eq!(container_debug_border_color(kind, 0), expected);
            assert_eq!(container_debug_border_color(kind, 1), expected);
        }
    }

    #[cfg(not(feature = "layout-debug-borders"))]
    #[test]
    fn debug_border_palette_is_disabled_without_feature() {
        assert_eq!(
            container_debug_border_color(ContainerKind::RootFrame, 0),
            None
        );
    }

    #[test]
    fn debug_border_is_not_drawn_for_root_or_top_level_containers() {
        assert!(!should_draw_container_debug_border(
            ContainerKind::RootFrame,
            0,
            true
        ));
        assert!(!should_draw_container_debug_border(
            ContainerKind::Flex,
            1,
            true
        ));
        assert!(should_draw_container_debug_border(
            ContainerKind::Panel,
            2,
            true
        ));
        assert!(!should_draw_container_debug_border(
            ContainerKind::Panel,
            2,
            false
        ));
    }

    #[test]
    fn debug_border_selection_prefers_deepest_hovered_container() {
        let candidates = vec![
            DebugBorderCandidate {
                rect: Rect {
                    origin: Point { x: 0, y: 0 },
                    size: Size {
                        width: 600,
                        height: 360,
                    },
                },
                kind: ContainerKind::Flex,
                depth: 2,
            },
            DebugBorderCandidate {
                rect: Rect {
                    origin: Point { x: 300, y: 120 },
                    size: Size {
                        width: 280,
                        height: 180,
                    },
                },
                kind: ContainerKind::Grid,
                depth: 3,
            },
        ];

        let selected = select_container_debug_border_candidate(&candidates)
            .expect("expected a selected debug border candidate");
        assert_eq!(selected.depth, 3);
        assert_eq!(selected.kind, ContainerKind::Grid);
    }

    #[test]
    fn debug_border_selection_prefers_smaller_area_on_depth_tie() {
        let candidates = vec![
            DebugBorderCandidate {
                rect: Rect {
                    origin: Point { x: 0, y: 0 },
                    size: Size {
                        width: 320,
                        height: 240,
                    },
                },
                kind: ContainerKind::Panel,
                depth: 3,
            },
            DebugBorderCandidate {
                rect: Rect {
                    origin: Point { x: 120, y: 90 },
                    size: Size {
                        width: 140,
                        height: 80,
                    },
                },
                kind: ContainerKind::Absolute,
                depth: 3,
            },
        ];

        let selected = select_container_debug_border_candidate(&candidates)
            .expect("expected a selected debug border candidate");
        assert_eq!(selected.kind, ContainerKind::Absolute);
        assert_eq!(candidate_area(selected), 11_200);
    }

    #[test]
    fn debug_border_selection_prefers_latest_render_on_full_tie() {
        let candidates = vec![
            DebugBorderCandidate {
                rect: Rect {
                    origin: Point { x: 32, y: 32 },
                    size: Size {
                        width: 90,
                        height: 90,
                    },
                },
                kind: ContainerKind::Panel,
                depth: 4,
            },
            DebugBorderCandidate {
                rect: Rect {
                    origin: Point { x: 32, y: 32 },
                    size: Size {
                        width: 90,
                        height: 90,
                    },
                },
                kind: ContainerKind::Grid,
                depth: 4,
            },
        ];

        let selected = select_container_debug_border_candidate(&candidates)
            .expect("expected a selected debug border candidate");
        assert_eq!(selected.kind, ContainerKind::Grid);
    }

    #[test]
    fn debug_border_draw_rect_shrinks_max_edges_by_thickness() {
        let rect = Rect {
            origin: Point { x: 10, y: 20 },
            size: Size {
                width: 100,
                height: 50,
            },
        };
        let draw = debug_border_draw_rect(rect, 1).expect("draw rect");
        assert_eq!(draw.origin, rect.origin);
        assert_eq!(
            draw.size,
            Size {
                width: 99,
                height: 49
            }
        );
    }

    #[test]
    fn debug_border_draw_rect_rejects_too_small_rectangles() {
        let rect = Rect {
            origin: Point { x: 0, y: 0 },
            size: Size {
                width: 1,
                height: 1,
            },
        };
        assert!(debug_border_draw_rect(rect, 1).is_none());
    }

    #[test]
    fn clamp_size_to_available_caps_oversized_children() {
        let available = Size {
            width: 60,
            height: 40,
        };
        let resolved = Size {
            width: 80,
            height: 70,
        };
        let clamped = clamp_size_to_available(resolved, available);
        assert_eq!(clamped, available);
    }

    #[test]
    fn rejects_duplicate_widget_keys() {
        let spec = UiSpec::new(RootFrameSpec::new(
            "root",
            Node::column(vec![
                Node::Knob(KnobSpec::new("k", "A", 0.5, (0.0, 1.0))),
                Node::Knob(KnobSpec::new("k", "B", 0.5, (0.0, 1.0))),
            ]),
        ));
        let error = measure_checked(&spec).expect_err("expected duplicate key error");
        assert!(matches!(error, DeclarativeError::DuplicateNodeKey { .. }));
    }

    #[test]
    fn rejects_root_key_collision_with_child() {
        let spec = UiSpec::new(RootFrameSpec::new(
            "dup",
            Node::Panel(PanelSpec::new("dup", label("content"))),
        ));
        let error = measure_checked(&spec).expect_err("expected duplicate key error");
        assert!(matches!(error, DeclarativeError::DuplicateNodeKey { key } if key == "dup"));
    }

    #[test]
    fn measures_grid_from_template_and_children() {
        let grid = GridSpec::new(
            GridTemplate::new(vec![TrackSize::Px(32), TrackSize::Fr(1)]),
            vec![
                Node::Spacer(SpacerSpec::new(Size {
                    width: 10,
                    height: 12,
                })),
                Node::Spacer(SpacerSpec::new(Size {
                    width: 20,
                    height: 14,
                })),
            ],
        );
        let spec = UiSpec::new(RootFrameSpec::new("root", Node::Grid(grid)));
        let measured = measure_checked(&spec).expect("measurement should succeed");
        assert!(measured.width >= 32);
        assert!(measured.height >= 14);
    }

    #[test]
    fn grid_gap_xy_affects_measured_width_and_height_independently() {
        let grid = GridSpec::new(
            GridTemplate::columns_fr(2).gap_xy(3, 7),
            vec![
                spacer(Size {
                    width: 10,
                    height: 10,
                }),
                spacer(Size {
                    width: 10,
                    height: 10,
                }),
                spacer(Size {
                    width: 10,
                    height: 10,
                }),
                spacer(Size {
                    width: 10,
                    height: 10,
                }),
            ],
        );
        let spec = UiSpec::new(RootFrameSpec::new("root", Node::Grid(grid)).padding(0));
        let measured = measure_checked(&spec).expect("measurement should succeed");
        assert_eq!(measured.width, 23);
        assert_eq!(measured.height, 27);
    }

    #[test]
    fn grid_gap_sets_both_axes() {
        let template = GridTemplate::columns_fr(2).gap(9);
        assert_eq!(template.column_gap, 9);
        assert_eq!(template.row_gap, 9);
    }

    #[test]
    fn grid_template_defaults_to_tight_left_packing() {
        let template = GridTemplate::columns_fr(3);
        assert_eq!(template.column_gap, 0);
        assert_eq!(template.row_gap, 0);
        assert_eq!(template.justify_x, Justify::Start);
    }

    #[test]
    fn grid_template_justify_helpers_set_horizontal_distribution() {
        assert_eq!(
            GridTemplate::columns_fr(2).justify_center().justify_x,
            Justify::Center
        );
        assert_eq!(
            GridTemplate::columns_fr(2).justify_end().justify_x,
            Justify::End
        );
        assert_eq!(
            GridTemplate::columns_fr(2)
                .justify_space_between()
                .justify_x,
            Justify::SpaceBetween
        );
        assert_eq!(
            GridTemplate::columns_fr(2).justify_space_around().justify_x,
            Justify::SpaceAround
        );
        assert_eq!(
            GridTemplate::columns_fr(2).justify_space_evenly().justify_x,
            Justify::SpaceEvenly
        );
    }
