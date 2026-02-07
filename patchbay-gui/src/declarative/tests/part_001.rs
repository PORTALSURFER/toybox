
    #[test]
    fn rejects_invalid_knob_range() {
        let spec = UiSpec::new(RootFrameSpec::new(
            "root",
            panel(
                "panel",
                Node::Knob(KnobSpec::new("k", "Drive", 0.5, (1.0, 1.0))),
            ),
        ));
        let error = measure_checked(&spec).expect_err("expected invalid range error");
        assert!(matches!(
            error,
            DeclarativeError::InvalidValueRange { node_kind, .. } if node_kind == "Knob"
        ));
    }

    #[test]
    fn rejects_invalid_slider_range() {
        let spec = UiSpec::new(RootFrameSpec::new(
            "root",
            panel(
                "panel",
                Node::Slider(SliderSpec::new("s", "Shape", 0.5, (0.8, 0.2))),
            ),
        ));
        let error = measure_checked(&spec).expect_err("expected invalid range error");
        assert!(matches!(
            error,
            DeclarativeError::InvalidValueRange { node_kind, .. } if node_kind == "Slider"
        ));
    }

    #[test]
    fn rejects_out_of_range_control_value() {
        let spec = UiSpec::new(RootFrameSpec::new(
            "root",
            panel(
                "panel",
                Node::Slider(SliderSpec::new("s", "Shape", 1.5, (0.0, 1.0))),
            ),
        ));
        let error = measure_checked(&spec).expect_err("expected invalid control value");
        assert!(matches!(
            error,
            DeclarativeError::InvalidControlValue { node_kind, key, .. }
                if node_kind == "Slider" && key == "s"
        ));
    }

    #[test]
    fn rejects_dropdown_selection_out_of_bounds() {
        let spec = UiSpec::new(RootFrameSpec::new(
            "root",
            panel(
                "panel",
                Node::Dropdown(DropdownSpec::new(
                    "mode",
                    "Mode",
                    vec!["A".to_string(), "B".to_string()],
                    2,
                )),
            ),
        ));
        let error = measure_checked(&spec).expect_err("expected invalid dropdown selection");
        assert!(matches!(
            error,
            DeclarativeError::InvalidDropdownSelection {
                key,
                selected,
                options_len
            } if key == "mode" && selected == 2 && options_len == 2
        ));
    }

    #[test]
    fn rejects_zero_control_size() {
        let spec = UiSpec::new(RootFrameSpec::new(
            "root",
            panel(
                "panel",
                Node::Slider(
                    SliderSpec::new("s", "Shape", 0.5, (0.0, 1.0)).control_size(Size {
                        width: 0,
                        height: 24,
                    }),
                ),
            ),
        ));
        let error = measure_checked(&spec).expect_err("expected invalid control size");
        assert!(matches!(
            error,
            DeclarativeError::InvalidControlSize { node_kind, .. } if node_kind == "Slider"
        ));
    }

    #[test]
    fn fixed_root_layout_expands_to_intrinsic_content() {
        let spec = UiSpec::new(
            RootFrameSpec::new("root", panel("panel", label("VeryWideLabel")).pad_all(0))
                .padding(0)
                .layout(LayoutBox::fixed(1, 1)),
        );
        let measured = measure_checked(&spec).expect("measurement should succeed");
        let intrinsic = text_size(
            "VeryWideLabel",
            ThemeTokens::default().typography.text_scale,
        );
        assert_eq!(measured, intrinsic);
    }

    #[test]
    fn fixed_panel_layout_expands_to_intrinsic_content() {
        let spec = UiSpec::new(
            RootFrameSpec::new(
                "root",
                panel("panel", label("WidePanelText"))
                    .pad_all(0)
                    .layout(LayoutBox::fixed(2, 2)),
            )
            .padding(0),
        );
        let measured = measure_checked(&spec).expect("measurement should succeed");
        let intrinsic = text_size(
            "WidePanelText",
            ThemeTokens::default().typography.text_scale,
        );
        assert_eq!(measured, intrinsic);
    }

    #[test]
    fn explicit_max_still_caps_fixed_pixel_layout() {
        let spec = UiSpec::new(
            RootFrameSpec::new("root", panel("panel", label("VeryWideLabel")).pad_all(0))
                .padding(0)
                .layout(LayoutBox::fixed(1, 1).max(12, 12)),
        );
        let measured = measure_checked(&spec).expect("measurement should succeed");
        assert_eq!(measured.width, 12);
        assert_eq!(measured.height, 12);
    }

    #[test]
    fn fixed_absolute_layout_expands_to_positioned_child_bounds() {
        let spec = UiSpec::new(
            RootFrameSpec::new(
                "root",
                Node::Absolute(
                    AbsoluteSpec::new(vec![AbsoluteChild::new(
                        Point { x: 40, y: 30 },
                        spacer(Size {
                            width: 15,
                            height: 11,
                        }),
                    )])
                    .layout(LayoutBox::fixed(10, 10)),
                ),
            )
            .padding(0),
        );
        let measured = measure_checked(&spec).expect("measurement should succeed");
        assert_eq!(
            measured,
            Size {
                width: 55,
                height: 41,
            }
        );
    }

    #[test]
    fn default_control_tokens_use_half_knob_diameter() {
        assert_eq!(ThemeTokens::default().controls.knob_diameter, 32);
    }

    #[test]
    fn default_color_tokens_use_main_palette() {
        let palette = MainPalette::main();
        let tokens = ColorTokens::default();
        assert_eq!(tokens.background, palette.background_primary);
        assert_eq!(tokens.surface, palette.background_secondary);
        assert_eq!(tokens.border, palette.ui_secondary);
        assert_eq!(tokens.text, palette.text_primary);
        assert_eq!(tokens.accent, palette.accent_focus);
    }

    #[test]
    fn theme_tokens_from_palette_uses_palette_for_color_roles() {
        let palette = MainPalette::main();
        let tokens = ThemeTokens::from_palette(palette);
        assert_eq!(tokens.colors.background, palette.background_primary);
        assert_eq!(tokens.colors.surface, palette.background_secondary);
        assert_eq!(tokens.colors.border, palette.ui_secondary);
        assert_eq!(tokens.colors.text, palette.text_primary);
        assert_eq!(tokens.colors.accent, palette.accent_focus);
    }

    #[test]
    fn label_with_explicit_box_does_not_expand_root_width() {
        let spec = UiSpec::new(
            RootFrameSpec::new(
                "root",
                panel(
                    "panel",
                    label("VERY LONG LABEL THAT MUST NOT WIDEN THE WINDOW")
                        .layout(LayoutBox::fixed(64, 16).max(64, 16)),
                )
                .pad_all(0),
            )
            .padding(0),
        );
        let measured = measure_checked(&spec).expect("measurement should succeed");
        assert_eq!(
            measured,
            Size {
                width: 64,
                height: 16,
            }
        );
    }

    #[test]
    fn helper_layout_box_methods_apply_expected_constraints() {
        let layout = LayoutBox::auto()
            .fill_width()
            .fixed_height(24)
            .min(10, 20)
            .max(200, 30);
        assert_eq!(layout.width, Length::Fill(1));
        assert_eq!(layout.height, Length::Px(24));
        assert_eq!(layout.min_width, Some(10));
        assert_eq!(layout.min_height, Some(20));
        assert_eq!(layout.max_width, Some(200));
        assert_eq!(layout.max_height, Some(30));
    }

    #[test]
    fn helper_justify_methods_apply_expected_distribution_modes() {
        let flex = FlexSpec::row(vec![label("A"), label("B")]).justify_space_between();
        assert_eq!(flex.justify, Justify::SpaceBetween);

        let flex = FlexSpec::row(vec![label("A"), label("B")]).justify_space_around();
        assert_eq!(flex.justify, Justify::SpaceAround);

        let flex = FlexSpec::row(vec![label("A"), label("B")]).justify_space_evenly();
        assert_eq!(flex.justify, Justify::SpaceEvenly);
    }

    #[test]
    fn weighted_child_clamps_zero_weight_to_one() {
        let child = weighted(label("x"), 0);
        assert_eq!(child.size, SectionSize::Fraction(1));
    }

    #[test]
    fn column_sections_apply_weighted_height_fill() {
        let node = column_sections(vec![weighted(label("A"), 7), weighted(label("B"), 30)]);
        let Node::Grid(grid) = node else {
            panic!("expected grid-backed column section container");
        };
        assert_eq!(grid.layout, LayoutBox::fill());
        assert_eq!(grid.template.columns, vec![TrackSize::Fr(1)]);
        assert_eq!(
            grid.template.rows,
            vec![TrackSize::Percent(7), TrackSize::Percent(30)]
        );
        assert_eq!(grid.template.column_gap, 0);
        assert_eq!(grid.template.row_gap, 0);
        assert_eq!(grid.template.padding, EdgeInsets::all(0));
        assert_eq!(grid.template.justify_x, Justify::Start);
        assert_eq!(grid.children.len(), 2);

        let first = node_layout(&grid.children[0]);
        assert_eq!(first.width, Length::Fill(1));
        assert_eq!(first.height, Length::Fill(1));

        let second = node_layout(&grid.children[1]);
        assert_eq!(second.width, Length::Fill(1));
        assert_eq!(second.height, Length::Fill(1));
    }

    #[test]
    fn row_sections_apply_weighted_width_fill() {
        let node = row_sections(vec![weighted(label("L"), 70), weighted(label("R"), 30)]);
        let Node::Grid(grid) = node else {
            panic!("expected grid-backed row section container");
        };
        assert_eq!(grid.layout, LayoutBox::fill());
        assert_eq!(
            grid.template.columns,
            vec![TrackSize::Percent(70), TrackSize::Percent(30)]
        );
        assert_eq!(grid.template.rows, vec![TrackSize::Fr(1)]);
        assert_eq!(grid.template.column_gap, 0);
        assert_eq!(grid.template.row_gap, 0);
        assert_eq!(grid.template.padding, EdgeInsets::all(0));
        assert_eq!(grid.template.justify_x, Justify::Start);
        assert_eq!(grid.children.len(), 2);

        let left = node_layout(&grid.children[0]);
        assert_eq!(left.width, Length::Fill(1));
        assert_eq!(left.height, Length::Fill(1));

        let right = node_layout(&grid.children[1]);
        assert_eq!(right.width, Length::Fill(1));
        assert_eq!(right.height, Length::Fill(1));
    }
