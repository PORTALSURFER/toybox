/// Render a scroll-view container.
fn render_scroll_view(
    scroll_view: &ScrollViewSpec,
    rect: Rect,
    ui: &mut Ui<'_>,
    ctx: &mut RenderCtx<'_>,
) {
    let inner = inset_rect(rect, scroll_view.padding);
    if inner.size.width == 0 || inner.size.height == 0 {
        return;
    }

    let child = scroll_view.content();
    let measured = measure_node(child, ctx.tokens);
    let layout = node_layout(child);
    let available = Size {
        width: inner.size.width,
        height: measured.height.max(inner.size.height),
    };
    let mut resolved = resolve_size(layout, measured, available);
    let mut child_origin = inner.origin;
    let max_offset = resolved.height.saturating_sub(inner.size.height) as i32;
    let offset_y = scroll_view.offset_y.clamp(0, max_offset);

    match scroll_view.overflow_policy() {
        OverflowPolicy::Clip => {
            child_origin.y = child_origin.y.saturating_sub(offset_y);
        }
        OverflowPolicy::Compress => {
            resolved = clamp_size_to_available(resolved, inner.size);
            child_origin = inner.origin;
            if measured.height > inner.size.height {
                record_layout_diagnostic(
                    ctx.layout_diagnostics,
                    ContainerKind::ScrollView,
                    "scroll-view content compressed to viewport",
                    Rect {
                        origin: inner.origin,
                        size: measured,
                    },
                    inner,
                );
            }
        }
    }

    let child_rect = Rect {
        origin: child_origin,
        size: resolved,
    };
    ui.with_clip(inner, |ui| {
        ctx.depth += 1;
        render_node(child, child_rect, ui, ctx);
        ctx.depth = ctx.depth.saturating_sub(1);
    });

    collect_container_debug_border_candidate(
        ctx.debug_border_candidates,
        ui,
        rect,
        ContainerKind::ScrollView,
        ctx.depth,
    );
}

