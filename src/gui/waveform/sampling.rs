//! Sample-to-pixel sampling helpers for waveform rendering.

/// Clamp bound applied to source sample values before rendering.
pub(super) const SAMPLE_CLAMP_LIMIT: f32 = 1.2;

/// One min/max envelope bin.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct EnvelopeBin {
    /// Minimum sample value in the covered range.
    pub(super) min: f32,
    /// Maximum sample value in the covered range.
    pub(super) max: f32,
}

impl EnvelopeBin {
    /// Build one empty aggregation bin.
    fn empty() -> Self {
        Self {
            min: SAMPLE_CLAMP_LIMIT,
            max: -SAMPLE_CLAMP_LIMIT,
        }
    }

    /// Build one clamped single-sample bin.
    fn from_sample(sample: f32) -> Self {
        let sample = clamp_sample(sample);
        Self {
            min: sample,
            max: sample,
        }
    }

    /// Merge two bins into one covering both ranges.
    fn combine(self, other: Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }
}

/// Segment-tree envelope cache for fast min/max range queries.
#[derive(Clone, Debug, Default)]
pub(super) struct EnvelopeMinMaxTree {
    /// Number of valid source samples represented in this tree.
    sample_count: usize,
    /// Power-of-two leaf count used by the packed segment tree.
    leaf_count: usize,
    /// Packed tree nodes, root at index 1.
    nodes: Vec<EnvelopeBin>,
}

impl EnvelopeMinMaxTree {
    /// Rebuild this tree from one channel sample slice.
    pub(super) fn rebuild_from_slice(&mut self, samples: &[f32]) {
        self.sample_count = samples.len();
        self.leaf_count = samples.len().max(1).next_power_of_two();
        self.nodes
            .resize(self.leaf_count.saturating_mul(2), EnvelopeBin::empty());

        let leaf_start = self.leaf_count;
        for (index, sample) in samples.iter().enumerate() {
            self.nodes[leaf_start + index] = EnvelopeBin::from_sample(*sample);
        }
        for index in samples.len()..self.leaf_count {
            self.nodes[leaf_start + index] = EnvelopeBin::empty();
        }
        for index in (1..leaf_start).rev() {
            self.nodes[index] = self.nodes[index * 2].combine(self.nodes[index * 2 + 1]);
        }
    }

    /// Query min/max for one half-open source range `[start, end)`.
    pub(super) fn query_range(&self, start: usize, end: usize) -> EnvelopeBin {
        if self.sample_count == 0 || start >= end {
            return EnvelopeBin::from_sample(0.0);
        }
        let bounded_start = start.min(self.sample_count - 1);
        let bounded_end = end.min(self.sample_count).max(bounded_start + 1);

        let mut left = bounded_start + self.leaf_count;
        let mut right = bounded_end + self.leaf_count;
        let mut aggregate = EnvelopeBin::empty();
        while left < right {
            if (left & 1) == 1 {
                aggregate = aggregate.combine(self.nodes[left]);
                left += 1;
            }
            if (right & 1) == 1 {
                right -= 1;
                aggregate = aggregate.combine(self.nodes[right]);
            }
            left /= 2;
            right /= 2;
        }
        if aggregate.min > aggregate.max {
            EnvelopeBin::from_sample(0.0)
        } else {
            aggregate
        }
    }
}

/// Resample one channel across equally spaced output points.
pub(super) fn resample_channel_linear_into<SampleAt>(
    sample_count: usize,
    channel: usize,
    points: usize,
    sample_at: &SampleAt,
    out: &mut Vec<f32>,
) where
    SampleAt: Fn(usize, usize) -> f32,
{
    if sample_count == 0 || points == 0 {
        out.clear();
        return;
    }
    if sample_count == 1 {
        out.clear();
        out.resize(points, clamp_sample(sample_at(channel, 0)));
        return;
    }
    if points == 1 {
        out.clear();
        out.push(clamp_sample(sample_at(channel, 0)));
        return;
    }

    out.clear();
    out.reserve(points);
    let max_src = (sample_count - 1) as f64;
    for point_index in 0..points {
        let t = point_index as f64 / (points - 1) as f64;
        let src_pos = t * max_src;
        let src_index = src_pos.floor() as usize;
        let next_index = (src_index + 1).min(sample_count - 1);
        let frac = (src_pos - src_index as f64) as f32;
        let a = sample_at(channel, src_index);
        let b = sample_at(channel, next_index);
        out.push(clamp_sample(a + (b - a) * frac));
    }
}

/// Resample one channel across equally spaced output points from a slice.
pub(super) fn resample_channel_linear_from_slice_into(
    samples: &[f32],
    points: usize,
    out: &mut Vec<f32>,
) {
    if samples.is_empty() || points == 0 {
        out.clear();
        return;
    }
    if samples.len() == 1 {
        out.clear();
        out.resize(points, clamp_sample(samples[0]));
        return;
    }
    if points == 1 {
        out.clear();
        out.push(clamp_sample(samples[0]));
        return;
    }

    out.clear();
    out.reserve(points);
    let sample_count = samples.len();
    let max_src = (sample_count - 1) as f64;
    for point_index in 0..points {
        let t = point_index as f64 / (points - 1) as f64;
        let src_pos = t * max_src;
        let src_index = src_pos.floor() as usize;
        let next_index = (src_index + 1).min(sample_count - 1);
        let frac = (src_pos - src_index as f64) as f32;
        let a = samples[src_index];
        let b = samples[next_index];
        out.push(clamp_sample(a + (b - a) * frac));
    }
}

/// Fill one reusable vector with clamped phase-aligned source ranges.
///
/// Ranges are half-open (`[start, end)`) and contain at least one sample when
/// `sample_count > 0` and `columns > 0`.
pub(super) fn fill_phase_aligned_column_bounds_into(
    sample_count: usize,
    columns: usize,
    start_sample: u64,
    out: &mut Vec<(usize, usize)>,
) {
    out.clear();
    if sample_count == 0 || columns == 0 {
        return;
    }
    out.reserve(columns);
    for column in 0..columns {
        let (mut start, mut end) =
            phase_aligned_column_bounds(sample_count, columns, start_sample, column);
        if start >= sample_count {
            start = sample_count - 1;
        }
        if end <= start {
            end = (start + 1).min(sample_count);
        }
        out.push((start, end));
    }
}

/// Clamp one source sample to renderer-safe bounds.
pub(super) fn clamp_sample(sample: f32) -> f32 {
    sample.clamp(-SAMPLE_CLAMP_LIMIT, SAMPLE_CLAMP_LIMIT)
}

/// Compute one phase-aligned source range for an envelope column.
fn phase_aligned_column_bounds(
    sample_count: usize,
    columns: usize,
    start_sample: u64,
    column: usize,
) -> (usize, usize) {
    if sample_count == 0 || columns == 0 {
        return (0, 0);
    }
    let n = sample_count as u128;
    let c = columns as u128;
    // Phase-lock the remainder distribution to output-column cadence so
    // column-bin boundaries advance consistently with transport phase.
    let phase = (start_sample % columns as u64) as u128;
    let start = (((column as u128) * n + phase) / c) as usize;
    let end = ((((column as u128) + 1) * n + phase) / c) as usize;
    (start, end)
}
