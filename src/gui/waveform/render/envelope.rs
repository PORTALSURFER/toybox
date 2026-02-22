//! Envelope min/max rendering and temporal smoothing helpers.

use super::super::context::{
    EnvelopeMotionSignature, WaveformEnvelopeMotionState, WaveformRenderScratch,
};
use super::super::sampling::{EnvelopeMinMaxTree, SAMPLE_CLAMP_LIMIT, clamp_sample};
use super::super::{
    Color, LaneBounds, Point, Rect, Size, SurfaceCommand, WaveformGeometry, WaveformViewStyle,
};
use super::contour::{emit_polyline_batched, point_x, sample_to_lane_y};
use super::glow::{emit_glow_outward, resolve_glow_plan, with_alpha};

/// Draw one channel using deterministic per-column min/max envelope segments.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_waveform_channel_envelope_legacy<SampleAt>(
    commands: &mut Vec<SurfaceCommand>,
    geometry: &WaveformGeometry,
    sample_count: usize,
    channel: usize,
    channel_samples: Option<&[f32]>,
    sample_at: &SampleAt,
    envelope_tree: Option<&EnvelopeMinMaxTree>,
    lane: LaneBounds,
    color: Color,
    center_y: i32,
    scale_y: f32,
    start_sample: u64,
    envelope_temporal_smoothing: bool,
    envelope_release_ms_per_pixel: f32,
    envelope_frame_delta_ms: f32,
    scratch: &mut WaveformRenderScratch,
) where
    SampleAt: Fn(usize, usize) -> f32,
{
    let columns = geometry.width_i32.max(2) as usize;
    let x_max = geometry.width_i32.max(2) - 1;
    {
        let (top_contour, bottom_contour, phase_bounds_cache) = (
            &mut scratch.top_contour,
            &mut scratch.bottom_contour,
            &mut scratch.phase_bounds_cache,
        );
        top_contour.clear();
        bottom_contour.clear();
        top_contour.reserve(columns);
        bottom_contour.reserve(columns);

        let column_bounds = phase_bounds_cache.bounds(sample_count, columns, start_sample);
        for_each_channel_envelope_column(
            channel,
            column_bounds,
            channel_samples,
            sample_at,
            envelope_tree,
            |column_index, min_sample, max_sample| {
                let x = point_x(column_index, columns, x_max);
                let y_top = sample_to_lane_y(max_sample, center_y, scale_y, lane);
                let y_bottom = sample_to_lane_y(min_sample, center_y, scale_y, lane);
                top_contour.push(Point { x, y: y_top });
                bottom_contour.push(Point { x, y: y_bottom });
            },
        );
    }

    if envelope_temporal_smoothing {
        let mut motion_state = scratch.take_envelope_motion_state(channel);
        smooth_envelope_contours(
            &mut scratch.top_contour,
            &mut scratch.bottom_contour,
            lane,
            center_y,
            scale_y,
            envelope_release_ms_per_pixel,
            envelope_frame_delta_ms,
            &mut motion_state,
        );
        scratch.restore_envelope_motion_state(channel, motion_state);
    } else {
        scratch.restore_envelope_motion_state(channel, WaveformEnvelopeMotionState::default());
    }

    for (top, bottom) in scratch.top_contour.iter().zip(&scratch.bottom_contour) {
        if top.y == bottom.y {
            commands.push(SurfaceCommand::FillRect {
                rect: Rect {
                    origin: *top,
                    size: Size {
                        width: 1,
                        height: 1,
                    },
                },
                color,
            });
        } else {
            commands.push(SurfaceCommand::Line {
                start: *top,
                end: *bottom,
                color,
            });
        }
    }
}

/// Draw one min/max envelope channel with body fill and gradient-like outlines.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_waveform_channel_envelope_styled<SampleAt>(
    commands: &mut Vec<SurfaceCommand>,
    geometry: &WaveformGeometry,
    sample_count: usize,
    channel: usize,
    channel_samples: Option<&[f32]>,
    sample_at: &SampleAt,
    envelope_tree: Option<&EnvelopeMinMaxTree>,
    lane: LaneBounds,
    color: Color,
    style: WaveformViewStyle,
    center_y: i32,
    scale_y: f32,
    start_sample: u64,
    channel_command_budget: usize,
    glow_point_budget: usize,
    envelope_temporal_smoothing: bool,
    envelope_release_ms_per_pixel: f32,
    envelope_frame_delta_ms: f32,
    scratch: &mut WaveformRenderScratch,
) where
    SampleAt: Fn(usize, usize) -> f32,
{
    let columns = geometry.width_i32.max(2) as usize;
    let x_max = geometry.width_i32.max(2) - 1;
    let body_color = with_alpha(color, style.waveform_body_alpha);
    {
        let (top_contour, bottom_contour, phase_bounds_cache) = (
            &mut scratch.top_contour,
            &mut scratch.bottom_contour,
            &mut scratch.phase_bounds_cache,
        );
        top_contour.clear();
        bottom_contour.clear();
        top_contour.reserve(columns);
        bottom_contour.reserve(columns);

        let column_bounds = phase_bounds_cache.bounds(sample_count, columns, start_sample);
        for_each_channel_envelope_column(
            channel,
            column_bounds,
            channel_samples,
            sample_at,
            envelope_tree,
            |column_index, min_sample, max_sample| {
                let x = point_x(column_index, columns, x_max);
                let y_top = sample_to_lane_y(max_sample, center_y, scale_y, lane);
                let y_bottom = sample_to_lane_y(min_sample, center_y, scale_y, lane);
                if body_color.a > 0 {
                    if y_top != y_bottom {
                        commands.push(SurfaceCommand::Line {
                            start: Point { x, y: y_top },
                            end: Point { x, y: y_bottom },
                            color: body_color,
                        });
                    } else {
                        commands.push(SurfaceCommand::FillRect {
                            rect: Rect {
                                origin: Point { x, y: y_top },
                                size: Size {
                                    width: 1,
                                    height: 1,
                                },
                            },
                            color: body_color,
                        });
                    }
                }
                top_contour.push(Point { x, y: y_top });
                bottom_contour.push(Point { x, y: y_bottom });
            },
        );
    }

    if scratch.top_contour.len() < 2 || scratch.bottom_contour.len() < 2 {
        return;
    }

    if envelope_temporal_smoothing {
        let mut motion_state = scratch.take_envelope_motion_state(channel);
        smooth_envelope_contours(
            &mut scratch.top_contour,
            &mut scratch.bottom_contour,
            lane,
            center_y,
            scale_y,
            envelope_release_ms_per_pixel,
            envelope_frame_delta_ms,
            &mut motion_state,
        );
        scratch.restore_envelope_motion_state(channel, motion_state);
    } else {
        scratch.restore_envelope_motion_state(channel, WaveformEnvelopeMotionState::default());
    }

    let core_commands = 2usize;
    // Keep glow quality stable for fixed geometry/config by budgeting against
    // deterministic envelope column count instead of frame-varying body draws.
    let glow_budget = channel_command_budget.saturating_sub(columns + core_commands);
    let per_contour_glow_points = glow_point_budget / 2;
    let glow_plan = resolve_glow_plan(
        scratch.top_contour.len(),
        style,
        glow_budget,
        per_contour_glow_points.max(1),
        2,
    );
    emit_glow_outward(
        commands,
        &scratch.top_contour,
        lane,
        color,
        style,
        -1,
        glow_plan,
        &mut scratch.shifted,
    );
    emit_glow_outward(
        commands,
        &scratch.bottom_contour,
        lane,
        color,
        style,
        1,
        glow_plan,
        &mut scratch.shifted,
    );
    let core_color = with_alpha(color, style.waveform_outline_alpha_inner);
    emit_polyline_batched(commands, &scratch.top_contour, core_color, 1.0);
    emit_polyline_batched(commands, &scratch.bottom_contour, core_color, 1.0);
}

/// Iterate min/max envelopes for one channel using the best available source.
#[allow(clippy::too_many_arguments)]
fn for_each_channel_envelope_column<SampleAt, Visit>(
    channel: usize,
    column_bounds: &[(usize, usize)],
    channel_samples: Option<&[f32]>,
    sample_at: &SampleAt,
    envelope_tree: Option<&EnvelopeMinMaxTree>,
    mut visit: Visit,
) where
    SampleAt: Fn(usize, usize) -> f32,
    Visit: FnMut(usize, f32, f32),
{
    if let Some(tree) = envelope_tree {
        for (column, (start, end)) in column_bounds.iter().copied().enumerate() {
            let bin = tree.query_range(start, end);
            visit(column, bin.min, bin.max);
        }
        return;
    }
    if let Some(samples) = channel_samples {
        for (column, (start, end)) in column_bounds.iter().copied().enumerate() {
            let mut min_sample = SAMPLE_CLAMP_LIMIT;
            let mut max_sample = -SAMPLE_CLAMP_LIMIT;
            for sample in &samples[start..end] {
                let sample = clamp_sample(*sample);
                if sample < min_sample {
                    min_sample = sample;
                }
                if sample > max_sample {
                    max_sample = sample;
                }
            }
            if min_sample > max_sample {
                let fallback = clamp_sample(samples[start]);
                min_sample = fallback;
                max_sample = fallback;
            }
            visit(column, min_sample, max_sample);
        }
        return;
    }
    for (column, (start, end)) in column_bounds.iter().copied().enumerate() {
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
        visit(column, min_sample, max_sample);
    }
}

/// Apply temporal smoothing to top/bottom envelope contours.
#[allow(clippy::too_many_arguments)]
fn smooth_envelope_contours(
    top_contour: &mut [Point],
    bottom_contour: &mut [Point],
    lane: LaneBounds,
    center_y: i32,
    scale_y: f32,
    release_ms_per_pixel: f32,
    frame_delta_ms: f32,
    state: &mut WaveformEnvelopeMotionState,
) {
    if top_contour.len() != bottom_contour.len() || top_contour.is_empty() {
        return;
    }

    let signature = EnvelopeMotionSignature {
        columns: top_contour.len(),
        lane_top: lane.top,
        lane_bottom: lane.bottom,
        center_y,
        scale_key: (scale_y * 1024.0).round() as i32,
        release_ms_per_pixel_key: (release_ms_per_pixel * 256.0).round() as i32,
        frame_delta_ms_key: (frame_delta_ms * 256.0).round() as i32,
    };
    let requires_reset = state.signature != Some(signature)
        || state.top_y.len() != top_contour.len()
        || state.bottom_y.len() != bottom_contour.len();
    if requires_reset {
        state.signature = Some(signature);
        state.top_y.clear();
        state.bottom_y.clear();
        state.top_y.reserve(top_contour.len());
        state.bottom_y.reserve(bottom_contour.len());
        for (top, bottom) in top_contour.iter().zip(bottom_contour.iter()) {
            state.top_y.push(top.y as f32);
            state.bottom_y.push(bottom.y as f32);
        }
        return;
    }

    let release_step = if release_ms_per_pixel <= 0.0 {
        f32::MAX
    } else {
        (frame_delta_ms.max(0.0) / release_ms_per_pixel).max(0.0)
    };
    for index in 0..top_contour.len() {
        let smoothed_top = smooth_edge(
            state.top_y[index],
            top_contour[index].y as f32,
            false,
            release_step,
        );
        let smoothed_bottom = smooth_edge(
            state.bottom_y[index],
            bottom_contour[index].y as f32,
            true,
            release_step,
        );

        let clamped_top = smoothed_top.clamp(lane.top as f32, lane.bottom as f32);
        let clamped_bottom = smoothed_bottom.clamp(lane.top as f32, lane.bottom as f32);
        let rendered_top = clamped_top.round() as i32;
        let rendered_bottom = clamped_bottom.round() as i32;

        top_contour[index].y = rendered_top;
        bottom_contour[index].y = rendered_bottom;
        state.top_y[index] = clamped_top;
        state.bottom_y[index] = clamped_bottom;
    }
}

/// Smooth one envelope edge with fast outward attack and limited inward release.
fn smooth_edge(prev: f32, target: f32, outward_positive: bool, release_step: f32) -> f32 {
    if prev == target {
        return target;
    }
    let moving_outward = if outward_positive {
        target > prev
    } else {
        target < prev
    };
    if moving_outward {
        return target;
    }
    if release_step.is_infinite() {
        return target;
    }
    let bounded_step = release_step.max(0.0);
    if target > prev {
        (prev + bounded_step).min(target)
    } else {
        (prev - bounded_step).max(target)
    }
}
