/// Render a single-slot alignment container.
fn render_align_box(align_box: &AlignBoxSpec, rect: Rect, ui: &mut Ui<'_>, ctx: &mut RenderCtx<'_>) {
    let child = align_box.content();
    let measured = measure_node(child, ctx.tokens);
    let layout = node_layout(child);
    let resolved = clamp_size_to_available(resolve_size(layout, measured, rect.size), rect.size);
    let child_rect = align_box_child_rect(rect, resolved, align_box.align_x, align_box.align_y);
    let requested_rect = child_rect;
    let Some(child_rect) = overflow_rect_with_policy(
        child_rect,
        rect,
        align_box.overflow_policy(),
        ContainerKind::AlignBox,
        ctx.layout_diagnostics,
    ) else {
        return;
    };
    if let Some(reason) = overflow_reason(requested_rect, child_rect, align_box.overflow_policy()) {
        queue_next_node_reason(ctx, reason);
    }
    ctx.depth += 1;
    render_node(child, child_rect, ui, ctx);
    ctx.depth = ctx.depth.saturating_sub(1);

    collect_container_debug_border_candidate(
        ctx.debug_border_candidates,
        ui,
        rect,
        ContainerKind::AlignBox,
        ctx.depth,
    );
}

/// Resolve one aligned child rectangle inside an align-box bounds rectangle.
fn align_box_child_rect(rect: Rect, resolved: Size, align_x: SlotAlign, align_y: SlotAlign) -> Rect {
    let width = resolved.width.min(rect.size.width);
    let height = resolved.height.min(rect.size.height);
    let available_x = rect.size.width.saturating_sub(width) as i32;
    let available_y = rect.size.height.saturating_sub(height) as i32;
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
            x: rect.origin.x + offset_x,
            y: rect.origin.y + offset_y,
        },
        size: Size { width, height },
    }
}
