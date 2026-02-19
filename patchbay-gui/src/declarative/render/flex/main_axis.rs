/// Measured child sizes used by flex layout solving.
fn measure_flex_intrinsic_children(flex: &FlexSpec, tokens: &ThemeTokens) -> Vec<Size> {
    flex.children
        .iter()
        .map(|child| measure_node(child, tokens))
        .collect()
}

/// Resolve each child's main-axis length for a flex container.
fn resolve_flex_main_lengths(
    flex: &FlexSpec,
    axis: Axis,
    intrinsic: &[Size],
    gap: i32,
    available_main: i32,
    overflow_policy: OverflowPolicy,
) -> Vec<i32> {
    let mut base_main = Vec::with_capacity(flex.children.len());
    let mut fill_weight_sum = 0u32;
    for (index, child) in flex.children.iter().enumerate() {
        let layout = node_layout(child);
        let measured_main = to_i32_saturating(axis.main(intrinsic[index]));
        let value = match axis.main_length(layout) {
            Length::Px(px) => to_i32_saturating(px),
            Length::Auto => measured_main,
            Length::Fill(_) => 0,
        };
        base_main.push(value.max(0));
        fill_weight_sum += axis.main_length(layout).fill_weight();
    }

    let main_sum = base_main.iter().copied().sum::<i32>();
    let total_gap = gap * (flex.children.len().saturating_sub(1) as i32);
    let remainder = (available_main - main_sum - total_gap).max(0);
    let mut resolved =
        distribute_flex_fill_remainder(flex, axis, remainder, fill_weight_sum, base_main);
    if overflow_policy == OverflowPolicy::Compress {
        compress_flex_main_lengths(flex, axis, available_main, gap, &mut resolved);
    }
    resolved
}

/// Distribute remaining main-axis space across `Length::Fill` children.
fn distribute_flex_fill_remainder(
    flex: &FlexSpec,
    axis: Axis,
    remainder: i32,
    fill_weight_sum: u32,
    mut resolved_main: Vec<i32>,
) -> Vec<i32> {
    if flex.children.is_empty() {
        return resolved_main;
    }

    if fill_weight_sum == 0 {
        return resolved_main;
    }

    let remainder_i64 = i64::from(remainder);
    let fill_weight_sum_i64 = i64::from(fill_weight_sum);
    let mut used = 0i64;
    for (index, child) in flex.children.iter().enumerate() {
        let weight = axis.main_length(node_layout(child)).fill_weight();
        if weight > 0 {
            let extra = (remainder_i64 * i64::from(weight) / fill_weight_sum_i64) as i32;
            resolved_main[index] += extra;
            used += i64::from(extra);
        }
    }

    let mut remaining = (remainder_i64 - used).max(0);
    let mut cursor = 0usize;
    while remaining > 0 {
        let index = cursor % flex.children.len();
        let child = &flex.children[index];
        let weight = axis.main_length(node_layout(child)).fill_weight();
        if weight > 0 {
            resolved_main[index] = resolved_main[index].saturating_add(1);
            remaining -= 1;
        }
        cursor = cursor.saturating_add(1);
    }

    resolved_main
}

/// Compress resolved flex main-axis lengths so total content fits available space.
fn compress_flex_main_lengths(
    flex: &FlexSpec,
    axis: Axis,
    available_main: i32,
    gap: i32,
    resolved_main: &mut [i32],
) {
    let total_gap = gap.max(0) * (resolved_main.len().saturating_sub(1) as i32);
    let mut overflow = resolved_main.iter().copied().sum::<i32>() + total_gap - available_main;
    if overflow <= 0 {
        return;
    }

    let min_main: Vec<i32> = flex
        .children
        .iter()
        .map(|child| {
            let layout = node_layout(child);
            let min = match axis {
                Axis::Horizontal => layout.min_width,
                Axis::Vertical => layout.min_height,
            };
            i32::try_from(min.unwrap_or(0)).unwrap_or(i32::MAX)
        })
        .collect();

    let reduce_fill: Vec<usize> = flex
        .children
        .iter()
        .enumerate()
        .filter_map(|(index, child)| {
            matches!(axis.main_length(node_layout(child)), Length::Fill(_)).then_some(index)
        })
        .collect();
    reduce_overflow_indices(resolved_main, &min_main, &reduce_fill, &mut overflow);
    if overflow <= 0 {
        return;
    }

    let reduce_auto: Vec<usize> = flex
        .children
        .iter()
        .enumerate()
        .filter_map(|(index, child)| {
            matches!(axis.main_length(node_layout(child)), Length::Auto).then_some(index)
        })
        .collect();
    reduce_overflow_indices(resolved_main, &min_main, &reduce_auto, &mut overflow);
    if overflow <= 0 {
        return;
    }

    let reduce_px: Vec<usize> = flex
        .children
        .iter()
        .enumerate()
        .filter_map(|(index, child)| {
            matches!(axis.main_length(node_layout(child)), Length::Px(_)).then_some(index)
        })
        .collect();
    reduce_overflow_indices(resolved_main, &min_main, &reduce_px, &mut overflow);
}

/// Reduce lengths for selected indices until overflow is removed or all hit min.
fn reduce_overflow_indices(
    resolved_main: &mut [i32],
    min_main: &[i32],
    indices: &[usize],
    overflow: &mut i32,
) {
    if *overflow <= 0 || indices.is_empty() {
        return;
    }

    while *overflow > 0 {
        let mut reduced_this_round = false;
        for index in indices.iter().copied() {
            if *overflow <= 0 {
                break;
            }
            let min = min_main.get(index).copied().unwrap_or(0).max(0);
            if resolved_main[index] > min {
                resolved_main[index] -= 1;
                *overflow -= 1;
                reduced_this_round = true;
            }
        }
        if !reduced_this_round {
            break;
        }
    }
}

/// Resolve flex main-axis spacing according to `justify`.
fn resolve_flex_main_spacing(
    flex: &FlexSpec,
    resolved_main: &[i32],
    child_count: usize,
    gap: i32,
    available_main: i32,
) -> FlexMainSpacing {
    let total_gap = gap * child_count.saturating_sub(1) as i32;
    let occupied_main = resolved_main.iter().copied().sum::<i32>() + total_gap;
    let free_main = (available_main - occupied_main).max(0);
    let space_weights = justify_space_weights(flex.justify, child_count);
    let extra_spaces = distribute_space(free_main, &space_weights);
    let mut gaps = vec![gap; child_count.saturating_sub(1)];
    for (index, gap_value) in gaps.iter_mut().enumerate() {
        *gap_value += extra_spaces.get(index + 1).copied().unwrap_or(0);
    }
    FlexMainSpacing {
        leading_offset: extra_spaces.first().copied().unwrap_or(0),
        gaps,
    }
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

/// Distribute integer space across weighted_slot slots.
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
