/// Resolve one grid axis using track definitions and available space.
fn resolve_grid_axis(request: GridAxisResolveRequest<'_>) -> Vec<u32> {
    let axis = if request.is_columns {
        GridAxis::Columns
    } else {
        GridAxis::Rows
    };
    let axis_count = if request.is_columns {
        request.columns
    } else {
        request.rows
    };
    resolve_grid_axis_plan(&GridAxisPlan {
        tracks: request.tracks,
        axis_count,
        columns: request.columns,
        gap: request.gap,
        available: request.available,
        axis,
        intrinsic: request.intrinsic,
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
