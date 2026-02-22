//! Mode-specific waveform channel rendering helpers.

use super::context::WaveformRenderScratch;
use super::sampling::{
    EnvelopeMinMaxTree, resample_channel_linear_from_slice_into, resample_channel_linear_into,
};
use super::{
    Color, LaneBounds, SurfaceCommand, WaveformGeometry, WaveformRenderQuality,
    WaveformSamplingMode, WaveformViewStyle,
};

mod contour;
mod envelope;
mod glow;

use self::contour::{emit_polyline, emit_polyline_batched, samples_to_points};
use self::envelope::{
    draw_waveform_channel_envelope_legacy, draw_waveform_channel_envelope_styled,
};
use self::glow::{emit_glow_symmetric, resolve_glow_plan, with_alpha};

/// Draw one waveform channel using the requested sampling mode.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_waveform_channel<SampleAt>(
    commands: &mut Vec<SurfaceCommand>,
    geometry: &WaveformGeometry,
    sample_count: usize,
    channel: usize,
    channel_samples: Option<&[f32]>,
    sample_at: &SampleAt,
    envelope_tree: Option<&EnvelopeMinMaxTree>,
    sampling_mode: WaveformSamplingMode,
    render_quality: WaveformRenderQuality,
    style: WaveformViewStyle,
    zoom_y: f32,
    start_sample: u64,
    lane: LaneBounds,
    color: Color,
    channel_command_budget: usize,
    glow_point_budget: usize,
    envelope_temporal_smoothing: bool,
    envelope_release_ms_per_pixel: f32,
    envelope_frame_delta_ms: f32,
    scratch: &mut WaveformRenderScratch,
) where
    SampleAt: Fn(usize, usize) -> f32,
{
    let lane_height = (lane.bottom - lane.top).max(1);
    let center_y = lane.top + lane_height / 2;
    let scale_y = (lane_height as f32 * 0.45) / zoom_y.max(0.05);

    match (sampling_mode, render_quality) {
        (WaveformSamplingMode::Linear, WaveformRenderQuality::LegacyCpuOnly) => {
            draw_waveform_channel_linear_legacy(
                commands,
                geometry,
                sample_count,
                channel,
                channel_samples,
                sample_at,
                lane,
                color,
                center_y,
                scale_y,
                start_sample,
                scratch,
            )
        }
        (WaveformSamplingMode::EnvelopeMinMax, WaveformRenderQuality::LegacyCpuOnly) => {
            draw_waveform_channel_envelope_legacy(
                commands,
                geometry,
                sample_count,
                channel,
                channel_samples,
                sample_at,
                envelope_tree,
                lane,
                color,
                center_y,
                scale_y,
                start_sample,
                envelope_temporal_smoothing,
                envelope_release_ms_per_pixel,
                envelope_frame_delta_ms,
                scratch,
            )
        }
        (WaveformSamplingMode::Linear, WaveformRenderQuality::AutoVectorPreferred) => {
            draw_waveform_channel_linear_styled(
                commands,
                geometry,
                sample_count,
                channel,
                channel_samples,
                sample_at,
                lane,
                color,
                style,
                center_y,
                scale_y,
                start_sample,
                channel_command_budget,
                glow_point_budget,
                scratch,
            )
        }
        (WaveformSamplingMode::EnvelopeMinMax, WaveformRenderQuality::AutoVectorPreferred) => {
            draw_waveform_channel_envelope_styled(
                commands,
                geometry,
                sample_count,
                channel,
                channel_samples,
                sample_at,
                envelope_tree,
                lane,
                color,
                style,
                center_y,
                scale_y,
                start_sample,
                channel_command_budget,
                glow_point_budget,
                envelope_temporal_smoothing,
                envelope_release_ms_per_pixel,
                envelope_frame_delta_ms,
                scratch,
            )
        }
    }
}

/// Draw one channel with linearly interpolated polyline sampling.
#[allow(clippy::too_many_arguments)]
fn draw_waveform_channel_linear_legacy<SampleAt>(
    commands: &mut Vec<SurfaceCommand>,
    geometry: &WaveformGeometry,
    sample_count: usize,
    channel: usize,
    channel_samples: Option<&[f32]>,
    sample_at: &SampleAt,
    lane: LaneBounds,
    color: Color,
    center_y: i32,
    scale_y: f32,
    start_sample: u64,
    scratch: &mut WaveformRenderScratch,
) where
    SampleAt: Fn(usize, usize) -> f32,
{
    let points = geometry.width_i32.max(2) as usize;
    linear_samples_into(
        sample_count,
        channel,
        channel_samples,
        sample_at,
        points,
        start_sample,
        &mut scratch.linear_samples,
    );
    if scratch.linear_samples.len() < 2 {
        return;
    }
    samples_to_points(
        &scratch.linear_samples,
        geometry.width_i32.max(2) - 1,
        lane,
        center_y,
        scale_y,
        &mut scratch.contour,
    );
    emit_polyline(commands, &scratch.contour, color);
}

/// Draw one linearly sampled channel with an outline glow treatment.
#[allow(clippy::too_many_arguments)]
fn draw_waveform_channel_linear_styled<SampleAt>(
    commands: &mut Vec<SurfaceCommand>,
    geometry: &WaveformGeometry,
    sample_count: usize,
    channel: usize,
    channel_samples: Option<&[f32]>,
    sample_at: &SampleAt,
    lane: LaneBounds,
    color: Color,
    style: WaveformViewStyle,
    center_y: i32,
    scale_y: f32,
    start_sample: u64,
    channel_command_budget: usize,
    glow_point_budget: usize,
    scratch: &mut WaveformRenderScratch,
) where
    SampleAt: Fn(usize, usize) -> f32,
{
    let points = geometry.width_i32.max(2) as usize;
    linear_samples_into(
        sample_count,
        channel,
        channel_samples,
        sample_at,
        points,
        start_sample,
        &mut scratch.linear_samples,
    );
    if scratch.linear_samples.len() < 2 {
        return;
    }

    samples_to_points(
        &scratch.linear_samples,
        geometry.width_i32.max(2) - 1,
        lane,
        center_y,
        scale_y,
        &mut scratch.contour,
    );
    let core_color = with_alpha(color, style.waveform_outline_alpha_inner);
    let glow_budget = channel_command_budget.saturating_sub(1);
    let glow_plan = resolve_glow_plan(
        scratch.contour.len(),
        style,
        glow_budget,
        glow_point_budget,
        2,
    );
    emit_glow_symmetric(
        commands,
        &scratch.contour,
        lane,
        color,
        style,
        glow_plan,
        &mut scratch.shifted,
    );
    emit_polyline_batched(commands, &scratch.contour, core_color, 1.0);
}

/// Fill linear resample output for one channel.
fn linear_samples_into<SampleAt>(
    sample_count: usize,
    channel: usize,
    channel_samples: Option<&[f32]>,
    sample_at: &SampleAt,
    points: usize,
    _start_sample: u64,
    out: &mut Vec<f32>,
) where
    SampleAt: Fn(usize, usize) -> f32,
{
    if let Some(samples) = channel_samples {
        let bounded = &samples[..sample_count.min(samples.len())];
        resample_channel_linear_from_slice_into(bounded, points, out);
    } else {
        resample_channel_linear_into(sample_count, channel, points, sample_at, out);
    }
}
