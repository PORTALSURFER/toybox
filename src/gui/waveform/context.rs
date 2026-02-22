//! Reusable state for cached waveform rendering.

use super::Point;
use super::sampling::EnvelopeMinMaxTree;

/// Reusable context for high-frequency waveform rendering.
///
/// Reusing one context across frames enables:
/// - cached multiresolution envelope queries for envelope mode,
/// - callback sample materialization reuse keyed by `sample_revision`, and
/// - scratch-buffer reuse to reduce allocator churn.
#[derive(Clone, Debug, Default)]
pub struct WaveformRenderContext {
    /// Per-channel envelope cache keyed by sample revision and dimensions.
    envelope_cache: WaveformEnvelopeCache,
    /// Reusable temporary vectors used by waveform builders.
    scratch: WaveformRenderScratch,
    /// Callback-materialized channel samples keyed by revision.
    callback_samples: Vec<Vec<f32>>,
    /// Revision tag for callback materialization.
    callback_revision: Option<u64>,
    /// Sample count stored in callback materialization.
    callback_sample_count: usize,
    /// Channel count stored in callback materialization.
    callback_channel_count: usize,
}

impl WaveformRenderContext {
    /// Create one empty waveform render context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Drop cached sample/envelope data while retaining allocated capacity.
    pub fn clear(&mut self) {
        self.envelope_cache.clear();
        self.scratch.clear();
        self.callback_revision = None;
        self.callback_sample_count = 0;
        self.callback_channel_count = 0;
        for channel in &mut self.callback_samples {
            channel.clear();
        }
    }

    /// Ensure callback sample materialization matches the requested revision.
    pub(super) fn ensure_callback_samples<SampleAt>(
        &mut self,
        sample_revision: u64,
        sample_count: usize,
        channel_count: usize,
        sample_at: &SampleAt,
    ) where
        SampleAt: Fn(usize, usize) -> f32,
    {
        let already_valid = self.callback_revision == Some(sample_revision)
            && self.callback_sample_count == sample_count
            && self.callback_channel_count == channel_count;
        if already_valid {
            return;
        }

        if self.callback_samples.len() < channel_count {
            self.callback_samples.resize_with(channel_count, Vec::new);
        }
        for channel in 0..channel_count {
            let values = &mut self.callback_samples[channel];
            values.clear();
            values.reserve(sample_count);
            for sample_index in 0..sample_count {
                values.push(sample_at(channel, sample_index));
            }
        }

        self.callback_revision = Some(sample_revision);
        self.callback_sample_count = sample_count;
        self.callback_channel_count = channel_count;
    }

    /// Return materialized samples for one callback channel.
    pub(super) fn callback_channel_samples(&self, channel: usize) -> Option<&[f32]> {
        self.callback_samples.get(channel).map(Vec::as_slice)
    }

    /// Ensure the envelope cache is ready for callback-materialized samples.
    pub(super) fn ensure_envelope_cache_from_callback(
        &mut self,
        sample_revision: u64,
        sample_count: usize,
        channel_count: usize,
    ) {
        self.envelope_cache.ensure_from_callback_samples(
            sample_revision,
            sample_count,
            channel_count,
            &self.callback_samples,
        );
    }

    /// Ensure the envelope cache is ready for slice-backed samples.
    pub(super) fn ensure_envelope_cache_from_slices(
        &mut self,
        sample_revision: u64,
        sample_count: usize,
        channel_samples: &[&[f32]],
        channel_count: usize,
    ) {
        self.envelope_cache.ensure_from_slices(
            sample_revision,
            sample_count,
            channel_samples,
            channel_count,
        );
    }

    /// Return one cached envelope tree for a channel when available.
    pub(super) fn envelope_tree(&self, channel: usize) -> Option<&EnvelopeMinMaxTree> {
        self.envelope_cache.tree(channel)
    }

    /// Move scratch storage out of the context for one render pass.
    pub(super) fn take_scratch(&mut self) -> WaveformRenderScratch {
        std::mem::take(&mut self.scratch)
    }

    /// Restore scratch storage after one render pass.
    pub(super) fn restore_scratch(&mut self, scratch: WaveformRenderScratch) {
        self.scratch = scratch;
    }
}

/// Reusable temporary vectors used by waveform command generation.
#[derive(Clone, Debug, Default)]
pub(super) struct WaveformRenderScratch {
    /// Linear resample output samples.
    pub(super) linear_samples: Vec<f32>,
    /// Generic contour polyline points.
    pub(super) contour: Vec<Point>,
    /// Top envelope contour for styled envelope mode.
    pub(super) top_contour: Vec<Point>,
    /// Bottom envelope contour for styled envelope mode.
    pub(super) bottom_contour: Vec<Point>,
    /// Shifted contour scratch for glow layer polylines.
    pub(super) shifted: Vec<Point>,
}

impl WaveformRenderScratch {
    /// Clear all temporary vectors while retaining capacity.
    fn clear(&mut self) {
        self.linear_samples.clear();
        self.contour.clear();
        self.top_contour.clear();
        self.bottom_contour.clear();
        self.shifted.clear();
    }
}

/// Per-channel hierarchical envelope cache.
#[derive(Clone, Debug, Default)]
struct WaveformEnvelopeCache {
    /// Revision tag for current cache contents.
    revision: Option<u64>,
    /// Sample count represented by cached trees.
    sample_count: usize,
    /// Channel count represented by cached trees.
    channel_count: usize,
    /// Per-channel segment trees.
    trees: Vec<EnvelopeMinMaxTree>,
}

impl WaveformEnvelopeCache {
    /// Drop cache metadata while retaining allocated tree storage.
    fn clear(&mut self) {
        self.revision = None;
        self.sample_count = 0;
        self.channel_count = 0;
        self.trees.clear();
    }

    /// Return one cached tree by channel index.
    fn tree(&self, channel: usize) -> Option<&EnvelopeMinMaxTree> {
        self.trees.get(channel)
    }

    /// Ensure cache reflects one set of slice-backed channels.
    fn ensure_from_slices(
        &mut self,
        sample_revision: u64,
        sample_count: usize,
        channel_samples: &[&[f32]],
        channel_count: usize,
    ) {
        if self.revision == Some(sample_revision)
            && self.sample_count == sample_count
            && self.channel_count == channel_count
        {
            return;
        }

        if self.trees.len() < channel_count {
            self.trees
                .resize_with(channel_count, EnvelopeMinMaxTree::default);
        }
        for (channel, samples) in channel_samples.iter().take(channel_count).enumerate() {
            self.trees[channel].rebuild_from_slice(&samples[..sample_count.min(samples.len())]);
        }

        self.revision = Some(sample_revision);
        self.sample_count = sample_count;
        self.channel_count = channel_count;
    }

    /// Ensure cache reflects one set of callback-materialized channels.
    fn ensure_from_callback_samples(
        &mut self,
        sample_revision: u64,
        sample_count: usize,
        channel_count: usize,
        callback_samples: &[Vec<f32>],
    ) {
        if self.revision == Some(sample_revision)
            && self.sample_count == sample_count
            && self.channel_count == channel_count
        {
            return;
        }

        if self.trees.len() < channel_count {
            self.trees
                .resize_with(channel_count, EnvelopeMinMaxTree::default);
        }
        for channel in 0..channel_count {
            let Some(samples) = callback_samples.get(channel) else {
                continue;
            };
            self.trees[channel].rebuild_from_slice(&samples[..sample_count.min(samples.len())]);
        }

        self.revision = Some(sample_revision);
        self.sample_count = sample_count;
        self.channel_count = channel_count;
    }
}
