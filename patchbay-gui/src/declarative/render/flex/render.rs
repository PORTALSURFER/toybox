/// Render a flex container.
fn render_flex(flex: &FlexSpec, rect: Rect, ui: &mut Ui<'_>, axis: Axis, ctx: &mut RenderCtx<'_>) {
    let child_count = flex.children.len();
    if child_count == 0 {
        return;
    }

    let gap = flex.gap.max(0);
    let inner = inset_rect(rect, flex.padding);
    let available_main = to_i32_saturating(axis.main(inner.size));
    let intrinsic = measure_flex_intrinsic_children(flex, ctx.tokens);
    let resolved_main = resolve_flex_main_lengths(
        flex,
        axis,
        &intrinsic,
        gap,
        available_main,
        flex.overflow_policy(),
    );
    let main_spacing =
        resolve_flex_main_spacing(flex, &resolved_main, child_count, gap, available_main);
    let render_ctx = FlexRenderContext::new(axis, inner, flex.align);
    let solved = FlexSolvedLengths::new(&intrinsic, &resolved_main, &main_spacing);
    render_flex_children(flex, &render_ctx, &solved, ui, ctx);
    emit_flex_debug_border(ui, ctx, rect);
}

/// Render all flex children using pre-resolved main sizes and spacing.
fn render_flex_children(
    flex: &FlexSpec,
    render_ctx: &FlexRenderContext,
    solved: &FlexSolvedLengths<'_>,
    ui: &mut Ui<'_>,
    ctx: &mut RenderCtx<'_>,
) {
    let mut cursor_main =
        render_ctx.axis.origin_main(render_ctx.inner.origin) + solved.main_spacing.leading_offset;
    for (index, child) in flex.children.iter().enumerate() {
        let child_rect = resolve_flex_child_rect(
            child,
            index,
            cursor_main,
            render_ctx,
            solved,
            ctx.layout_diagnostics,
        );
        let requested_rect = child_rect;
        let Some(child_rect) = overflow_rect_with_policy(
            child_rect,
            render_ctx.inner,
            flex.overflow_policy(),
            ContainerKind::Flex,
            ctx.layout_diagnostics,
        ) else {
            cursor_main = advance_flex_cursor(cursor_main, index, solved);
            continue;
        };
        if let Some(reason) = overflow_reason(requested_rect, child_rect, flex.overflow_policy()) {
            queue_next_node_reason(ctx, reason);
        }
        ctx.depth += 1;
        render_node(child, child_rect, ui, ctx);
        ctx.depth = ctx.depth.saturating_sub(1);
        cursor_main = advance_flex_cursor(cursor_main, index, solved);
    }
}

/// Advance the main-axis cursor past one child and its trailing gap.
fn advance_flex_cursor(cursor_main: i32, index: usize, solved: &FlexSolvedLengths<'_>) -> i32 {
    let next_gap = solved.main_spacing.gaps.get(index).copied().unwrap_or(0);
    cursor_main + solved.resolved_main[index] + next_gap
}

/// Resolve a child's final clamped rectangle in flex layout.
fn resolve_flex_child_rect(
    child: &Node,
    index: usize,
    cursor_main: i32,
    render_ctx: &FlexRenderContext,
    solved: &FlexSolvedLengths<'_>,
    layout_diagnostics: &mut Vec<LayoutDiagnostic>,
) -> Rect {
    let intrinsic = solved.intrinsic[index];
    let resolved_main = solved.resolved_main[index];
    let layout = node_layout(child);
    let cross_size = resolve_flex_child_cross_size(
        render_ctx.axis,
        intrinsic,
        layout,
        render_ctx.available_cross,
        render_ctx.align,
    );
    let cross_origin = resolve_flex_child_cross_origin(
        render_ctx.axis,
        render_ctx.inner,
        render_ctx.available_cross,
        cross_size,
        render_ctx.align,
    );
    let slot_rect = render_ctx
        .axis
        .compose_rect(cursor_main, cross_origin, resolved_main, cross_size);
    let resolved_child = clamp_size_to_available(
        resolve_size_with_diagnostics(
            layout,
            intrinsic,
            slot_rect.size,
            ContainerKind::Flex,
            layout_diagnostics,
        ),
        slot_rect.size,
    );
    Rect {
        origin: slot_rect.origin,
        size: resolved_child,
    }
}

/// Resolve cross-axis size for one flex child.
fn resolve_flex_child_cross_size(
    axis: Axis,
    intrinsic: Size,
    layout: LayoutBox,
    available_cross: i32,
    align: Align,
) -> i32 {
    let intrinsic_cross = to_i32_saturating(axis.cross(intrinsic));
    match axis.cross_length(layout) {
        Length::Px(px) => to_i32_saturating(px),
        Length::Fill(_) => available_cross,
        Length::Auto => {
            if align == Align::Stretch {
                available_cross
            } else {
                intrinsic_cross
            }
        }
    }
    .max(0)
}

/// Resolve cross-axis origin for one flex child.
fn resolve_flex_child_cross_origin(
    axis: Axis,
    inner: Rect,
    available_cross: i32,
    cross_size: i32,
    align: Align,
) -> i32 {
    let available_cross = available_cross.max(0);
    let clamped_cross_size = cross_size.max(0).min(available_cross);
    let base_origin = axis.origin_cross(inner.origin);
    let target = if available_cross == 0 {
        0
    } else {
        match align {
            Align::Start | Align::Stretch => 0,
            Align::Center => (available_cross - clamped_cross_size) / 2,
            Align::End => available_cross - clamped_cross_size,
        }
    };
    base_origin + target.max(0).min(available_cross - clamped_cross_size)
}

/// Emit the debug-border candidate for the flex container.
fn emit_flex_debug_border(ui: &mut Ui<'_>, ctx: &mut RenderCtx<'_>, rect: Rect) {
    collect_container_debug_border_candidate(
        ctx.debug_border_candidates,
        ui,
        rect,
        ContainerKind::Flex,
        ctx.depth,
    );
}
