//! Mode-specific waveform channel rendering helpers.

use super::sampling::{clamp_sample, resample_channel_linear, sample_envelope_min_max_for_columns};
use super::{Color, LaneBounds, Point, SurfaceCommand, WaveformGeometry, WaveformSamplingMode};

/// Draw one waveform channel using the requested sampling mode.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_waveform_channel<SampleAt>(
    commands: &mut Vec<SurfaceCommand>,
    geometry: &WaveformGeometry,
    sample_count: usize,
    channel: usize,
    sample_at: &SampleAt,
    sampling_mode: WaveformSamplingMode,
    zoom_y: f32,
    lane: LaneBounds,
    color: Color,
) where
    SampleAt: Fn(usize, usize) -> f32,
{
    let lane_height = (lane.bottom - lane.top).max(1);
    let center_y = lane.top + lane_height / 2;
    let scale_y = (lane_height as f32 * 0.45) / zoom_y.max(0.05);

    match sampling_mode {
        WaveformSamplingMode::Linear => draw_waveform_channel_linear(
            commands,
            geometry,
            sample_count,
            channel,
            sample_at,
            lane,
            color,
            center_y,
            scale_y,
        ),
        WaveformSamplingMode::EnvelopeMinMax => draw_waveform_channel_envelope_min_max(
            commands,
            geometry,
            sample_count,
            channel,
            sample_at,
            lane,
            color,
            center_y,
            scale_y,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
/// Draw one channel with linearly interpolated polyline sampling.
fn draw_waveform_channel_linear<SampleAt>(
    commands: &mut Vec<SurfaceCommand>,
    geometry: &WaveformGeometry,
    sample_count: usize,
    channel: usize,
    sample_at: &SampleAt,
    lane: LaneBounds,
    color: Color,
    center_y: i32,
    scale_y: f32,
) where
    SampleAt: Fn(usize, usize) -> f32,
{
    let points = geometry.width_i32.max(2) as usize;
    let samples = resample_channel_linear(sample_count, channel, points, sample_at);
    if samples.len() < 2 {
        return;
    }

    let x_max = geometry.width_i32.max(2) - 1;
    let mut prev = None;

    for (point_index, sample) in samples.iter().enumerate() {
        let x = point_x(point_index, points, x_max);
        let y = sample_to_lane_y(*sample, center_y, scale_y, lane);
        let current = Point { x, y };

        if let Some(previous) = prev {
            commands.push(SurfaceCommand::Line {
                start: previous,
                end: current,
                color,
            });
        }
        prev = Some(current);
    }
}

#[allow(clippy::too_many_arguments)]
/// Draw one channel using deterministic per-column min/max envelope segments.
fn draw_waveform_channel_envelope_min_max<SampleAt>(
    commands: &mut Vec<SurfaceCommand>,
    geometry: &WaveformGeometry,
    sample_count: usize,
    channel: usize,
    sample_at: &SampleAt,
    lane: LaneBounds,
    color: Color,
    center_y: i32,
    scale_y: f32,
) where
    SampleAt: Fn(usize, usize) -> f32,
{
    let columns = geometry.width_i32.max(2) as usize;
    let bins = sample_envelope_min_max_for_columns(sample_count, channel, columns, sample_at);
    if bins.is_empty() {
        return;
    }

    let x_max = geometry.width_i32.max(2) - 1;
    for (column_index, (min_sample, max_sample)) in bins.iter().enumerate() {
        let x = point_x(column_index, columns, x_max);
        let y_top = sample_to_lane_y(*max_sample, center_y, scale_y, lane);
        let y_bottom = sample_to_lane_y(*min_sample, center_y, scale_y, lane);
        commands.push(SurfaceCommand::Line {
            start: Point { x, y: y_top },
            end: Point { x, y: y_bottom },
            color,
        });
    }
}

/// Map one point index to a rounded pixel-space x coordinate.
fn point_x(point_index: usize, points: usize, x_max: i32) -> i32 {
    let normalized = point_index as f32 / (points - 1) as f32;
    (normalized * x_max as f32).round() as i32
}

/// Convert one normalized sample value into lane Y coordinates.
fn sample_to_lane_y(sample: f32, center_y: i32, scale_y: f32, lane: LaneBounds) -> i32 {
    ((center_y as f32 - clamp_sample(sample) * scale_y).round() as i32).clamp(lane.top, lane.bottom)
}
