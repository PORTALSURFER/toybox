use super::*;
    use crate::canvas::Canvas;
    use crate::host::InputState;
use crate::ui::{Layout, Theme, UiState};

    #[cfg(feature = "layout-debug-borders")]
    #[test]
    fn debug_border_palette_is_available_when_feature_enabled() {
        assert_eq!(
            container_debug_border_color(ContainerKind::RootFrame, 0),
            None
        );
        let expected = Some(Color::rgb(245, 98, 98));
        for kind in [
            ContainerKind::Slot,
            ContainerKind::Panel,
            ContainerKind::PaddingBox,
            ContainerKind::AlignBox,
            ContainerKind::AspectBox,
            ContainerKind::Flex,
            ContainerKind::Grid,
            ContainerKind::Absolute,
            ContainerKind::Stack,
            ContainerKind::ScrollView,
            ContainerKind::Wrap,
            ContainerKind::SwitchLayout,
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
        assert!(!should_collect_container_debug_border_candidate_with_mode(
            ContainerKind::RootFrame,
            0,
            true,
            false
        ));
        assert!(!should_collect_container_debug_border_candidate_with_mode(
            ContainerKind::Flex,
            1,
            true,
            false
        ));
        assert!(should_collect_container_debug_border_candidate_with_mode(
            ContainerKind::Panel,
            2,
            true,
            false
        ));
        assert!(should_collect_container_debug_border_candidate_with_mode(
            ContainerKind::Slot,
            2,
            true,
            false
        ));
        assert!(!should_collect_container_debug_border_candidate_with_mode(
            ContainerKind::Panel,
            2,
            false,
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

    #[cfg(feature = "layout-debug-borders")]
    #[test]
    fn debug_border_env_toggle_controls_all_candidates_mode() {
        const ENV_VAR: &str = "PATCHBAY_DEBUG_ALL_LAYOUT_BORDERS";

        // SAFETY: tests are single-threaded and isolate temporary env overrides
        // within the same process for deterministic feature-flag behavior.
        unsafe { std::env::remove_var(ENV_VAR) };
        assert!(!should_draw_all_layout_debug_borders());
        assert!(!should_draw_all_layout_debug_borders_from_env(Err(
            std::env::VarError::NotPresent
        )));

        // SAFETY: see above.
        unsafe { std::env::set_var(ENV_VAR, "1") };
        assert!(should_draw_all_layout_debug_borders());
        assert!(should_draw_all_layout_debug_borders_from_env(Ok("1".to_string())));

        // SAFETY: see above.
        unsafe { std::env::set_var(ENV_VAR, "TRUE") };
        assert!(should_draw_all_layout_debug_borders());

        // SAFETY: see above.
        unsafe { std::env::set_var(ENV_VAR, "false") };
        assert!(!should_draw_all_layout_debug_borders());

        // SAFETY: see above.
        unsafe { std::env::remove_var(ENV_VAR) };
    }

    #[cfg(feature = "layout-debug-borders")]
    #[test]
    fn debug_border_all_mode_accepts_case_variants() {
        assert!(should_draw_all_layout_debug_borders_from_env(Ok("true".to_string())));
        assert!(should_draw_all_layout_debug_borders_from_env(Ok("On".to_string())));
        assert!(should_draw_all_layout_debug_borders_from_env(Ok("yes".to_string())));
        assert!(!should_draw_all_layout_debug_borders_from_env(Ok(
            "off".to_string()
        )));
        assert!(!should_draw_all_layout_debug_borders_from_env(Ok("0".to_string())));
    }

    #[cfg(feature = "layout-debug-borders")]
    #[test]
    fn debug_border_all_mode_collects_non_hovered_slot_candidates() {
        assert!(!should_collect_container_debug_border_candidate_with_mode(
            ContainerKind::Slot,
            3,
            false,
            false
        ));
        assert!(should_collect_container_debug_border_candidate_with_mode(
            ContainerKind::Slot,
            3,
            false,
            true
        ));
    }

    #[test]
    fn debug_border_draw_rect_preserves_exact_candidate_bounds() {
        let rect = Rect {
            origin: Point { x: 10, y: 20 },
            size: Size {
                width: 100,
                height: 50,
            },
        };
        let draw = debug_border_draw_rect(rect, 1).expect("draw rect");
        assert_eq!(draw, rect);
    }

    #[test]
    fn debug_border_draw_rect_rejects_zero_thickness() {
        let rect = Rect {
            origin: Point { x: 0, y: 0 },
            size: Size {
                width: 1,
                height: 1,
            },
        };
        assert!(debug_border_draw_rect(rect, 0).is_none());
    }

    #[test]
    fn debug_border_draw_rect_accepts_single_pixel_rectangles() {
        let rect = Rect {
            origin: Point { x: 2, y: 3 },
            size: Size {
                width: 1,
                height: 1,
            },
        };
        let draw = debug_border_draw_rect(rect, 1).expect("single-pixel draw rect");
        assert_eq!(draw, rect);
    }
