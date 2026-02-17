use super::super::*;

#[test]
fn plan_root_render_uniform_fit_centers_letterboxed_content() {
    let spec = UiSpec::new(
        RootFrameSpec::new(
            "root",
            Node::Region(RegionSpec::new(
                "plot",
                Size {
                    width: 100,
                    height: 50,
                },
            )),
        )
        .padding(0)
        .layout(LayoutBox::fixed(100, 50))
        .design_size(Size {
            width: 100,
            height: 50,
        })
        .scale_mode(RootScaleMode::UniformFit),
    );

    let plan = plan_root_render(
        &spec,
        Size {
            width: 300,
            height: 300,
        },
    );

    assert_eq!(
        plan.layout_size,
        Size {
            width: 100,
            height: 50
        }
    );
    assert_eq!(plan.resolved_scale, 3.0);
    assert_eq!(
        plan.transform.content_rect_surface,
        Rect {
            origin: Point { x: 0, y: 75 },
            size: Size {
                width: 300,
                height: 150,
            },
        }
    );
    assert_eq!(plan.transform.scale_x, 3.0);
    assert_eq!(plan.transform.scale_y, 3.0);
}

#[test]
fn root_transform_surface_to_design_maps_letterboxed_coordinates() {
    let spec = UiSpec::new(
        RootFrameSpec::new(
            "root",
            Node::Region(RegionSpec::new(
                "plot",
                Size {
                    width: 100,
                    height: 50,
                },
            )),
        )
        .padding(0)
        .layout(LayoutBox::fixed(100, 50))
        .design_size(Size {
            width: 100,
            height: 50,
        })
        .scale_mode(RootScaleMode::UniformFit),
    );

    let plan = plan_root_render(
        &spec,
        Size {
            width: 300,
            height: 300,
        },
    );

    assert_eq!(
        plan.transform.surface_to_design(Point { x: 150, y: 150 }),
        Point { x: 50, y: 25 }
    );
    assert_eq!(
        plan.transform.surface_to_design(Point { x: 150, y: 74 }),
        Point { x: 50, y: 0 }
    );
    assert_eq!(
        plan.transform.surface_to_design(Point { x: 150, y: 0 }),
        Point { x: 50, y: -25 }
    );
    assert_eq!(
        plan.transform.surface_to_design_clamped(Point { x: 150, y: 0 }),
        Point { x: 50, y: 0 }
    );
}

#[test]
fn plan_root_render_uniform_fit_keeps_shared_scale_for_non_integer_ratio() {
    let spec = UiSpec::new(
        RootFrameSpec::new(
            "root",
            Node::Region(RegionSpec::new(
                "plot",
                Size {
                    width: 101,
                    height: 50,
                },
            )),
        )
        .padding(0)
        .layout(LayoutBox::fixed(101, 50))
        .design_size(Size {
            width: 101,
            height: 50,
        })
        .scale_mode(RootScaleMode::UniformFit),
    );

    let plan = plan_root_render(
        &spec,
        Size {
            width: 300,
            height: 300,
        },
    );

    assert!((plan.transform.scale_x - plan.transform.scale_y).abs() < f32::EPSILON);
    assert_eq!(
        plan.transform.content_rect_surface,
        Rect {
            origin: Point { x: 0, y: 76 },
            size: Size {
                width: 300,
                height: 149,
            },
        }
    );
}

#[test]
fn uniform_fit_layout_size_is_clamped_to_design_viewport() {
    let spec = UiSpec::new(
        RootFrameSpec::new(
            "root",
            Node::Region(RegionSpec::new(
                "plot",
                Size {
                    width: 10,
                    height: 10,
                },
            )),
        )
        .padding(0)
        .layout(LayoutBox::fixed(200, 100))
        .design_size(Size {
            width: 100,
            height: 50,
        })
        .scale_mode(RootScaleMode::UniformFit),
    );

    let plan = plan_root_render(
        &spec,
        Size {
            width: 420,
            height: 200,
        },
    );

    assert_eq!(plan.layout_size, Size { width: 100, height: 50 });
    assert_eq!(plan.resolved_scale, 4.0);
    assert_eq!(
        plan.transform.scale_x,
        4.0,
        "UniformFit scale should stay shared across axes"
    );
    assert_eq!(
        plan.transform.scale_y,
        4.0,
        "UniformFit scale should stay shared across axes"
    );
    assert_eq!(
        plan.transform.content_rect_surface,
        Rect {
            origin: Point { x: 10, y: 0 },
            size: Size {
                width: 400,
                height: 200,
            },
        }
    );
}

#[test]
fn plan_root_render_none_mode_keeps_top_left_anchor_with_zoom() {
    let spec = UiSpec::new(
        RootFrameSpec::new(
            "root",
            Node::Region(RegionSpec::new(
                "plot",
                Size {
                    width: 64,
                    height: 48,
                },
            )),
        )
        .padding(0)
        .layout(LayoutBox::fixed(64, 48))
        .zoom_override(2.0),
    );

    let plan = plan_root_render(
        &spec,
        Size {
            width: 320,
            height: 200,
        },
    );

    assert_eq!(
        plan.transform.content_rect_surface,
        Rect {
            origin: Point { x: 0, y: 0 },
            size: Size {
                width: 128,
                height: 96,
            },
        }
    );
    assert_eq!(plan.resolved_scale, 2.0);
}

#[test]
fn plan_root_render_replaces_invalid_zoom_override_with_default_scale() {
    let spec = UiSpec::new(
        RootFrameSpec::new(
            "root",
            Node::Region(RegionSpec::new(
                "plot",
                Size {
                    width: 64,
                    height: 48,
                },
            )),
        )
        .padding(0)
        .layout(LayoutBox::fixed(64, 48))
        .zoom_override(-1.0),
    );

    let plan = plan_root_render(
        &spec,
        Size {
            width: 160,
            height: 120,
        },
    );

    assert_eq!(plan.resolved_scale, 1.0);
    assert_eq!(
        plan.transform.content_rect_surface,
        Rect {
            origin: Point { x: 0, y: 0 },
            size: Size {
                width: 64,
                height: 48,
            },
        }
    );
}

#[test]
fn plan_root_render_tiny_surface_clamps_surface_rect_within_window_bounds() {
    let spec = UiSpec::new(
        RootFrameSpec::new(
            "root",
            Node::Region(RegionSpec::new(
                "plot",
                Size {
                    width: 420,
                    height: 258,
                },
            )),
        )
        .padding(0)
        .layout(LayoutBox::fixed(420, 258))
        .design_size(Size {
            width: 420,
            height: 258,
        })
        .scale_mode(RootScaleMode::UniformFit),
    );

    let surface = Size {
        width: 3,
        height: 2,
    };
    let plan = plan_root_render(&spec, surface);

    let expected_scale = (3.0_f32 / 420.0).min(2.0 / 258.0);
    assert!((plan.resolved_scale - expected_scale).abs() < f32::EPSILON);
    assert_eq!(plan.transform.content_rect_surface.origin.y, 0);
    assert!(plan.transform.content_rect_surface.size.width <= surface.width);
    assert!(plan.transform.content_rect_surface.size.height <= surface.height);
    assert!(
        (plan
            .transform
            .content_rect_surface
            .origin
            .x as u32)
            .saturating_add(plan.transform.content_rect_surface.size.width)
            <= surface.width
    );
    assert!(
        (plan
            .transform
            .content_rect_surface
            .origin
            .y as u32)
            .saturating_add(plan.transform.content_rect_surface.size.height)
            <= surface.height
    );
    assert!(plan.transform.content_rect_surface.origin.x >= 0);
    assert!(plan.transform.content_rect_surface.origin.y >= 0);
}

#[test]
fn plan_root_render_stable_across_jittering_host_sizes() {
    let spec = UiSpec::new(
        RootFrameSpec::new(
            "root",
            Node::Region(RegionSpec::new(
                "plot",
                Size {
                    width: 420,
                    height: 258,
                },
            )),
        )
        .padding(0)
        .layout(LayoutBox::fixed(420, 258))
        .design_size(Size {
            width: 420,
            height: 258,
        })
        .scale_mode(RootScaleMode::UniformFit),
    );

    let host_sizes = [
        Size { width: 420, height: 258 },
        Size { width: 1, height: 1 },
        Size { width: 1024, height: 320 },
        Size { width: 2, height: 3 },
        Size { width: 16, height: 16 },
        Size { width: 420, height: 258 },
        Size { width: 640, height: 480 },
        Size { width: 3, height: 2 },
    ];

    let mut repeated_host_scale = None;
    let repeated_host_size = Size {
        width: 420,
        height: 258,
    };

    for host_size in host_sizes {
        let plan = plan_root_render(&spec, host_size);

        let expected_scale =
            (host_size.width as f32 / 420.0).min(host_size.height as f32 / 258.0);
        if host_size.width >= 420 && host_size.height >= 258 {
            assert!(
                (plan.resolved_scale - expected_scale).abs() < 1e-6,
                "expected exact uniform-fit scale for oversized host, got {}, expected {}",
                plan.resolved_scale,
                expected_scale
            );
        } else {
            assert!(plan.resolved_scale <= expected_scale + 1e-6);
        }
        assert!(plan.resolved_scale.is_finite());
        assert!(plan.resolved_scale > 0.0);

        assert!(plan.transform.content_rect_surface.origin.x >= 0);
        assert!(plan.transform.content_rect_surface.origin.y >= 0);
        assert!(
            (plan.transform.content_rect_surface.origin.x as u32)
                .saturating_add(plan.transform.content_rect_surface.size.width)
                <= host_size.width
        );
        assert!(
            (plan.transform.content_rect_surface.origin.y as u32)
                .saturating_add(plan.transform.content_rect_surface.size.height)
                <= host_size.height
        );

        if host_size == repeated_host_size {
            if let Some(last_scale) = repeated_host_scale {
                assert_eq!(plan.resolved_scale, last_scale);
            } else {
                repeated_host_scale = Some(plan.resolved_scale);
            }
        }
    }
}
