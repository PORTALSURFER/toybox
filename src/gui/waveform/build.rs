//! Internal command-building pipeline for waveform rendering.

use super::context;
use super::grid::vertical_grid_lines;
use super::render::draw_waveform_channel;
use super::{
    GridTone, LaneBounds, Point, Rect, Size, SurfaceCommand, WaveformDisplayMode, WaveformGeometry,
    WaveformRenderContext, WaveformRenderQuality, WaveformSamplingMode, WaveformViewConfig,
    WaveformViewStyle,
};

/// Shared waveform command builder using channel sample slices.
#[allow(clippy::too_many_arguments)]
pub(super) fn build_waveform_surface_commands_from_channel_slices<SampleAt>(
    width: u32,
    height: u32,
    sample_count: usize,
    channel_count: usize,
    channel_samples: &[&[f32]],
    sample_at: &SampleAt,
    config: &WaveformViewConfig<'_>,
    context: &WaveformRenderContext,
    scratch: &mut context::WaveformRenderScratch,
) -> Vec<SurfaceCommand>
where
    SampleAt: Fn(usize, usize) -> f32,
{
    let geometry = WaveformGeometry::new(width, height);
    let mut commands = Vec::new();

    push_background(&mut commands, &geometry, config.style);
    push_vertical_grid(&mut commands, &geometry, config);
    push_horizontal_grid(&mut commands, &geometry, config);

    if sample_count < 2 {
        return commands;
    }

    let visible_channels = visible_channel_indices(channel_count, config.channels);
    if visible_channels.is_empty() {
        return commands;
    }

    commands.reserve(estimate_waveform_command_capacity(
        &geometry,
        sample_count,
        visible_channels.len(),
        config,
    ));

    match config.display_mode {
        WaveformDisplayMode::Overlay => {
            for (channel_index, channel) in visible_channels.iter().copied().enumerate() {
                let channel_budget = channel_waveform_budget(
                    config.max_waveform_commands,
                    visible_channels.len(),
                    channel_index,
                );
                draw_waveform_channel(
                    &mut commands,
                    &geometry,
                    sample_count,
                    channel,
                    channel_samples.get(channel).copied(),
                    sample_at,
                    context.envelope_tree(channel),
                    config.sampling_mode,
                    config.render_quality,
                    config.style,
                    config.zoom_y,
                    config.start_sample,
                    LaneBounds::full(&geometry),
                    config.channels[channel].color,
                    channel_budget,
                    config.max_glow_points_per_channel.max(1),
                    config.envelope_temporal_smoothing,
                    config.envelope_release_ms_per_pixel,
                    config.envelope_frame_delta_ms,
                    scratch,
                );
            }
        }
        WaveformDisplayMode::Split => {
            let lane_count = visible_channels.len().max(1) as i32;
            for (lane_index, channel) in visible_channels.iter().enumerate() {
                let channel_budget = channel_waveform_budget(
                    config.max_waveform_commands,
                    visible_channels.len(),
                    lane_index,
                );
                let lane = LaneBounds::for_split_lane(&geometry, lane_index as i32, lane_count);
                draw_waveform_channel(
                    &mut commands,
                    &geometry,
                    sample_count,
                    *channel,
                    channel_samples.get(*channel).copied(),
                    sample_at,
                    context.envelope_tree(*channel),
                    config.sampling_mode,
                    config.render_quality,
                    config.style,
                    config.zoom_y,
                    config.start_sample,
                    lane,
                    config.channels[*channel].color,
                    channel_budget,
                    config.max_glow_points_per_channel.max(1),
                    config.envelope_temporal_smoothing,
                    config.envelope_release_ms_per_pixel,
                    config.envelope_frame_delta_ms,
                    scratch,
                );
                if lane_index > 0 {
                    commands.push(SurfaceCommand::Line {
                        start: Point { x: 0, y: lane.top },
                        end: Point {
                            x: geometry.width_i32,
                            y: lane.top,
                        },
                        color: config.style.lane_divider,
                    });
                }
            }
        }
    }

    commands
}

/// Return a deterministic per-channel waveform command budget.
///
/// Budget distribution is index-stable and does not depend on prior channels'
/// emitted command counts, which avoids per-frame quality flicker.
pub(super) fn channel_waveform_budget(
    max_waveform_commands: usize,
    visible_channels: usize,
    channel_index: usize,
) -> usize {
    if visible_channels == 0 {
        return 0;
    }
    let base = max_waveform_commands / visible_channels;
    let remainder = max_waveform_commands % visible_channels;
    base.saturating_add(usize::from(channel_index < remainder))
        .max(1)
}

/// Return the minimum shared sample length across all channel slices.
pub(super) fn min_channel_sample_count(channel_samples: &[&[f32]]) -> usize {
    channel_samples
        .iter()
        .map(|samples| samples.len())
        .min()
        .unwrap_or(0)
}

/// Return visible channel indices in ascending order.
fn visible_channel_indices(
    channel_count: usize,
    channels: &[super::WaveformChannelStyle],
) -> Vec<usize> {
    let max_channels = channel_count.min(channels.len());
    (0..max_channels)
        .filter(|index| channels[*index].visible)
        .collect()
}

/// Push the background fill command for the full waveform surface.
fn push_background(
    commands: &mut Vec<SurfaceCommand>,
    geometry: &WaveformGeometry,
    style: WaveformViewStyle,
) {
    commands.push(SurfaceCommand::FillRect {
        rect: Rect {
            origin: Point { x: 0, y: 0 },
            size: Size {
                width: geometry.width_i32 as u32,
                height: geometry.height_i32 as u32,
            },
        },
        color: style.background,
    });
}

/// Push all configured vertical grid lines for the current geometry.
fn push_vertical_grid(
    commands: &mut Vec<SurfaceCommand>,
    geometry: &WaveformGeometry,
    config: &WaveformViewConfig<'_>,
) {
    for (x, tone) in vertical_grid_lines(config.grid_mode, geometry.width_i32) {
        commands.push(SurfaceCommand::Line {
            start: Point { x, y: 0 },
            end: Point {
                x,
                y: geometry.height_i32,
            },
            color: match tone {
                GridTone::Bar => config.style.grid_bar,
                GridTone::Beat => config.style.grid_beat,
                GridTone::Subdivision => config.style.grid_subdivision,
            },
        });
    }
}

/// Push evenly distributed horizontal grid lines for the current geometry.
fn push_horizontal_grid(
    commands: &mut Vec<SurfaceCommand>,
    geometry: &WaveformGeometry,
    config: &WaveformViewConfig<'_>,
) {
    let horizontal_count = config.horizontal_grid_lines.max(1);
    let mut y_lines = Vec::with_capacity((horizontal_count + 1) as usize);
    for step in 0..=horizontal_count {
        let y = ((step as i64 * geometry.height_i32 as i64 + i64::from(horizontal_count / 2))
            / i64::from(horizontal_count)) as i32;
        let is_center = step == horizontal_count / 2;
        match y_lines.last_mut() {
            Some((last_y, last_is_center)) if *last_y == y => {
                *last_is_center |= is_center;
            }
            _ => y_lines.push((y, is_center)),
        }
    }

    for (y, is_center) in y_lines {
        commands.push(SurfaceCommand::Line {
            start: Point { x: 0, y },
            end: Point {
                x: geometry.width_i32,
                y,
            },
            color: if is_center {
                config.style.grid_horizontal_center
            } else {
                config.style.grid_horizontal
            },
        });
    }
}

/// Estimate command count for one frame to minimize command-vector reallocations.
fn estimate_waveform_command_capacity(
    geometry: &WaveformGeometry,
    sample_count: usize,
    visible_channels: usize,
    config: &WaveformViewConfig<'_>,
) -> usize {
    let width = geometry.width_i32.max(2) as usize;
    let base_grid = 1
        + vertical_grid_lines(config.grid_mode, geometry.width_i32).len()
        + (config.horizontal_grid_lines.max(1) as usize + 1);
    if sample_count < 2 || visible_channels == 0 {
        return base_grid + 8;
    }

    let segments = width.saturating_sub(1);
    let lane_dividers = if matches!(config.display_mode, WaveformDisplayMode::Split) {
        visible_channels.saturating_sub(1)
    } else {
        0
    };
    let layers = config.style.waveform_outline_layers.max(1) as usize;
    let per_channel = match (config.sampling_mode, config.render_quality) {
        (WaveformSamplingMode::Linear, WaveformRenderQuality::LegacyCpuOnly) => segments,
        (WaveformSamplingMode::EnvelopeMinMax, WaveformRenderQuality::LegacyCpuOnly) => width,
        (WaveformSamplingMode::Linear, WaveformRenderQuality::AutoVectorPreferred) => {
            // Core contour polyline + glow above/below polylines.
            1 + (2 * layers)
        }
        (WaveformSamplingMode::EnvelopeMinMax, WaveformRenderQuality::AutoVectorPreferred) => {
            // Body columns + top/bottom core polylines + top/bottom glow polylines.
            width + 2 + (2 * layers)
        }
    };
    let waveform_estimate = per_channel.saturating_mul(visible_channels);
    let budget_cap = config.max_waveform_commands.max(1).saturating_add(8);
    base_grid + lane_dividers + waveform_estimate.min(budget_cap)
}
