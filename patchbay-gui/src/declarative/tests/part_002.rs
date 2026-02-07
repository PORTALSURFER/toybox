
    #[test]
    fn section_tracks_allocate_percentages_before_fill_remainder() {
        let widths = resolve_grid_axis(
            &[
                TrackSize::Percent(25),
                TrackSize::Percent(35),
                TrackSize::Fill,
            ],
            3,
            1,
            0,
            200,
            true,
            &[Size {
                width: 0,
                height: 0,
            }; 3],
        );
        assert_eq!(widths, vec![50, 70, 80]);
        assert_eq!(widths.iter().sum::<u32>(), 200);
    }

    #[test]
    fn section_grid_requires_exact_hundred_percent_without_fill() {
        let spec = UiSpec::new(root_frame_sized(
            "root",
            column_sections(vec![
                weighted(panel("top", label("Top")).pad_all(0), 10),
                weighted(panel("bottom", label("Bottom")).pad_all(0), 20),
            ]),
            Size {
                width: 300,
                height: 180,
            },
            Size {
                width: 300,
                height: 180,
            },
        ));
        let error = measure_checked(&spec).expect_err("fractions below 100% must be rejected");
        assert!(matches!(
            error,
            DeclarativeError::InvalidSectionFractions {
                total_percent: 30,
                fill_count: 0
            }
        ));
    }

    #[test]
    fn section_grid_allows_sub_hundred_percent_when_fill_present() {
        let spec = UiSpec::new(root_frame_sized(
            "root",
            column_sections(vec![
                fraction(panel("fixed", label("Fixed")).pad_all(0), 30),
                fill_section(panel("fill", label("Fill")).pad_all(0)),
            ]),
            Size {
                width: 300,
                height: 180,
            },
            Size {
                width: 300,
                height: 180,
            },
        ));
        let measured = measure_checked(&spec).expect("fill should absorb remaining section space");
        assert_eq!(
            measured,
            Size {
                width: 300,
                height: 180,
            }
        );
    }

    #[test]
    fn weighted_section_lengths_consume_total_without_gaps() {
        let heights = weighted_section_lengths(259, &[7, 63, 30]);
        assert_eq!(heights, vec![18, 163, 78]);
        assert_eq!(heights.iter().sum::<u32>(), 259);

        let widths = weighted_section_lengths(799, &[70, 30]);
        assert_eq!(widths, vec![559, 240]);
        assert_eq!(widths.iter().sum::<u32>(), 799);
    }

    fn assert_tracks_tile_parent_exactly(parent_extent: u32, gap: i32, tracks: &[u32]) {
        let gap_u32 = gap.max(0) as u32;
        let gap_total = gap_u32.saturating_mul(tracks.len().saturating_sub(1) as u32);
        let tracks_total = tracks.iter().copied().sum::<u32>();
        assert_eq!(
            tracks_total.saturating_add(gap_total),
            parent_extent,
            "tracks must exactly consume parent extent with configured gaps"
        );
        let mut cursor = 0u32;
        for track in tracks {
            cursor = cursor.saturating_add(*track);
            cursor = cursor.saturating_add(gap_u32);
        }
        let used = cursor.saturating_sub(gap_u32);
        assert_eq!(used, parent_extent, "no trailing slack is allowed");
    }

    #[test]
    fn root_vertical_sections_tile_parent_without_gaps() {
        let root_size = Size {
            width: 421,
            height: 259,
        };
        let spec = UiSpec::new(
            root_frame_sized(
                "root",
                column_sections(vec![
                    weighted(panel("header", label("Header")).pad_all(0), 7),
                    weighted(panel("curve", label("Curve")).pad_all(0), 63),
                    weighted(panel("controls", label("Controls")).pad_all(0), 30),
                ]),
                root_size,
                root_size,
            )
            .padding(0),
        );

        let measured = measure_checked(&spec).expect("measurement should succeed");
        assert_eq!(measured, root_size);
        let Node::Grid(root_grid) = spec.root.content.as_ref() else {
            panic!("expected grid-backed root sections");
        };
        let row_heights = resolve_grid_axis(
            &root_grid.template.rows,
            1,
            root_grid.children.len(),
            root_grid.template.row_gap,
            measured.height,
            false,
            &vec![
                Size {
                    width: 0,
                    height: 0,
                };
                root_grid.children.len()
            ],
        );
        assert_tracks_tile_parent_exactly(
            measured.height,
            root_grid.template.row_gap,
            &row_heights,
        );
    }

    #[test]
    fn root_horizontal_sections_tile_parent_without_gaps() {
        let root_size = Size {
            width: 799,
            height: 301,
        };
        let spec = UiSpec::new(
            root_frame_sized(
                "root",
                row_sections(vec![
                    weighted(panel("left", label("Left")).pad_all(0), 17),
                    weighted(panel("center", label("Center")).pad_all(0), 55),
                    weighted(panel("right", label("Right")).pad_all(0), 28),
                ]),
                root_size,
                root_size,
            )
            .padding(0),
        );

        let measured = measure_checked(&spec).expect("measurement should succeed");
        assert_eq!(measured, root_size);
        let Node::Grid(root_grid) = spec.root.content.as_ref() else {
            panic!("expected grid-backed root sections");
        };
        let column_widths = resolve_grid_axis(
            &root_grid.template.columns,
            root_grid.template.columns.len(),
            1,
            root_grid.template.column_gap,
            measured.width,
            true,
            &vec![
                Size {
                    width: 0,
                    height: 0,
                };
                root_grid.children.len()
            ],
        );
        assert_tracks_tile_parent_exactly(
            measured.width,
            root_grid.template.column_gap,
            &column_widths,
        );
    }

    #[test]
    fn nested_section_layouts_tile_each_parent_without_gaps() {
        let right_nested = column_sections(vec![
            weighted(panel("right-top", label("R1")).pad_all(0), 40),
            weighted(panel("right-bottom", label("R2")).pad_all(0), 60),
        ]);
        let controls = row_sections(vec![
            weighted(panel("knobs", label("Knobs")).pad_all(0), 70),
            weighted(panel("dropdowns", right_nested).pad_all(0), 30),
        ]);
        let content = column_sections(vec![
            weighted(panel("header", label("Header")).pad_all(0), 9),
            weighted(panel("curve", label("Curve")).pad_all(0), 61),
            weighted(panel("controls", controls).pad_all(0), 30),
        ]);
        let root_size = Size {
            width: 803,
            height: 511,
        };
        let spec = UiSpec::new(root_frame_sized("root", content, root_size, root_size).padding(0));
        let measured = measure_checked(&spec).expect("measurement should succeed");
        assert_eq!(measured, root_size);

        let Node::Grid(root_grid) = spec.root.content.as_ref() else {
            panic!("expected grid-backed root sections");
        };
        let root_row_heights = resolve_grid_axis(
            &root_grid.template.rows,
            1,
            root_grid.children.len(),
            root_grid.template.row_gap,
            measured.height,
            false,
            &vec![
                Size {
                    width: 0,
                    height: 0,
                };
                root_grid.children.len()
            ],
        );
        assert_tracks_tile_parent_exactly(
            measured.height,
            root_grid.template.row_gap,
            &root_row_heights,
        );

        let controls_panel = match &root_grid.children[2] {
            Node::Panel(panel) => panel,
            other => panic!("expected controls panel, got {other:?}"),
        };
        let Node::Grid(controls_grid) = controls_panel.content.as_ref() else {
            panic!("expected row section grid in controls panel");
        };
        let controls_column_widths = resolve_grid_axis(
            &controls_grid.template.columns,
            controls_grid.template.columns.len(),
            1,
            controls_grid.template.column_gap,
            measured.width,
            true,
            &vec![
                Size {
                    width: 0,
                    height: 0,
                };
                controls_grid.children.len()
            ],
        );
        assert_tracks_tile_parent_exactly(
            measured.width,
            controls_grid.template.column_gap,
            &controls_column_widths,
        );

        let right_panel = match &controls_grid.children[1] {
            Node::Panel(panel) => panel,
            other => panic!("expected right panel, got {other:?}"),
        };
        let Node::Grid(right_grid) = right_panel.content.as_ref() else {
            panic!("expected nested column section grid in right panel");
        };
        let nested_row_heights = resolve_grid_axis(
            &right_grid.template.rows,
            1,
            right_grid.children.len(),
            right_grid.template.row_gap,
            root_row_heights[2],
            false,
            &vec![
                Size {
                    width: 0,
                    height: 0,
                };
                right_grid.children.len()
            ],
        );
        assert_tracks_tile_parent_exactly(
            root_row_heights[2],
            right_grid.template.row_gap,
            &nested_row_heights,
        );
    }
