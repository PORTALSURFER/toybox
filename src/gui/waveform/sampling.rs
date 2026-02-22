//! Sample-to-pixel sampling helpers for waveform rendering.

/// Clamp bound applied to source sample values before rendering.
pub(super) const SAMPLE_CLAMP_LIMIT: f32 = 1.2;

/// Resample one channel across equally spaced output points.
pub(super) fn resample_channel_linear<SampleAt>(
    sample_count: usize,
    channel: usize,
    points: usize,
    sample_at: &SampleAt,
) -> Vec<f32>
where
    SampleAt: Fn(usize, usize) -> f32,
{
    if sample_count == 0 || points == 0 {
        return Vec::new();
    }
    if sample_count == 1 {
        return vec![clamp_sample(sample_at(channel, 0)); points];
    }
    if points == 1 {
        return vec![clamp_sample(sample_at(channel, 0))];
    }

    let max_src = (sample_count - 1) as f64;
    let mut values = Vec::with_capacity(points);
    for point_index in 0..points {
        let t = point_index as f64 / (points - 1) as f64;
        let src_pos = t * max_src;
        let src_index = src_pos.floor() as usize;
        let next_index = (src_index + 1).min(sample_count - 1);
        let frac = (src_pos - src_index as f64) as f32;
        let a = sample_at(channel, src_index);
        let b = sample_at(channel, next_index);
        values.push(clamp_sample(a + (b - a) * frac));
    }
    values
}

/// Build deterministic min/max bins for each output column.
pub(super) fn sample_envelope_min_max_for_columns<SampleAt>(
    sample_count: usize,
    channel: usize,
    columns: usize,
    sample_at: &SampleAt,
) -> Vec<(f32, f32)>
where
    SampleAt: Fn(usize, usize) -> f32,
{
    if sample_count == 0 || columns == 0 {
        return Vec::new();
    }

    let mut bins = Vec::with_capacity(columns);
    for column in 0..columns {
        let mut start = (column * sample_count) / columns;
        let mut end = ((column + 1) * sample_count) / columns;

        if start >= sample_count {
            start = sample_count - 1;
        }
        if end <= start {
            end = (start + 1).min(sample_count);
        }

        let mut min_sample = SAMPLE_CLAMP_LIMIT;
        let mut max_sample = -SAMPLE_CLAMP_LIMIT;
        for source_index in start..end {
            let sample = clamp_sample(sample_at(channel, source_index));
            if sample < min_sample {
                min_sample = sample;
            }
            if sample > max_sample {
                max_sample = sample;
            }
        }

        if min_sample > max_sample {
            let fallback = clamp_sample(sample_at(channel, start));
            min_sample = fallback;
            max_sample = fallback;
        }

        bins.push((min_sample, max_sample));
    }

    bins
}

/// Clamp one source sample to renderer-safe bounds.
pub(super) fn clamp_sample(sample: f32) -> f32 {
    sample.clamp(-SAMPLE_CLAMP_LIMIT, SAMPLE_CLAMP_LIMIT)
}
