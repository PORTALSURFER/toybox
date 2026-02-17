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
    distribute_flex_fill_remainder(flex, axis, remainder, fill_weight_sum, base_main)
}

/// Distribute remaining main-axis space across `Length::Fill` children.
fn distribute_flex_fill_remainder(
    flex: &FlexSpec,
    axis: Axis,
    remainder: i32,
    fill_weight_sum: u32,
    mut resolved_main: Vec<i32>,
) -> Vec<i32> {
    if fill_weight_sum == 0 {
        return resolved_main;
    }
    for (index, child) in flex.children.iter().enumerate() {
        let weight = axis.main_length(node_layout(child)).fill_weight();
        if weight > 0 {
            let extra = ((remainder as i64) * (weight as i64) / (fill_weight_sum as i64)) as i32;
            resolved_main[index] += extra;
        }
    }
    resolved_main
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
