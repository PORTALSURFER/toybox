/// Resolve one grid axis using track definitions and available space.
fn resolve_grid_axis(
    tracks: &[TrackSize],
    columns: usize,
    rows: usize,
    gap: i32,
    available: u32,
    is_columns: bool,
    intrinsic: &[Size],
) -> Vec<u32> {
    let axis = if is_columns {
        GridAxis::Columns
    } else {
        GridAxis::Rows
    };
    let axis_count = if is_columns { columns } else { rows };
    resolve_grid_axis_plan(&GridAxisPlan {
        tracks,
        axis_count,
        columns,
        gap,
        available,
        axis,
        intrinsic,
    })
}

/// Resolve one grid axis from an already-built axis plan.
fn resolve_grid_axis_plan(plan: &GridAxisPlan<'_>) -> Vec<u32> {
    let mut result = seed_pixel_tracks(plan);
    apply_auto_tracks(plan, &mut result);
    assign_percent_tracks(plan, &mut result);
    distribute_remaining_tracks(plan, &mut result);
    result
}

/// Axis selection for grid resolution.
#[derive(Copy, Clone)]
enum GridAxis {
    /// Resolve track sizes along the horizontal column axis.
    Columns,
    /// Resolve track sizes along the vertical row axis.
    Rows,
}

impl GridAxis {
    /// Return the axis index for a flattened grid child index.
    fn index_for_item(self, item: usize, columns: usize) -> usize {
        match self {
            Self::Columns => item % columns,
            Self::Rows => item / columns,
        }
    }

    /// Read the intrinsic size component matching this axis.
    fn intrinsic_size(self, measured: Size) -> u32 {
        match self {
            Self::Columns => measured.width,
            Self::Rows => measured.height,
        }
    }
}

/// Parameters required to resolve one grid axis.
struct GridAxisPlan<'a> {
    /// Track-size definitions for the resolved axis.
    tracks: &'a [TrackSize],
    /// Number of tracks that must be produced on this axis.
    axis_count: usize,
    /// Grid column count used to map flattened child indices.
    columns: usize,
    /// Gap size between adjacent tracks on this axis.
    gap: i32,
    /// Total axis space available before gap subtraction.
    available: u32,
    /// Axis mode that controls index and intrinsic component lookup.
    axis: GridAxis,
    /// Intrinsic child measurements used for `Auto` track sizing.
    intrinsic: &'a [Size],
}

/// Resolve track definition at an index, defaulting to `Auto`.
fn axis_track(plan: &GridAxisPlan<'_>, index: usize) -> TrackSize {
    plan.tracks.get(index).copied().unwrap_or(TrackSize::Auto)
}

/// Seed the result with fixed pixel tracks.
fn seed_pixel_tracks(plan: &GridAxisPlan<'_>) -> Vec<u32> {
    let mut result = vec![0u32; plan.axis_count];
    for (index, value) in result.iter_mut().enumerate().take(plan.axis_count) {
        if let TrackSize::Px(px) = axis_track(plan, index) {
            *value = px;
        }
    }
    result
}

/// Size auto tracks from intrinsic child measurements.
fn apply_auto_tracks(plan: &GridAxisPlan<'_>, result: &mut [u32]) {
    for (item, measured) in plan.intrinsic.iter().enumerate() {
        let index = plan.axis.index_for_item(item, plan.columns);
        if matches!(axis_track(plan, index), TrackSize::Auto) {
            result[index] = result[index].max(plan.axis.intrinsic_size(*measured));
        }
    }
}

/// Assign percent tracks as fixed percentages of available axis space.
fn assign_percent_tracks(plan: &GridAxisPlan<'_>, result: &mut [u32]) {
    let percent_weights: Vec<u8> = (0..plan.axis_count)
        .map(|index| match axis_track(plan, index) {
            TrackSize::Percent(percent) => percent,
            _ => 0,
        })
        .collect();
    let total_percent: u16 = percent_weights.iter().map(|percent| *percent as u16).sum();
    if total_percent == 0 {
        return;
    }
    let target_total = plan.available.saturating_mul(total_percent as u32).saturating_div(100);
    let weights: Vec<u32> = percent_weights.iter().map(|percent| u32::from(*percent)).collect();
    let assigned = distribute_weighted_u32(target_total, &weights);
    for (index, value) in assigned.into_iter().enumerate() {
        if percent_weights[index] > 0 {
            result[index] = value;
        }
    }
}

/// Distribute remaining space to fill tracks, otherwise FR tracks.
fn distribute_remaining_tracks(plan: &GridAxisPlan<'_>, result: &mut [u32]) {
    let remainder = remaining_axis_space(plan, result);
    let fill_indices: Vec<usize> = (0..plan.axis_count)
        .filter(|index| matches!(axis_track(plan, *index), TrackSize::Fill))
        .collect();
    if !fill_indices.is_empty() {
        let fill = distribute_weighted_u32(remainder, &vec![1u32; fill_indices.len()]);
        for (offset, index) in fill_indices.into_iter().enumerate() {
            result[index] += fill[offset];
        }
        return;
    }
    let weights: Vec<u32> = (0..plan.axis_count)
        .map(|index| axis_track(plan, index).fr_weight())
        .collect();
    let fr = distribute_weighted_u32(remainder, &weights);
    for (value, added) in result.iter_mut().zip(fr.into_iter()) {
        *value += added;
    }
}

/// Remaining space after fixed, auto and percent track resolution.
fn remaining_axis_space(plan: &GridAxisPlan<'_>, result: &[u32]) -> u32 {
    let total_gap = plan.gap.max(0) as u32 * plan.axis_count.saturating_sub(1) as u32;
    let used = result.iter().copied().sum::<u32>() + total_gap;
    plan.available.saturating_sub(used)
}

/// Distribute integer lengths across weighted slots using largest remainder.
///
/// The returned vector length matches `weights.len()`, and the sum of all
/// returned values equals `total` when at least one weight is non-zero.
fn distribute_weighted_u32(total: u32, weights: &[u32]) -> Vec<u32> {
    if weights.is_empty() {
        return Vec::new();
    }
    if total == 0 {
        return vec![0; weights.len()];
    }

    let weight_sum: u64 = weights.iter().map(|weight| u64::from(*weight)).sum();
    if weight_sum == 0 {
        return vec![0; weights.len()];
    }

    let total_u64 = u64::from(total);
    let mut distributed = vec![0u32; weights.len()];
    let mut used = 0u64;
    let mut remainder_order = Vec::new();

    for (index, weight) in weights.iter().copied().enumerate() {
        if weight == 0 {
            continue;
        }
        let numerator = total_u64 * u64::from(weight);
        let base = numerator / weight_sum;
        distributed[index] = base as u32;
        used += base;
        remainder_order.push((index, numerator % weight_sum));
    }

    let leftover = total_u64.saturating_sub(used) as usize;
    if leftover > 0 && !remainder_order.is_empty() {
        remainder_order
            .sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
        for offset in 0..leftover {
            let index = remainder_order[offset % remainder_order.len()].0;
            distributed[index] = distributed[index].saturating_add(1);
        }
    }

    distributed
}
