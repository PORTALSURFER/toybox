//! Outline glow planning and emission utilities.

use super::super::{Color, LaneBounds, Point, SurfaceCommand, WaveformViewStyle};
use super::contour::{emit_polyline_batched, sampled_polyline_point_count};

/// Deterministic adaptive glow quality settings for one channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct GlowPlan {
    /// Number of glow layers to emit.
    pub(super) layers: u8,
    /// Global stride multiplier applied to all glow layers.
    pub(super) stride_scale: usize,
}

/// Resolve adaptive glow layers and point stride from command/point budgets.
pub(super) fn resolve_glow_plan(
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

/// Emit a symmetric glow around one contour by layering upward and downward offsets.
pub(super) fn emit_glow_symmetric(
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
pub(super) fn emit_glow_outward(
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

/// Return `color` with alpha multiplied by `alpha` in `0..=255`.
pub(super) fn with_alpha(color: Color, alpha: u8) -> Color {
    let scaled = (u16::from(color.a) * u16::from(alpha) + 127) / 255;
    Color::rgba(color.r, color.g, color.b, scaled as u8)
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
