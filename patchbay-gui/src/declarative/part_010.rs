
/// Render a panel container.
fn render_panel(panel: &PanelSpec, rect: Rect, ui: &mut Ui<'_>, ctx: &mut RenderCtx<'_>) {
    let title = panel.title.as_deref();
    let header_height = panel
        .header_height
        .unwrap_or_else(|| panel_header_height(title, ctx.tokens));
    let style = crate::ui::PanelStyle {
        title,
        padding: panel.padding,
        background: Some(panel.background.unwrap_or(ctx.tokens.colors.surface)),
        outline: Some(panel.outline.unwrap_or(ctx.tokens.colors.border)),
        header_height: Some(header_height),
    };

    let mut outer_rect = rect;
    ui.with_layout(rect.origin, |ui| {
        let response = ui.panel_with_key(&panel.key, style, Some(rect.size), |ui, content_rect| {
            ctx.depth += 1;
            render_node(&panel.content, content_rect, ui, ctx);
            ctx.depth = ctx.depth.saturating_sub(1);
        });
        outer_rect = response.outer_rect;
    });
    collect_container_debug_border_candidate(
        ctx.debug_border_candidates,
        ui,
        outer_rect,
        ContainerKind::Panel,
        ctx.depth,
    );
}

/// Render a flex container.
fn render_flex(flex: &FlexSpec, rect: Rect, ui: &mut Ui<'_>, axis: Axis, ctx: &mut RenderCtx<'_>) {
    let child_count = flex.children.len();
    if child_count == 0 {
        return;
    }

    let mut intrinsic = Vec::with_capacity(child_count);
    for child in &flex.children {
        intrinsic.push(measure_node(child, ctx.tokens));
    }

    let gap = flex.gap.max(0);
    let inner = inset_rect(rect, flex.padding);
    let available_main = axis.main(inner.size) as i32;
    let available_cross = axis.cross(inner.size) as i32;

    let mut base_main = vec![0i32; child_count];
    let mut fill_weight_sum = 0u32;
    let mut main_sum = 0i32;

    for (index, child) in flex.children.iter().enumerate() {
        let layout = node_layout(child);
        let measured_main = axis.main(intrinsic[index]) as i32;
        let value = match axis.main_length(layout) {
            Length::Px(px) => px as i32,
            Length::Auto => measured_main,
            Length::Fill(_) => 0,
        };
        base_main[index] = value.max(0);
        main_sum += base_main[index];
        fill_weight_sum += axis.main_length(layout).fill_weight();
    }

    let total_gap = gap * (child_count.saturating_sub(1) as i32);
    let remainder = (available_main - main_sum - total_gap).max(0);

    let mut resolved_main = base_main.clone();
    if fill_weight_sum > 0 {
        for (index, child) in flex.children.iter().enumerate() {
            let weight = axis.main_length(node_layout(child)).fill_weight();
            if weight > 0 {
                let extra =
                    ((remainder as i64) * (weight as i64) / (fill_weight_sum as i64)) as i32;
                resolved_main[index] += extra;
            }
        }
    }

    let occupied_main = resolved_main.iter().copied().sum::<i32>() + total_gap;
    let free_main = (available_main - occupied_main).max(0);
    let space_weights = justify_space_weights(flex.justify, child_count);
    let extra_spaces = distribute_space(free_main, &space_weights);
    let mut gaps = vec![gap; child_count.saturating_sub(1)];
    for (index, gap_value) in gaps.iter_mut().enumerate() {
        *gap_value += extra_spaces.get(index + 1).copied().unwrap_or(0);
    }

    let mut cursor_main =
        axis.origin_main(inner.origin) + extra_spaces.first().copied().unwrap_or(0);

    for (index, child) in flex.children.iter().enumerate() {
        let layout = node_layout(child);
        let intrinsic_cross = axis.cross(intrinsic[index]) as i32;
        let cross_size = match axis.cross_length(layout) {
            Length::Px(px) => px as i32,
            Length::Fill(_) => available_cross,
            Length::Auto => {
                if flex.align == Align::Stretch {
                    available_cross
                } else {
                    intrinsic_cross
                }
            }
        }
        .max(0);

        let cross_origin = match flex.align {
            Align::Start | Align::Stretch => axis.origin_cross(inner.origin),
            Align::Center => axis.origin_cross(inner.origin) + (available_cross - cross_size) / 2,
            Align::End => axis.origin_cross(inner.origin) + (available_cross - cross_size),
        };

        let child_rect =
            axis.compose_rect(cursor_main, cross_origin, resolved_main[index], cross_size);
        let resolved_child = clamp_size_to_available(
            resolve_size(layout, intrinsic[index], child_rect.size),
            child_rect.size,
        );
        let child_rect = Rect {
            origin: child_rect.origin,
            size: resolved_child,
        };

        ctx.depth += 1;
        render_node(child, child_rect, ui, ctx);
        ctx.depth = ctx.depth.saturating_sub(1);
        let next_gap = gaps.get(index).copied().unwrap_or(0);
        cursor_main += resolved_main[index] + next_gap;
    }

    collect_container_debug_border_candidate(
        ctx.debug_border_candidates,
        ui,
        rect,
        ContainerKind::Flex,
        ctx.depth,
    );
}

/// Return per-space weighting for flex main-axis justification.
///
/// The returned vector length is always `child_count + 1`, where:
/// - index `0` is leading edge space
/// - index `child_count` is trailing edge space
/// - interior indices represent gaps between children
fn justify_space_weights(justify: Justify, child_count: usize) -> Vec<u32> {
    let mut weights = vec![0u32; child_count.saturating_add(1)];
    if child_count == 0 {
        return weights;
    }

    match justify {
        Justify::Start => {
            if let Some(last) = weights.last_mut() {
                *last = 1;
            }
        }
        Justify::Center => {
            weights[0] = 1;
            if let Some(last) = weights.last_mut() {
                *last = 1;
            }
        }
        Justify::End => {
            weights[0] = 1;
        }
        Justify::SpaceBetween => {
            if child_count > 1 {
                for weight in weights.iter_mut().skip(1).take(child_count - 1) {
                    *weight = 1;
                }
            }
        }
        Justify::SpaceAround => {
            weights[0] = 1;
            if let Some(last) = weights.last_mut() {
                *last = 1;
            }
            if child_count > 1 {
                for weight in weights.iter_mut().skip(1).take(child_count - 1) {
                    *weight = 2;
                }
            }
        }
        Justify::SpaceEvenly => {
            weights.fill(1);
        }
    }

    weights
}

/// Distribute integer space across weighted slots.
fn distribute_space(total: i32, weights: &[u32]) -> Vec<i32> {
    if total <= 0 || weights.is_empty() {
        return vec![0; weights.len()];
    }
    let weight_sum: u32 = weights.iter().copied().sum();
    if weight_sum == 0 {
        return vec![0; weights.len()];
    }

    let mut distributed = vec![0i32; weights.len()];
    let mut used = 0i64;
    let total_i64 = total as i64;
    let weight_sum_i64 = weight_sum as i64;
    for (index, weight) in weights.iter().copied().enumerate() {
        if weight == 0 {
            continue;
        }
        let value = (total_i64 * weight as i64 / weight_sum_i64) as i32;
        distributed[index] = value;
        used += value as i64;
    }

    let mut remainder = (total_i64 - used).max(0) as i32;
    let mut cursor = 0usize;
    while remainder > 0 {
        if weights[cursor] > 0 {
            distributed[cursor] += 1;
            remainder -= 1;
        }
        cursor = (cursor + 1) % weights.len();
    }

    distributed
}
