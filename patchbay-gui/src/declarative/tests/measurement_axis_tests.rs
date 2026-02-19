use super::super::*;

#[test]
fn resolve_flex_child_cross_origin_clamps_oversized_children_to_container() {
    let inner = Rect {
        origin: Point { x: 10, y: 5 },
        size: Size { width: 120, height: 80 },
    };
    let oversized_child = resolve_flex_child_cross_origin(Axis::Horizontal, inner, 10, 24, Align::Center);
    assert_eq!(oversized_child, 5);

    let end_aligned = resolve_flex_child_cross_origin(Axis::Horizontal, inner, 10, 24, Align::End);
    assert_eq!(end_aligned, 5);

    let stretched = resolve_flex_child_cross_origin(Axis::Horizontal, inner, 10, 24, Align::Stretch);
    assert_eq!(stretched, 5);
}

#[test]
fn axis_cross_to_i32_is_saturating_for_large_sizes() {
    assert_eq!(to_i32_saturating(u32::MAX), i32::MAX);
}

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
fn resolve_grid_axis_auto_tracks_keep_intrinsic_widths_with_free_space() {
    let widths = resolve_grid_axis(GridAxisResolveRequest {
        tracks: &[
            TrackSize::Auto,
            TrackSize::Auto,
            TrackSize::Auto,
            TrackSize::Auto,
        ],
        columns: 4,
        rows: 1,
        gap: 0,
        available: 294,
        is_columns: true,
        intrinsic: &[
            Size {
                width: 48,
                height: 0,
            },
            Size {
                width: 48,
                height: 0,
            },
            Size {
                width: 48,
                height: 0,
            },
            Size {
                width: 48,
                height: 0,
            },
        ],
    });

    assert_eq!(widths, vec![48, 48, 48, 48]);
    assert_eq!(widths.iter().sum::<u32>(), 192);
}

#[test]
fn resolve_grid_axis_percent_tracks_shrink_for_fixed_tracks_and_gaps() {
    let widths = resolve_grid_axis(GridAxisResolveRequest {
        tracks: &[TrackSize::Px(60), TrackSize::Percent(60), TrackSize::Percent(40)],
        columns: 3,
        rows: 1,
        gap: 2,
        available: 302,
        is_columns: true,
        intrinsic: &[Size {
            width: 0,
            height: 0,
        }; 3],
    });
    assert_eq!(widths, vec![60, 143, 95]);
    assert_eq!(widths.iter().sum::<u32>(), 298);
}

#[test]
fn resolve_grid_axis_percent_tracks_shrink_for_auto_tracks_and_gap_space() {
    let widths = resolve_grid_axis(GridAxisResolveRequest {
        tracks: &[
            TrackSize::Px(70),
            TrackSize::Auto,
            TrackSize::Percent(75),
            TrackSize::Percent(25),
        ],
        columns: 4,
        rows: 1,
        gap: 4,
        available: 400,
        is_columns: true,
        intrinsic: &[
            Size {
                width: 30,
                height: 0,
            },
            Size {
                width: 44,
                height: 0,
            },
            Size {
                width: 12,
                height: 0,
            },
            Size {
                width: 28,
                height: 0,
            },
        ],
    });
    assert_eq!(widths, vec![70, 44, 206, 68]);
    assert_eq!(widths.iter().sum::<u32>(), 388);
}

#[test]
fn resolve_grid_axis_percent_tracks_normalize_and_fit_when_over_subscribed() {
    let widths = resolve_grid_axis(GridAxisResolveRequest {
        tracks: &[
            TrackSize::Px(80),
            TrackSize::Percent(90),
            TrackSize::Percent(90),
        ],
        columns: 3,
        rows: 1,
        gap: 0,
        available: 250,
        is_columns: true,
        intrinsic: &[Size {
            width: 0,
            height: 0,
        }; 3],
    });
    assert_eq!(widths, vec![80, 85, 85]);
    assert_eq!(widths.iter().sum::<u32>(), 250);
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
        LayoutBox::fixed(420, 400),
        "root should clamp to min width and use host-provided height"
    );
    assert_eq!(root.scale_mode, RootScaleMode::None);
    assert_eq!(root.design_size, None);
}

#[test]
fn root_frame_sized_uses_expanded_host_size() {
    let root = root_frame_sized(
        "root",
        label("x"),
        Size {
            width: 420,
            height: 258,
        },
        Size {
            width: 840,
            height: 516,
        },
    );
    assert_eq!(
        root.layout,
        LayoutBox::fixed(840, 516),
        "root should track host-provided size when host is larger than minimum"
    );
    assert_eq!(root.scale_mode, RootScaleMode::None);
    assert_eq!(root.design_size, None);
}

#[test]
fn nested_slot_helpers_measure_successfully() {
    let controls = row_slots(vec![
        weighted_slot(panel("left", label("Knobs")).pad_all(0), 70),
        weighted_slot(panel("right", label("Dropdowns")).pad_all(0), 30),
    ]);
    let content = column_slots(vec![
        weighted_slot(panel("header", label("Header")).pad_all(0), 7),
        weighted_slot(panel("curve", label("Curve")).pad_all(0), 63),
        weighted_slot(panel("controls", controls).pad_all(0), 30),
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

    let measured = measure_checked(&spec).expect("nested slot helpers should validate");
    assert!(measured.width >= 420);
    assert!(measured.height >= 258);
}


#[test]
fn justify_weighting_and_distribution_cover_new_modes() {
    let start = justify_space_weights(Justify::Start, 4);
    assert_eq!(start, vec![0, 0, 0, 0, 1]);
    assert_eq!(distribute_space(27, &start), vec![0, 0, 0, 0, 27]);

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
fn resolve_flex_fill_remainder_keeps_total_exact_for_uneven_weights() {
    let flex = FlexSpec::row(vec![
        label("").layout(LayoutBox::auto().with_width(Length::Fill(1))),
        label("").layout(LayoutBox::auto().with_width(Length::Fill(1))),
        label("").layout(LayoutBox::auto().with_width(Length::Fill(1))),
    ]);
    let intrinsic = vec![
        Size {
            width: 0,
            height: 1,
        },
        Size {
            width: 0,
            height: 1,
        },
        Size {
            width: 0,
            height: 1,
        },
    ];
    let resolved = resolve_flex_main_lengths(&flex, Axis::Horizontal, &intrinsic, 0, 100);
    assert_eq!(resolved, vec![34, 33, 33]);
    assert_eq!(resolved.iter().sum::<i32>(), 100);
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
fn clip_rect_to_bounds_keeps_rects_inside_bounds() {
    let rect = Rect {
        origin: Point { x: 4, y: 6 },
        size: Size {
            width: 40,
            height: 50,
        },
    };
    let bounds = Rect {
        origin: Point { x: 0, y: 0 },
        size: Size {
            width: 100,
            height: 100,
        },
    };

    let clipped = clip_rect_to_bounds(rect, bounds).expect("in-bounds rect should remain");
    assert_eq!(clipped, rect);
}

#[test]
fn clip_rect_to_bounds_clamps_rect_partially_outside_bounds() {
    let rect = Rect {
        origin: Point { x: 80, y: 40 },
        size: Size {
            width: 50,
            height: 30,
        },
    };
    let bounds = Rect {
        origin: Point { x: 0, y: 0 },
        size: Size {
            width: 100,
            height: 100,
        },
    };

    let clipped = clip_rect_to_bounds(rect, bounds)
        .expect("partially overlapping rect should still render with clipped area");
    assert_eq!(clipped.origin.x, 80);
    assert_eq!(clipped.size.width, 20);
    assert_eq!(clipped.origin.y, 40);
    assert_eq!(clipped.size.height, 30);
}

#[test]
fn clip_rect_to_bounds_returns_none_for_disjoint_rect() {
    let rect = Rect {
        origin: Point { x: 160, y: 160 },
        size: Size {
            width: 20,
            height: 20,
        },
    };
    let bounds = Rect {
        origin: Point { x: 0, y: 0 },
        size: Size {
            width: 100,
            height: 100,
        },
    };

    assert_eq!(clip_rect_to_bounds(rect, bounds), None);
}

#[test]
#[cfg(not(debug_assertions))]
fn resolve_axis_with_inverted_min_max_constraints_still_clamps_to_max() {
    let resolved = resolve_axis(
        Length::Auto,
        32,
        128,
        Some(200),
        Some(100),
    );
    assert_eq!(resolved, 100);
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "layout-axis: layout min constraint (200) exceeds max constraint (100)")]
fn resolve_axis_with_inverted_min_max_constraints_triggers_debug_assert() {
    let _ = resolve_axis(
        Length::Auto,
        32,
        128,
        Some(200),
        Some(100),
    );
}
