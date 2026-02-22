//! Mode-specific waveform channel rendering helpers.

use super::context::WaveformRenderScratch;
use super::sampling::{
    EnvelopeMinMaxTree, clamp_sample, for_each_envelope_min_max_column,
    for_each_envelope_min_max_column_cached, for_each_envelope_min_max_column_from_slice,
    resample_channel_linear_from_slice_into, resample_channel_linear_into,
};
use super::{
    Color, LaneBounds, Point, SurfaceCommand, WaveformGeometry, WaveformRenderQuality,
    WaveformSamplingMode, WaveformViewStyle,
};

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
    lane: LaneBounds,
    color: Color,
    channel_command_budget: usize,
    glow_point_budget: usize,
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
                channel_command_budget,
                glow_point_budget,
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

/// Draw one channel using deterministic per-column min/max envelope segments.
#[allow(clippy::too_many_arguments)]
fn draw_waveform_channel_envelope_legacy<SampleAt>(
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
) where
    SampleAt: Fn(usize, usize) -> f32,
{
    let columns = geometry.width_i32.max(2) as usize;
    let x_max = geometry.width_i32.max(2) - 1;
    for_each_channel_envelope_column(
        sample_count,
        channel,
        columns,
        channel_samples,
        sample_at,
        envelope_tree,
        |column_index, min_sample, max_sample| {
            let x = point_x(column_index, columns, x_max);
            let y_top = sample_to_lane_y(max_sample, center_y, scale_y, lane);
            let y_bottom = sample_to_lane_y(min_sample, center_y, scale_y, lane);
            commands.push(SurfaceCommand::Line {
                start: Point { x, y: y_top },
                end: Point { x, y: y_bottom },
                color,
            });
        },
    );
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

/// Draw one min/max envelope channel with body fill and gradient-like outlines.
#[allow(clippy::too_many_arguments)]
fn draw_waveform_channel_envelope_styled<SampleAt>(
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
    channel_command_budget: usize,
    glow_point_budget: usize,
    scratch: &mut WaveformRenderScratch,
) where
    SampleAt: Fn(usize, usize) -> f32,
{
    let columns = geometry.width_i32.max(2) as usize;
    let x_max = geometry.width_i32.max(2) - 1;
    let body_color = with_alpha(color, style.waveform_body_alpha);
    scratch.top_contour.clear();
    scratch.bottom_contour.clear();
    scratch.top_contour.reserve(columns);
    scratch.bottom_contour.reserve(columns);

    for_each_channel_envelope_column(
        sample_count,
        channel,
        columns,
        channel_samples,
        sample_at,
        envelope_tree,
        |column_index, min_sample, max_sample| {
            let x = point_x(column_index, columns, x_max);
            let y_top = sample_to_lane_y(max_sample, center_y, scale_y, lane);
            let y_bottom = sample_to_lane_y(min_sample, center_y, scale_y, lane);
            if y_top != y_bottom && body_color.a > 0 {
                commands.push(SurfaceCommand::Line {
                    start: Point { x, y: y_top },
                    end: Point { x, y: y_bottom },
                    color: body_color,
                });
            }
            scratch.top_contour.push(Point { x, y: y_top });
            scratch.bottom_contour.push(Point { x, y: y_bottom });
        },
    );

    if scratch.top_contour.len() < 2 || scratch.bottom_contour.len() < 2 {
        return;
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

/// Fill linear resample output for one channel.
fn linear_samples_into<SampleAt>(
    sample_count: usize,
    channel: usize,
    channel_samples: Option<&[f32]>,
    sample_at: &SampleAt,
    points: usize,
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

/// Iterate min/max envelopes for one channel using the best available source.
fn for_each_channel_envelope_column<SampleAt, Visit>(
    sample_count: usize,
    channel: usize,
    columns: usize,
    channel_samples: Option<&[f32]>,
    sample_at: &SampleAt,
    envelope_tree: Option<&EnvelopeMinMaxTree>,
    visit: Visit,
) where
    SampleAt: Fn(usize, usize) -> f32,
    Visit: FnMut(usize, f32, f32),
{
    if let Some(tree) = envelope_tree {
        for_each_envelope_min_max_column_cached(sample_count, columns, tree, visit);
        return;
    }
    if let Some(samples) = channel_samples {
        let bounded = &samples[..sample_count.min(samples.len())];
        for_each_envelope_min_max_column_from_slice(bounded, columns, visit);
        return;
    }
    for_each_envelope_min_max_column(sample_count, channel, columns, sample_at, visit);
}

/// Convert sampled values into x/y contour points.
fn samples_to_points(
    samples: &[f32],
    x_max: i32,
    lane: LaneBounds,
    center_y: i32,
    scale_y: f32,
    out: &mut Vec<Point>,
) {
    out.clear();
    if samples.is_empty() {
        return;
    }

    let points = samples.len();
    out.reserve(points);
    for (point_index, sample) in samples.iter().enumerate() {
        let x = point_x(point_index, points, x_max);
        let y = sample_to_lane_y(*sample, center_y, scale_y, lane);
        out.push(Point { x, y });
    }
}

/// Emit a polyline as a deterministic sequence of line commands.
fn emit_polyline(commands: &mut Vec<SurfaceCommand>, points: &[Point], color: Color) {
    if points.len() < 2 || color.a == 0 {
        return;
    }

    for segment in points.windows(2) {
        if let [start, end] = segment {
            if start.x == end.x && start.y == end.y {
                continue;
            }
            commands.push(SurfaceCommand::Line {
                start: *start,
                end: *end,
                color,
            });
        }
    }
}

/// Emit one batched polyline command.
fn emit_polyline_batched(
    commands: &mut Vec<SurfaceCommand>,
    points: &[Point],
    color: Color,
    thickness: f32,
) {
    if points.len() < 2 || color.a == 0 {
        return;
    }
    let compact = compact_polyline_points(points);
    if compact.len() < 2 {
        return;
    }
    commands.push(SurfaceCommand::Polyline {
        points: compact,
        thickness: thickness.max(1.0),
        color,
    });
}

/// Deterministic adaptive glow quality settings for one channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GlowPlan {
    /// Number of glow layers to emit.
    layers: u8,
    /// Global stride multiplier applied to all glow layers.
    stride_scale: usize,
}

/// Resolve adaptive glow layers and point stride from command/point budgets.
fn resolve_glow_plan(
    contour_points: usize,
    style: WaveformViewStyle,
    max_glow_commands: usize,
    max_glow_points: usize,
    commands_per_layer: usize,
) -> GlowPlan {
    let configured_layers = usize::from(style.waveform_outline_layers);
    if configured_layers == 0
        || contour_points < 2
        || max_glow_commands == 0
        || max_glow_points == 0
    {
        return GlowPlan {
            layers: 0,
            stride_scale: 1,
        };
    }

    let commands_per_layer = commands_per_layer.max(1);
    let mut layers = configured_layers.min(max_glow_commands / commands_per_layer);
    if layers == 0 {
        return GlowPlan {
            layers: 0,
            stride_scale: 1,
        };
    }

    let mut stride_scale = 1usize;
    loop {
        let estimated_points =
            estimate_glow_points(contour_points, layers, stride_scale, commands_per_layer);
        if estimated_points <= max_glow_points {
            break;
        }
        if layers > 1 {
            layers -= 1;
            stride_scale = 1;
            continue;
        }
        if stride_scale >= contour_points {
            break;
        }
        stride_scale += 1;
    }

    GlowPlan {
        layers: layers as u8,
        stride_scale,
    }
}

/// Estimate emitted glow-polyline points for one contour size and plan.
fn estimate_glow_points(
    contour_points: usize,
    layers: usize,
    stride_scale: usize,
    commands_per_layer: usize,
) -> usize {
    if contour_points < 2 || layers == 0 {
        return 0;
    }
    let stride_scale = stride_scale.max(1);
    let mut total = 0usize;
    for layer in 0..layers {
        let layer_stride = outline_layer_segment_stride(layer as u8).saturating_mul(stride_scale);
        let sampled_points = sampled_polyline_point_count(contour_points, layer_stride);
        total = total.saturating_add(sampled_points.saturating_mul(commands_per_layer));
    }
    total
}

/// Emit a symmetric glow around one contour by layering upward and downward offsets.
fn emit_glow_symmetric(
    commands: &mut Vec<SurfaceCommand>,
    contour: &[Point],
    lane: LaneBounds,
    color: Color,
    style: WaveformViewStyle,
    glow_plan: GlowPlan,
    shifted_scratch: &mut Vec<Point>,
) {
    let layers = glow_plan.layers;
    for layer in 0..layers {
        let alpha = outline_layer_alpha(style, layer, layers);
        if alpha == 0 {
            continue;
        }
        let layer_color = with_alpha(color, alpha);
        let offset = (layer as i32) + 1;
        let segment_stride =
            outline_layer_segment_stride(layer).saturating_mul(glow_plan.stride_scale);
        emit_shifted_polyline(
            commands,
            contour,
            lane,
            layer_color,
            -offset,
            segment_stride,
            shifted_scratch,
        );
        emit_shifted_polyline(
            commands,
            contour,
            lane,
            layer_color,
            offset,
            segment_stride,
            shifted_scratch,
        );
    }
}

/// Emit a directional glow for one contour by layering offsets away from waveform body.
#[allow(clippy::too_many_arguments)]
fn emit_glow_outward(
    commands: &mut Vec<SurfaceCommand>,
    contour: &[Point],
    lane: LaneBounds,
    color: Color,
    style: WaveformViewStyle,
    direction: i32,
    glow_plan: GlowPlan,
    shifted_scratch: &mut Vec<Point>,
) {
    let layers = glow_plan.layers;
    for layer in 0..layers {
        let alpha = outline_layer_alpha(style, layer, layers);
        if alpha == 0 {
            continue;
        }
        let layer_color = with_alpha(color, alpha);
        let signed_offset = ((layer as i32) + 1) * direction;
        let segment_stride =
            outline_layer_segment_stride(layer).saturating_mul(glow_plan.stride_scale);
        emit_shifted_polyline(
            commands,
            contour,
            lane,
            layer_color,
            signed_offset,
            segment_stride,
            shifted_scratch,
        );
    }
}

/// Emit one polyline shifted in Y by `offset` pixels.
fn emit_shifted_polyline(
    commands: &mut Vec<SurfaceCommand>,
    points: &[Point],
    lane: LaneBounds,
    color: Color,
    offset: i32,
    segment_stride: usize,
    shifted_scratch: &mut Vec<Point>,
) {
    if points.len() < 2 || color.a == 0 {
        return;
    }

    let stride = segment_stride.max(1);
    shifted_scratch.clear();
    shifted_scratch.reserve(sampled_polyline_point_count(points.len(), stride));

    let mut index = 0usize;
    loop {
        let source = points[index];
        let transformed = Point {
            x: source.x,
            y: (source.y + offset).clamp(lane.top, lane.bottom),
        };
        if shifted_scratch.last().copied() != Some(transformed) {
            shifted_scratch.push(transformed);
        }
        if index == points.len() - 1 {
            break;
        }
        index = (index + stride).min(points.len() - 1);
    }
    emit_polyline_batched(commands, shifted_scratch, color, 1.0);
}

/// Resolve one alpha value for a glow layer index.
fn outline_layer_alpha(style: WaveformViewStyle, layer: u8, layers: u8) -> u8 {
    let inner = i32::from(style.waveform_outline_alpha_inner);
    let outer = i32::from(style.waveform_outline_alpha_outer);
    if layers <= 1 {
        return style.waveform_outline_alpha_inner;
    }

    let numerator = (outer - inner) * i32::from(layer);
    let denominator = i32::from(layers - 1);
    (inner + numerator / denominator).clamp(0, 255) as u8
}

/// Resolve polyline segment stride for one glow layer.
///
/// Outer glow layers use progressively coarser segment strides. This preserves
/// the perceived gradient halo while reducing path complexity on dense waveforms.
fn outline_layer_segment_stride(layer: u8) -> usize {
    usize::from(layer) + 1
}

/// Return sampled point count for one stride over a polyline.
fn sampled_polyline_point_count(points: usize, stride: usize) -> usize {
    if points <= 1 {
        return points;
    }
    let stride = stride.max(1);
    ((points - 1) / stride) + 1
}

/// Remove consecutive duplicate points while preserving deterministic order.
fn compact_polyline_points(points: &[Point]) -> Vec<Point> {
    let mut compact = Vec::with_capacity(points.len());
    for point in points {
        if compact.last().copied() != Some(*point) {
            compact.push(*point);
        }
    }
    compact
}

/// Return `color` with alpha multiplied by `alpha` in `0..=255`.
fn with_alpha(color: Color, alpha: u8) -> Color {
    let scaled = (u16::from(color.a) * u16::from(alpha) + 127) / 255;
    Color::rgba(color.r, color.g, color.b, scaled as u8)
}

/// Map one point index to a rounded pixel-space x coordinate.
fn point_x(point_index: usize, points: usize, x_max: i32) -> i32 {
    if points <= 1 || x_max <= 0 {
        return 0;
    }
    if points == (x_max + 1) as usize {
        return point_index as i32;
    }
    let denominator = (points - 1) as i64;
    let numerator = point_index as i64 * x_max as i64;
    ((numerator + denominator / 2) / denominator) as i32
}

/// Convert one normalized sample value into lane Y coordinates.
fn sample_to_lane_y(sample: f32, center_y: i32, scale_y: f32, lane: LaneBounds) -> i32 {
    ((center_y as f32 - clamp_sample(sample) * scale_y).round() as i32).clamp(lane.top, lane.bottom)
}
