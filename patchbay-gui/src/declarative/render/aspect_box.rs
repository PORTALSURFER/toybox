/// Render a single-slot aspect-ratio container.
fn render_aspect_box(
    aspect_box: &AspectBoxSpec,
    rect: Rect,
    ui: &mut Ui<'_>,
    ctx: &mut RenderCtx<'_>,
) {
    let child_bounds = fit_aspect_rect_in_bounds(
        rect,
        aspect_box.aspect_ratio,
        aspect_box.align_x,
        aspect_box.align_y,
    );
    if child_bounds.size.width == 0 || child_bounds.size.height == 0 {
        return;
    }

    let child = aspect_box.content();
    let measured = measure_node(child, ctx.tokens);
    let layout = node_layout(child);
    let resolved = clamp_size_to_available(
        resolve_size_with_diagnostics(
            layout,
            measured,
            child_bounds.size,
            ContainerKind::AspectBox,
            ctx.layout_diagnostics,
        ),
        child_bounds.size,
    );
    let child_rect = Rect {
        origin: child_bounds.origin,
        size: resolved,
    };
    let requested_rect = child_rect;
    let Some(child_rect) = overflow_rect_with_policy(
        child_rect,
        child_bounds,
        aspect_box.overflow_policy(),
        ContainerKind::AspectBox,
        ctx.layout_diagnostics,
    ) else {
        return;
    };
    if let Some(reason) = overflow_reason(requested_rect, child_rect, aspect_box.overflow_policy()) {
        queue_next_node_reason(ctx, reason);
    }
    ctx.depth += 1;
    render_node(child, child_rect, ui, ctx);
    ctx.depth = ctx.depth.saturating_sub(1);

    collect_container_debug_border_candidate(
        ctx.debug_border_candidates,
        ui,
        rect,
        ContainerKind::AspectBox,
        ctx.depth,
    );
}

/// Fit an aspect-constrained rectangle inside `bounds` and align it.
fn fit_aspect_rect_in_bounds(
    bounds: Rect,
    aspect_ratio: AspectRatio,
    align_x: SlotAlign,
    align_y: SlotAlign,
) -> Rect {
    if bounds.size.width == 0
        || bounds.size.height == 0
        || aspect_ratio.width == 0
        || aspect_ratio.height == 0
    {
        return Rect {
            origin: bounds.origin,
            size: Size {
                width: 0,
                height: 0,
            },
        };
    }

    let width_limited_height = u64::from(bounds.size.width)
        .saturating_mul(u64::from(aspect_ratio.height))
        .saturating_div(u64::from(aspect_ratio.width))
        .min(u64::from(u32::MAX)) as u32;
    let (fitted_width, fitted_height) = if width_limited_height <= bounds.size.height {
        (bounds.size.width, width_limited_height.max(1))
    } else {
        let height_limited_width = u64::from(bounds.size.height)
            .saturating_mul(u64::from(aspect_ratio.width))
            .saturating_div(u64::from(aspect_ratio.height))
            .min(u64::from(u32::MAX)) as u32;
        (height_limited_width.max(1), bounds.size.height)
    };

    let available_x = bounds.size.width.saturating_sub(fitted_width) as i32;
    let available_y = bounds.size.height.saturating_sub(fitted_height) as i32;
    let offset_x = match align_x {
        SlotAlign::Start | SlotAlign::Stretch => 0,
        SlotAlign::Center => available_x / 2,
        SlotAlign::End => available_x,
    };
    let offset_y = match align_y {
        SlotAlign::Start | SlotAlign::Stretch => 0,
        SlotAlign::Center => available_y / 2,
        SlotAlign::End => available_y,
    };

    Rect {
        origin: Point {
            x: bounds.origin.x + offset_x,
            y: bounds.origin.y + offset_y,
        },
        size: Size {
            width: fitted_width,
            height: fitted_height,
        },
    }
}
