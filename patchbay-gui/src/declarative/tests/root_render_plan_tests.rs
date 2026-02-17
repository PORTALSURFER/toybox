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
