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

    assert!(plan.transform.content_rect_surface.size.width <= surface.width);
    assert!(plan.transform.content_rect_surface.size.height <= surface.height);
    assert!(plan.transform.content_rect_surface.origin.x >= 0);
    assert!(plan.transform.content_rect_surface.origin.y >= 0);
}
