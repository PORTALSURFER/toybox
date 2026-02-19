/// Render a stack container.
fn render_stack(stack: &StackSpec, rect: Rect, ui: &mut Ui<'_>, ctx: &mut RenderCtx<'_>) {
    let inner = inset_rect(rect, stack.padding);
    for child in &stack.children {
        let measured = measure_node(child, ctx.tokens);
        let layout = node_layout(child);
        let resolved = clamp_size_to_available(resolve_size(layout, measured, inner.size), inner.size);
        let child_rect = align_stack_child_rect(inner, resolved, stack.align_x, stack.align_y);
        let Some(child_rect) = overflow_rect_with_policy(
            child_rect,
            inner,
            stack.overflow_policy(),
            ContainerKind::Stack,
            ctx.layout_diagnostics,
        ) else {
            continue;
        };
        ctx.depth += 1;
        render_node(child, child_rect, ui, ctx);
        ctx.depth = ctx.depth.saturating_sub(1);
    }

    collect_container_debug_border_candidate(
        ctx.debug_border_candidates,
        ui,
        rect,
        ContainerKind::Stack,
        ctx.depth,
    );
}

/// Resolve stack child rectangle based on stack alignment and resolved child size.
fn align_stack_child_rect(inner: Rect, resolved: Size, align_x: SlotAlign, align_y: SlotAlign) -> Rect {
    let width = resolved.width.min(inner.size.width);
    let height = resolved.height.min(inner.size.height);
    let available_x = inner.size.width.saturating_sub(width) as i32;
    let available_y = inner.size.height.saturating_sub(height) as i32;
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
            x: inner.origin.x + offset_x,
            y: inner.origin.y + offset_y,
        },
        size: Size { width, height },
    }
}

