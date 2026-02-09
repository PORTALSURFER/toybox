
    #[test]
    fn resolve_grid_axis_distributes_fr_remainder_without_slack() {
        let row_heights = resolve_grid_axis(GridAxisResolveRequest {
            tracks: &[TrackSize::Fr(7), TrackSize::Fr(63), TrackSize::Fr(30)],
            columns: 1,
            rows: 3,
            gap: 0,
            available: 259,
            is_columns: false,
            intrinsic: &[Size {
                width: 0,
                height: 0,
            }; 3],
        });
        assert_eq!(row_heights, vec![18, 163, 78]);
        assert_eq!(row_heights.iter().sum::<u32>(), 259);

        let column_widths = resolve_grid_axis(GridAxisResolveRequest {
            tracks: &[TrackSize::Fr(70), TrackSize::Fr(30)],
            columns: 2,
            rows: 1,
            gap: 0,
            available: 799,
            is_columns: true,
            intrinsic: &[Size {
                width: 0,
                height: 0,
            }; 2],
        });
        assert_eq!(column_widths, vec![559, 240]);
        assert_eq!(column_widths.iter().sum::<u32>(), 799);
    }

    #[test]
    fn root_frame_sized_uses_window_size_with_minimum_floor() {
        let root = root_frame_sized(
            "root",
            label("x"),
            Size {
                width: 420,
                height: 258,
            },
            Size {
                width: 360,
                height: 400,
            },
        );
        assert_eq!(
            root.layout,
            LayoutBox::fixed(420, 400).max(420, 400),
            "root should clamp to min width and use host-provided height"
        );
        assert_eq!(root.scale_mode, RootScaleMode::None);
        assert_eq!(root.design_size, None);
    }

    #[test]
    fn nested_section_helpers_measure_successfully() {
        let controls = row_sections(vec![
            weighted(panel("left", label("Knobs")).pad_all(0), 70),
            weighted(panel("right", label("Dropdowns")).pad_all(0), 30),
        ]);
        let content = column_sections(vec![
            weighted(panel("header", label("Header")).pad_all(0), 7),
            weighted(panel("curve", label("Curve")).pad_all(0), 63),
            weighted(panel("controls", controls).pad_all(0), 30),
        ]);
        let spec = UiSpec::new(
            root_frame_sized(
                "root",
                content,
                Size {
                    width: 420,
                    height: 258,
                },
                Size {
                    width: 840,
                    height: 516,
                },
            )
            .padding(0),
        );

        let measured = measure_checked(&spec).expect("nested section helpers should validate");
        assert!(measured.width >= 420);
        assert!(measured.height >= 258);
    }

    #[test]
    fn justify_weighting_and_distribution_cover_new_modes() {
        let between = justify_space_weights(Justify::SpaceBetween, 3);
        assert_eq!(between, vec![0, 1, 1, 0]);

        let around = justify_space_weights(Justify::SpaceAround, 3);
        assert_eq!(around, vec![1, 2, 2, 1]);

        let evenly = justify_space_weights(Justify::SpaceEvenly, 3);
        assert_eq!(evenly, vec![1, 1, 1, 1]);

        let distributed = distribute_space(7, &[1, 2, 0, 1]);
        assert_eq!(distributed.iter().sum::<i32>(), 7);
        assert_eq!(distributed[2], 0);
    }

    #[test]
    fn node_layout_helpers_apply_constraints_when_supported() {
        let node = panel("main", label("x")).fill_width();
        match node {
            Node::Panel(panel) => {
                assert_eq!(panel.layout.width, Length::Fill(1));
                assert_eq!(panel.layout.height, Length::Auto);
            }
            _ => panic!("expected panel node"),
        }

        let spacer_node = spacer(Size {
            width: 10,
            height: 10,
        })
        .fill();
        assert!(matches!(spacer_node, Node::Spacer(_)));
    }

    #[test]
    fn node_fluent_helpers_apply_container_and_style_fields() {
        let panel_node = panel("main", label("x"))
            .title("Main")
            .pad_all(14)
            .background(Color::rgb(12, 16, 22))
            .outline(Color::rgb(44, 52, 68));
        match panel_node {
            Node::Panel(panel) => {
                assert_eq!(panel.title.as_deref(), Some("Main"));
                assert_eq!(panel.padding, 14);
                assert_eq!(panel.background, Some(Color::rgb(12, 16, 22)));
                assert_eq!(panel.outline, Some(Color::rgb(44, 52, 68)));
            }
            _ => panic!("expected panel node"),
        }

        let row_node = row(vec![label("a"), label("b")])
            .gap(6)
            .pad_xy(10, 8)
            .align_center()
            .justify_space_between();
        match row_node {
            Node::Row(flex) => {
                assert_eq!(flex.gap, 6);
                assert_eq!(flex.padding, EdgeInsets::symmetric(10, 8));
                assert_eq!(flex.align, Align::Center);
                assert_eq!(flex.justify, Justify::SpaceBetween);
            }
            _ => panic!("expected row node"),
        }

        let grid_node = grid(
            GridTemplate::columns_fr(2),
            vec![spacer(Size {
                width: 8,
                height: 8,
            })],
        )
        .gap_xy(3, 9)
        .pad_all(5);
        match grid_node {
            Node::Grid(grid) => {
                assert_eq!(grid.template.column_gap, 3);
                assert_eq!(grid.template.row_gap, 9);
                assert_eq!(grid.template.padding, EdgeInsets::all(5));
            }
            _ => panic!("expected grid node"),
        }

        let label_node = label("name").text_color(Color::rgb(200, 180, 90));
        match label_node {
            Node::Label(label) => assert_eq!(label.color, Some(Color::rgb(200, 180, 90))),
            _ => panic!("expected label node"),
        }

        let knob_node = knob("k", "Drive", 0.5, (0.0, 1.0)).value_label("50%");
        match knob_node {
            Node::Knob(knob) => assert_eq!(knob.value_label.as_deref(), Some("50%")),
            _ => panic!("expected knob node"),
        }

        let slider_node = slider("mix", "Mix", 0.3, (0.0, 1.0)).control_size(Size {
            width: 140,
            height: 24,
        });
        match slider_node {
            Node::Slider(slider) => assert_eq!(
                slider.control_size,
                Some(Size {
                    width: 140,
                    height: 24
                })
            ),
            _ => panic!("expected slider node"),
        }

        let dropdown_node = dropdown(
            "mode",
            "Mode",
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            0,
        )
        .selected(2)
        .control_size(Size {
            width: 160,
            height: 24,
        });
        match dropdown_node {
            Node::Dropdown(dropdown) => {
                assert_eq!(dropdown.selected, 2);
                assert_eq!(
                    dropdown.control_size,
                    Some(Size {
                        width: 160,
                        height: 24
                    })
                );
            }
            _ => panic!("expected dropdown node"),
        }
    }

    #[test]
    fn helper_node_constructors_build_valid_spec() {
        let controls = row(vec![
            knob("drive", "Drive", 0.5, (0.0, 1.0)),
            slider("mix", "Mix", 0.25, (0.0, 1.0)),
            toggle("sync", "Sync", false),
            button("ping", "Ping"),
            dropdown("mode", "Mode", vec!["A".to_string(), "B".to_string()], 1),
        ]);
        let content = column(vec![
            label("Header"),
            controls,
            grid(
                GridTemplate::columns_fr(2).rows_fr(1).pad_all(4).gap(8),
                vec![
                    spacer(Size {
                        width: 8,
                        height: 8,
                    }),
                    indicator(
                        Size {
                            width: 8,
                            height: 8,
                        },
                        true,
                    ),
                ],
            ),
            region(
                "plot",
                Size {
                    width: 120,
                    height: 40,
                },
            ),
        ]);
        let spec = UiSpec::new(RootFrameSpec::new(
            "root",
            panel("main", content).layout(LayoutBox::fill()),
        ));
        let measured = measure_checked(&spec).expect("helper-composed tree should validate");
        assert!(measured.width > 0);
        assert!(measured.height > 0);
    }

    #[test]
    fn measure_knob_matches_shared_block_metrics() {
        let mut tokens = ThemeTokens::default();
        tokens.controls.knob_diameter = 90;
        tokens.typography.text_scale = 3;

        let knob = KnobSpec::new("k", "Drive", 0.5, (0.0, 1.0));
        let measured = measure_knob(&knob, &tokens);
        let expected = knob_block_size_for_diameter(90, 3);

        assert_eq!(measured, expected);
    }
