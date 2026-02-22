//! Mode-specific waveform channel rendering helpers.

use super::sampling::{clamp_sample, for_each_envelope_min_max_column, resample_channel_linear};
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
    sample_at: &SampleAt,
    sampling_mode: WaveformSamplingMode,
    render_quality: WaveformRenderQuality,
    style: WaveformViewStyle,
    zoom_y: f32,
    lane: LaneBounds,
    color: Color,
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
                sample_at,
                lane,
                color,
                center_y,
                scale_y,
            )
        }
        (WaveformSamplingMode::EnvelopeMinMax, WaveformRenderQuality::LegacyCpuOnly) => {
            draw_waveform_channel_envelope_legacy(
                commands,
                geometry,
                sample_count,
                channel,
                sample_at,
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
                sample_at,
                lane,
                color,
                style,
                center_y,
                scale_y,
            )
        }
        (WaveformSamplingMode::EnvelopeMinMax, WaveformRenderQuality::AutoVectorPreferred) => {
            draw_waveform_channel_envelope_styled(
                commands,
                geometry,
                sample_count,
                channel,
                sample_at,
                lane,
                color,
                style,
                center_y,
                scale_y,
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
    sample_at: &SampleAt,
    lane: LaneBounds,
    color: Color,
    center_y: i32,
    scale_y: f32,
) where
    SampleAt: Fn(usize, usize) -> f32,
{
    let samples = linear_samples(
        sample_count,
        channel,
        sample_at,
        geometry.width_i32.max(2) as usize,
    );
    if samples.len() < 2 {
        return;
    }
    let points = samples_to_points(
        &samples,
        geometry.width_i32.max(2) - 1,
        lane,
        center_y,
        scale_y,
    );
    emit_polyline(commands, &points, color);
}

/// Draw one channel using deterministic per-column min/max envelope segments.
#[allow(clippy::too_many_arguments)]
fn draw_waveform_channel_envelope_legacy<SampleAt>(
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
    let x_max = geometry.width_i32.max(2) - 1;
    for_each_envelope_min_max_column(
        sample_count,
        channel,
        columns,
        sample_at,
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
    sample_at: &SampleAt,
    lane: LaneBounds,
    color: Color,
    style: WaveformViewStyle,
    center_y: i32,
    scale_y: f32,
) where
    SampleAt: Fn(usize, usize) -> f32,
{
    let samples = linear_samples(
        sample_count,
        channel,
        sample_at,
        geometry.width_i32.max(2) as usize,
    );
    if samples.len() < 2 {
        return;
    }

    let contour = samples_to_points(
        &samples,
        geometry.width_i32.max(2) - 1,
        lane,
        center_y,
        scale_y,
    );
    emit_glow_symmetric(commands, &contour, lane, color, style);
    emit_polyline(
        commands,
        &contour,
        with_alpha(color, style.waveform_outline_alpha_inner),
    );
}

/// Draw one min/max envelope channel with body fill and gradient-like outlines.
#[allow(clippy::too_many_arguments)]
fn draw_waveform_channel_envelope_styled<SampleAt>(
    commands: &mut Vec<SurfaceCommand>,
    geometry: &WaveformGeometry,
    sample_count: usize,
    channel: usize,
    sample_at: &SampleAt,
    lane: LaneBounds,
    color: Color,
    style: WaveformViewStyle,
    center_y: i32,
    scale_y: f32,
) where
    SampleAt: Fn(usize, usize) -> f32,
{
    let columns = geometry.width_i32.max(2) as usize;
    let x_max = geometry.width_i32.max(2) - 1;
    let body_color = with_alpha(color, style.waveform_body_alpha);
    let mut top_contour = Vec::with_capacity(columns);
    let mut bottom_contour = Vec::with_capacity(columns);

    for_each_envelope_min_max_column(
        sample_count,
        channel,
        columns,
        sample_at,
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
            top_contour.push(Point { x, y: y_top });
            bottom_contour.push(Point { x, y: y_bottom });
        },
    );

    if top_contour.len() < 2 || bottom_contour.len() < 2 {
        return;
    }

    emit_glow_outward(commands, &top_contour, lane, color, style, -1);
    emit_glow_outward(commands, &bottom_contour, lane, color, style, 1);
    let core_color = with_alpha(color, style.waveform_outline_alpha_inner);
    emit_polyline(commands, &top_contour, core_color);
    emit_polyline(commands, &bottom_contour, core_color);
}

/// Build linearly sampled values for one channel.
fn linear_samples<SampleAt>(
    sample_count: usize,
    channel: usize,
    sample_at: &SampleAt,
    points: usize,
) -> Vec<f32>
where
    SampleAt: Fn(usize, usize) -> f32,
{
    resample_channel_linear(sample_count, channel, points, sample_at)
}

/// Convert sampled values into x/y contour points.
fn samples_to_points(
    samples: &[f32],
    x_max: i32,
    lane: LaneBounds,
    center_y: i32,
    scale_y: f32,
) -> Vec<Point> {
    if samples.is_empty() {
        return Vec::new();
    }

    let points = samples.len();
    let mut contour = Vec::with_capacity(points);
    for (point_index, sample) in samples.iter().enumerate() {
        let x = point_x(point_index, points, x_max);
        let y = sample_to_lane_y(*sample, center_y, scale_y, lane);
        contour.push(Point { x, y });
    }
    contour
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

/// Emit a symmetric glow around one contour by layering upward and downward offsets.
fn emit_glow_symmetric(
    commands: &mut Vec<SurfaceCommand>,
    contour: &[Point],
    lane: LaneBounds,
    color: Color,
    style: WaveformViewStyle,
) {
    let layers = style.waveform_outline_layers.max(1);
    for layer in 0..layers {
        let alpha = outline_layer_alpha(style, layer, layers);
        if alpha == 0 {
            continue;
        }
        let layer_color = with_alpha(color, alpha);
        let offset = (layer as i32) + 1;
        let segment_stride = outline_layer_segment_stride(layer);
        emit_shifted_polyline(
            commands,
            contour,
            lane,
            layer_color,
            -offset,
            segment_stride,
        );
        emit_shifted_polyline(commands, contour, lane, layer_color, offset, segment_stride);
    }
}

/// Emit a directional glow for one contour by layering offsets away from waveform body.
fn emit_glow_outward(
    commands: &mut Vec<SurfaceCommand>,
    contour: &[Point],
    lane: LaneBounds,
    color: Color,
    style: WaveformViewStyle,
    direction: i32,
) {
    let layers = style.waveform_outline_layers.max(1);
    for layer in 0..layers {
        let alpha = outline_layer_alpha(style, layer, layers);
        if alpha == 0 {
            continue;
        }
        let layer_color = with_alpha(color, alpha);
        let signed_offset = ((layer as i32) + 1) * direction;
        let segment_stride = outline_layer_segment_stride(layer);
        emit_shifted_polyline(
            commands,
            contour,
            lane,
            layer_color,
            signed_offset,
            segment_stride,
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
) {
    if points.len() < 2 || color.a == 0 {
        return;
    }

    let stride = segment_stride.max(1);
    let mut index = 0usize;
    while index + 1 < points.len() {
        let next_index = (index + stride).min(points.len() - 1);
        let start_point = points[index];
        let end_point = points[next_index];
        let start = Point {
            x: start_point.x,
            y: (start_point.y + offset).clamp(lane.top, lane.bottom),
        };
        let end = Point {
            x: end_point.x,
            y: (end_point.y + offset).clamp(lane.top, lane.bottom),
        };
        if start.x != end.x || start.y != end.y {
            commands.push(SurfaceCommand::Line { start, end, color });
        }
        index = next_index;
    }
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
/// the perceived gradient halo while reducing command count on dense waveforms.
fn outline_layer_segment_stride(layer: u8) -> usize {
    usize::from(layer) + 1
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
