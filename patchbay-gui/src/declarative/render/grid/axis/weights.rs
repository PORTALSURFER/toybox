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
