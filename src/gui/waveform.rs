//! Reusable waveform view renderer for declarative surface widgets.
//!
//! The API builds `SurfaceCommand` vectors from sample accessors so plugins can
//! render high-frequency waveform views without duplicating draw logic.

use super::Color;
use super::declarative::SurfaceCommand;
use super::{Point, Rect, Size};

mod build;
mod context;
mod grid;
mod render;
mod sampling;

#[cfg(test)]
mod tests;

pub use self::context::WaveformRenderContext;
/// Internal grid-tone classifier shared by grid generation and tests.
type GridTone = grid::GridTone;

/// Channel arrangement mode for waveform rendering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaveformDisplayMode {
    /// Draw all visible channels in a single shared lane.
    Overlay,
    /// Draw each visible channel in its own vertical lane.
    Split,
}

/// Sampling strategy used to map audio samples onto horizontal pixels.
///
/// `Linear` draws a polyline through interpolated sample points. It preserves
/// exact local curve detail at low sample densities but can shimmer when many
/// source samples are projected into a narrow pixel width.
///
/// `EnvelopeMinMax` computes one min/max sample envelope per x-column and draws
/// a deterministic vertical segment for each column. This improves visual
/// stability for dense, periodic signals and preserves transient peaks.
///
/// Example:
///
/// ```ignore
/// let mut config = WaveformViewConfig::new(&channels);
/// config.sampling_mode = WaveformSamplingMode::EnvelopeMinMax;
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaveformSamplingMode {
    /// Draw a polyline through linearly interpolated samples.
    Linear,
    /// Draw one vertical min/max envelope segment per x-column.
    EnvelopeMinMax,
}

/// Rendering quality policy for waveform command generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaveformRenderQuality {
    /// Emit enhanced outline/glow commands intended for vector-backed rendering.
    AutoVectorPreferred,
    /// Emit legacy line-only commands for strict CPU compatibility.
    LegacyCpuOnly,
}

/// Grid configuration for waveform rendering.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WaveformGridMode {
    /// Draw an evenly spaced fixed grid independent of host tempo.
    Fixed {
        /// Number of vertical divisions across the full width.
        line_count: u32,
    },
    /// Draw a beat-domain grid anchored to a visible musical span.
    TempoLocked {
        /// Number of beats represented across the full width.
        beats_visible: f64,
        /// Number of beats in one musical bar.
        beats_per_bar: f64,
        /// Number of subdivisions within one beat.
        subdivisions_per_beat: u32,
        /// Absolute beat position at the left edge (`x = 0`).
        ///
        /// Pass the host-resolved transport beat for the visible window start
        /// so grid placement stays phase-locked across long sessions, loop
        /// wraps, and tempo changes.
        start_beat: f64,
    },
}

/// Visual style for one waveform channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WaveformChannelStyle {
    /// Whether the channel is visible.
    pub visible: bool,
    /// Stroke color used for waveform channel rendering.
    pub color: Color,
}

/// Visual style tokens for the waveform surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WaveformViewStyle {
    /// Surface background color.
    pub background: Color,
    /// Bar-aligned vertical grid line color.
    pub grid_bar: Color,
    /// Beat-aligned vertical grid line color.
    pub grid_beat: Color,
    /// Subdivision vertical grid line color.
    pub grid_subdivision: Color,
    /// Non-center horizontal grid line color.
    pub grid_horizontal: Color,
    /// Center horizontal line color.
    pub grid_horizontal_center: Color,
    /// Split-lane divider color.
    pub lane_divider: Color,
    /// Alpha used for the waveform body fill.
    pub waveform_body_alpha: u8,
    /// Alpha used at the inner edge of the waveform outline glow.
    pub waveform_outline_alpha_inner: u8,
    /// Alpha used at the outer edge of the waveform outline glow.
    pub waveform_outline_alpha_outer: u8,
    /// Number of glow layers emitted around each contour.
    pub waveform_outline_layers: u8,
}

impl Default for WaveformViewStyle {
    fn default() -> Self {
        Self {
            background: Color::rgb(14, 17, 20),
            grid_bar: Color::rgb(44, 50, 57),
            grid_beat: Color::rgb(36, 41, 47),
            grid_subdivision: Color::rgb(30, 35, 41),
            grid_horizontal: Color::rgb(27, 31, 37),
            grid_horizontal_center: Color::rgb(36, 40, 45),
            lane_divider: Color::rgb(42, 48, 54),
            waveform_body_alpha: 72,
            waveform_outline_alpha_inner: 210,
            waveform_outline_alpha_outer: 0,
            waveform_outline_layers: 4,
        }
    }
}

/// Full configuration for waveform surface command generation.
#[derive(Clone, Debug, PartialEq)]
pub struct WaveformViewConfig<'a> {
    /// Channel rendering mode.
    pub display_mode: WaveformDisplayMode,
    /// Sample-to-column mapping mode.
    pub sampling_mode: WaveformSamplingMode,
    /// Rendering quality policy for contour command generation.
    pub render_quality: WaveformRenderQuality,
    /// Vertical zoom multiplier applied to sample amplitude.
    pub zoom_y: f32,
    /// Per-channel visibility and color styles.
    pub channels: &'a [WaveformChannelStyle],
    /// Vertical grid behavior.
    pub grid_mode: WaveformGridMode,
    /// Absolute sample index at the left edge (`x = 0`).
    ///
    /// Envelope-mode bin boundaries are phase-aligned against this index so
    /// high-density scrolling remains visually stable across frames.
    pub start_sample: u64,
    /// Number of horizontal divisions including top and bottom lines.
    pub horizontal_grid_lines: u32,
    /// Global waveform surface style.
    pub style: WaveformViewStyle,
    /// Upper bound on waveform draw commands emitted per surface.
    ///
    /// The renderer uses this as a deterministic quality budget. When command
    /// pressure is high, glow layers are reduced first while preserving core
    /// contours and envelope body segments.
    pub max_waveform_commands: usize,
    /// Upper bound on glow polyline points emitted per channel.
    ///
    /// Larger values preserve smoother halos but increase vector path cost.
    /// Lower values increase polyline stride for outer glow layers.
    pub max_glow_points_per_channel: usize,
    /// Enable temporal smoothing for envelope contour motion.
    pub envelope_temporal_smoothing: bool,
    /// Milliseconds required for one inward release pixel in envelope smoothing.
    ///
    /// Lower values release toward the center faster; higher values retain
    /// transients longer. This only affects inward motion. Outward expansion
    /// remains immediate to preserve attack peaks.
    pub envelope_release_ms_per_pixel: f32,
    /// Frame time in milliseconds used by envelope temporal smoothing.
    ///
    /// Callers should update this from UI frame timing so smoothing speed
    /// remains stable across variable refresh rates.
    pub envelope_frame_delta_ms: f32,
}

impl<'a> WaveformViewConfig<'a> {
    /// Build a default-configured view with explicit channel styles.
    pub fn new(channels: &'a [WaveformChannelStyle]) -> Self {
        Self {
            display_mode: WaveformDisplayMode::Overlay,
            sampling_mode: WaveformSamplingMode::EnvelopeMinMax,
            render_quality: WaveformRenderQuality::AutoVectorPreferred,
            zoom_y: 1.0,
            channels,
            grid_mode: WaveformGridMode::Fixed { line_count: 8 },
            start_sample: 0,
            horizontal_grid_lines: 8,
            style: WaveformViewStyle::default(),
            max_waveform_commands: 32_768,
            max_glow_points_per_channel: 16_384,
            envelope_temporal_smoothing: true,
            envelope_release_ms_per_pixel: 16.0,
            envelope_frame_delta_ms: 16.0,
        }
    }
}

/// Generate declarative draw commands for one waveform surface.
///
/// `sample_at(channel, index)` must return a normalized sample in `[-1.0, 1.0]`
/// (values are internally clamped to `[-1.2, 1.2]`).
pub fn build_waveform_surface_commands<SampleAt>(
    width: u32,
    height: u32,
    sample_count: usize,
    channel_count: usize,
    sample_at: SampleAt,
    config: &WaveformViewConfig<'_>,
) -> Vec<SurfaceCommand>
where
    SampleAt: Fn(usize, usize) -> f32,
{
    let mut context = WaveformRenderContext::default();
    build_waveform_surface_commands_with_context(
        width,
        height,
        sample_count,
        channel_count,
        sample_at,
        0,
        config,
        &mut context,
    )
}

/// Generate waveform surface commands using a reusable render context.
///
/// Pass a monotonic `sample_revision` that changes whenever source samples
/// change. Stable revisions let the renderer reuse cached envelope trees and
/// callback materialization buffers across frames.
#[allow(clippy::too_many_arguments)]
pub fn build_waveform_surface_commands_with_context<SampleAt>(
    width: u32,
    height: u32,
    sample_count: usize,
    channel_count: usize,
    sample_at: SampleAt,
    sample_revision: u64,
    config: &WaveformViewConfig<'_>,
    context: &mut WaveformRenderContext,
) -> Vec<SurfaceCommand>
where
    SampleAt: Fn(usize, usize) -> f32,
{
    context.ensure_callback_samples(sample_revision, sample_count, channel_count, &sample_at);
    if matches!(config.sampling_mode, WaveformSamplingMode::EnvelopeMinMax) {
        context.ensure_envelope_cache_from_callback(sample_revision, sample_count, channel_count);
    }

    let mut scratch = context.take_scratch();
    let mut channel_samples = Vec::with_capacity(channel_count);
    for channel in 0..channel_count {
        channel_samples.push(context.callback_channel_samples(channel).unwrap_or(&[]));
    }
    let commands = build::build_waveform_surface_commands_from_channel_slices(
        width,
        height,
        sample_count,
        channel_count,
        &channel_samples,
        &sample_at,
        config,
        context,
        &mut scratch,
    );
    context.restore_scratch(scratch);
    commands
}

/// Generate waveform surface commands from channel sample slices.
///
/// This avoids callback dispatch in the hot render loop. The function uses one
/// temporary context and does not persist cache state across calls.
pub fn build_waveform_surface_commands_from_slices(
    width: u32,
    height: u32,
    channel_samples: &[&[f32]],
    config: &WaveformViewConfig<'_>,
) -> Vec<SurfaceCommand> {
    let mut context = WaveformRenderContext::default();
    build_waveform_surface_commands_from_slices_with_context(
        width,
        height,
        channel_samples,
        0,
        config,
        &mut context,
    )
}

/// Generate waveform surface commands from slices with reusable cache context.
///
/// Pass a stable `sample_revision` to skip multiresolution envelope rebuilds
/// when sample content is unchanged.
pub fn build_waveform_surface_commands_from_slices_with_context(
    width: u32,
    height: u32,
    channel_samples: &[&[f32]],
    sample_revision: u64,
    config: &WaveformViewConfig<'_>,
    context: &mut WaveformRenderContext,
) -> Vec<SurfaceCommand> {
    let sample_count = build::min_channel_sample_count(channel_samples);
    let channel_count = channel_samples.len();
    if matches!(config.sampling_mode, WaveformSamplingMode::EnvelopeMinMax) {
        context.ensure_envelope_cache_from_slices(
            sample_revision,
            sample_count,
            channel_samples,
            channel_count,
        );
    }
    let mut scratch = context.take_scratch();
    let commands = build::build_waveform_surface_commands_from_channel_slices(
        width,
        height,
        sample_count,
        channel_count,
        channel_samples,
        &|_, _| 0.0,
        config,
        context,
        &mut scratch,
    );
    context.restore_scratch(scratch);
    commands
}

/// Pixel-space geometry for one waveform surface.
#[derive(Clone, Copy, Debug)]
pub(super) struct WaveformGeometry {
    /// Surface width in pixels.
    pub(super) width_i32: i32,
    /// Surface height in pixels.
    pub(super) height_i32: i32,
}

impl WaveformGeometry {
    /// Build geometry from unsigned surface dimensions.
    pub(super) fn new(width: u32, height: u32) -> Self {
        Self {
            width_i32: width.max(1) as i32,
            height_i32: height.max(1) as i32,
        }
    }
}

/// Vertical lane bounds for one rendered channel.
#[derive(Clone, Copy, Debug)]
pub(super) struct LaneBounds {
    /// Inclusive lane top pixel.
    pub(super) top: i32,
    /// Inclusive lane bottom pixel.
    pub(super) bottom: i32,
}

impl LaneBounds {
    /// Build bounds spanning the full surface.
    pub(super) fn full(geometry: &WaveformGeometry) -> Self {
        Self {
            top: 0,
            bottom: geometry.height_i32,
        }
    }

    /// Build bounds for one split lane.
    pub(super) fn for_split_lane(
        geometry: &WaveformGeometry,
        lane_index: i32,
        lane_count: i32,
    ) -> Self {
        let top = lane_index * geometry.height_i32 / lane_count.max(1);
        let bottom = ((lane_index + 1) * geometry.height_i32 / lane_count.max(1))
            .clamp(top + 1, geometry.height_i32);
        Self { top, bottom }
    }
}
