//! Reusable waveform view renderer for declarative surface widgets.
//!
//! The API builds `SurfaceCommand` vectors from sample accessors so plugins can
//! render high-frequency waveform views without duplicating draw logic.

use super::declarative::SurfaceCommand;
use super::{Color, Point, Rect, Size};

/// Channel arrangement mode for waveform rendering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaveformDisplayMode {
    /// Draw all visible channels in a single shared lane.
    Overlay,
    /// Draw each visible channel in its own vertical lane.
    Split,
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
    },
}

/// Visual style for one waveform channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WaveformChannelStyle {
    /// Whether the channel is visible.
    pub visible: bool,
    /// Stroke color used for the channel polyline.
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
}

impl Default for WaveformViewStyle {
    fn default() -> Self {
        Self {
            background: Color::rgb(14, 17, 20),
            grid_bar: Color::rgb(44, 50, 57),
            grid_beat: Color::rgb(36, 41, 47),
            grid_subdivision: Color::rgb(30, 35, 41),
            grid_horizontal: Color::rgb(27, 31, 37),
            grid_horizontal_center: Color::rgb(53, 61, 69),
            lane_divider: Color::rgb(42, 48, 54),
        }
    }
}

/// Full configuration for waveform surface command generation.
#[derive(Clone, Debug, PartialEq)]
pub struct WaveformViewConfig<'a> {
    /// Channel rendering mode.
    pub display_mode: WaveformDisplayMode,
    /// Vertical zoom multiplier applied to sample amplitude.
    pub zoom_y: f32,
    /// Per-channel visibility and color styles.
    pub channels: &'a [WaveformChannelStyle],
    /// Vertical grid behavior.
    pub grid_mode: WaveformGridMode,
    /// Number of horizontal divisions including top and bottom lines.
    pub horizontal_grid_lines: u32,
    /// Global waveform surface style.
    pub style: WaveformViewStyle,
}

impl<'a> WaveformViewConfig<'a> {
    /// Build a default-configured view with explicit channel styles.
    pub fn new(channels: &'a [WaveformChannelStyle]) -> Self {
        Self {
            display_mode: WaveformDisplayMode::Overlay,
            zoom_y: 1.0,
            channels,
            grid_mode: WaveformGridMode::Fixed { line_count: 8 },
            horizontal_grid_lines: 8,
            style: WaveformViewStyle::default(),
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
    let geometry = WaveformGeometry::new(width, height);
    let mut commands = Vec::with_capacity(2048);

    commands.push(SurfaceCommand::FillRect {
        rect: Rect {
            origin: Point { x: 0, y: 0 },
            size: Size { width, height },
        },
        color: config.style.background,
    });

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

    let horizontal_count = config.horizontal_grid_lines.max(1);
    for step in 0..=horizontal_count {
        let y =
            ((step as f32 / horizontal_count as f32) * geometry.height_i32 as f32).round() as i32;
        commands.push(SurfaceCommand::Line {
            start: Point { x: 0, y },
            end: Point {
                x: geometry.width_i32,
                y,
            },
            color: if step == horizontal_count / 2 {
                config.style.grid_horizontal_center
            } else {
                config.style.grid_horizontal
            },
        });
    }

    if sample_count < 2 {
        return commands;
    }

    let max_channels = channel_count.min(config.channels.len());
    let visible_channels: Vec<usize> = (0..max_channels)
        .filter(|index| config.channels[*index].visible)
        .collect();
    if visible_channels.is_empty() {
        return commands;
    }

    match config.display_mode {
        WaveformDisplayMode::Overlay => {
            for channel in visible_channels {
                draw_waveform_channel(
                    &mut commands,
                    &geometry,
                    sample_count,
                    channel,
                    &sample_at,
                    config.zoom_y,
                    LaneBounds::full(&geometry),
                    config.channels[channel].color,
                );
            }
        }
        WaveformDisplayMode::Split => {
            let lane_count = visible_channels.len().max(1) as i32;
            for (lane_index, channel) in visible_channels.iter().enumerate() {
                let lane = LaneBounds::for_split_lane(&geometry, lane_index as i32, lane_count);
                draw_waveform_channel(
                    &mut commands,
                    &geometry,
                    sample_count,
                    *channel,
                    &sample_at,
                    config.zoom_y,
                    lane,
                    config.channels[*channel].color,
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

/// Classifies grid line emphasis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GridTone {
    /// Bar boundary line.
    Bar,
    /// Beat boundary line.
    Beat,
    /// Beat subdivision line.
    Subdivision,
}

/// Pixel-space geometry for one waveform surface.
#[derive(Clone, Copy, Debug)]
struct WaveformGeometry {
    /// Surface width in pixels.
    width_i32: i32,
    /// Surface height in pixels.
    height_i32: i32,
}

impl WaveformGeometry {
    /// Build geometry from unsigned surface dimensions.
    fn new(width: u32, height: u32) -> Self {
        Self {
            width_i32: width.max(1) as i32,
            height_i32: height.max(1) as i32,
        }
    }
}

/// Vertical lane bounds for one rendered channel.
#[derive(Clone, Copy, Debug)]
struct LaneBounds {
    /// Inclusive lane top pixel.
    top: i32,
    /// Inclusive lane bottom pixel.
    bottom: i32,
}

impl LaneBounds {
    /// Build bounds spanning the full surface.
    fn full(geometry: &WaveformGeometry) -> Self {
        Self {
            top: 0,
            bottom: geometry.height_i32,
        }
    }

    /// Build bounds for one split lane.
    fn for_split_lane(geometry: &WaveformGeometry, lane_index: i32, lane_count: i32) -> Self {
        let top = lane_index * geometry.height_i32 / lane_count.max(1);
        let bottom = ((lane_index + 1) * geometry.height_i32 / lane_count.max(1))
            .clamp(top + 1, geometry.height_i32);
        Self { top, bottom }
    }
}

/// Draw one waveform channel as a stable polyline.
#[allow(clippy::too_many_arguments)]
fn draw_waveform_channel<SampleAt>(
    commands: &mut Vec<SurfaceCommand>,
    geometry: &WaveformGeometry,
    sample_count: usize,
    channel: usize,
    sample_at: &SampleAt,
    zoom_y: f32,
    lane: LaneBounds,
    color: Color,
) where
    SampleAt: Fn(usize, usize) -> f32,
{
    let lane_height = (lane.bottom - lane.top).max(1);
    let center_y = lane.top + lane_height / 2;
    let scale_y = (lane_height as f32 * 0.45) / zoom_y.max(0.05);
    let points = geometry.width_i32.max(2) as usize;
    let samples = resample_channel_linear(sample_count, channel, points, sample_at);
    if samples.len() < 2 {
        return;
    }
    let x_max = geometry.width_i32.max(2) - 1;

    let mut prev = None;
    for (point_index, sample) in samples.iter().enumerate() {
        let x = ((point_index as f32 / (points - 1) as f32) * x_max as f32).round() as i32;
        let y = sample_to_lane_y(*sample, center_y, scale_y, lane.top, lane.bottom);
        let current = Point {
            x: x.clamp(0, x_max),
            y,
        };
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

/// Convert one normalized sample value into lane Y coordinates.
fn sample_to_lane_y(
    sample: f32,
    center_y: i32,
    scale_y: f32,
    lane_top: i32,
    lane_bottom: i32,
) -> i32 {
    ((center_y as f32 - sample.clamp(-1.2, 1.2) * scale_y).round() as i32)
        .clamp(lane_top, lane_bottom)
}

/// Build vertical grid lines for fixed or tempo-locked modes.
fn vertical_grid_lines(mode: WaveformGridMode, width: i32) -> Vec<(i32, GridTone)> {
    match mode {
        WaveformGridMode::Fixed { line_count } => {
            let line_count = line_count.max(1);
            (0..=line_count)
                .map(|step| {
                    let x = ((step as f32 / line_count as f32) * width as f32).round() as i32;
                    let tone = if step % 4 == 0 {
                        GridTone::Bar
                    } else {
                        GridTone::Subdivision
                    };
                    (x.clamp(0, width), tone)
                })
                .collect()
        }
        WaveformGridMode::TempoLocked {
            beats_visible,
            beats_per_bar,
            subdivisions_per_beat,
        } => {
            if !beats_visible.is_finite() || beats_visible <= 0.0 {
                return vec![(0, GridTone::Bar), (width, GridTone::Bar)];
            }
            let subdivisions = subdivisions_per_beat.max(1) as f64;
            let step_beats = 1.0 / subdivisions;
            let last_step = (beats_visible / step_beats).ceil() as i64 + 1;
            let mut lines = Vec::new();
            for step in 0..=last_step {
                let beat = step as f64 * step_beats;
                if beat < 0.0 || beat > beats_visible {
                    continue;
                }
                let x_norm = (beat / beats_visible).clamp(0.0, 1.0);
                let x = (x_norm * width as f64).round() as i32;
                let tone = if is_multiple(beat, beats_per_bar.max(1.0), 1.0e-6) {
                    GridTone::Bar
                } else if is_multiple(beat, 1.0, 1.0e-6) {
                    GridTone::Beat
                } else {
                    GridTone::Subdivision
                };
                lines.push((x.clamp(0, width), tone));
            }
            if lines.is_empty() {
                return vec![(0, GridTone::Bar), (width, GridTone::Bar)];
            }
            lines.sort_by_key(|(x, _)| *x);
            lines.dedup_by(|left, right| left.0 == right.0);
            lines
        }
    }
}

/// Return true when `value` lies on one periodic boundary.
fn is_multiple(value: f64, period: f64, epsilon: f64) -> bool {
    if !value.is_finite() || !period.is_finite() || period <= 0.0 {
        return false;
    }
    let wrapped = value.rem_euclid(period);
    wrapped <= epsilon || (period - wrapped) <= epsilon
}

/// Resample one channel across equally spaced output points.
fn resample_channel_linear<SampleAt>(
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
        return vec![sample_at(channel, 0).clamp(-1.2, 1.2); points];
    }
    if points == 1 {
        return vec![sample_at(channel, 0).clamp(-1.2, 1.2)];
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
        values.push((a + (b - a) * frac).clamp(-1.2, 1.2));
    }
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tempo_locked_grid_lines_are_phase_stable() {
        let one_bar = WaveformGridMode::TempoLocked {
            beats_visible: 4.0,
            beats_per_bar: 4.0,
            subdivisions_per_beat: 2,
        };
        let a = vertical_grid_lines(one_bar, 320);
        let b = vertical_grid_lines(one_bar, 320);
        assert_eq!(a, b);
    }

    #[test]
    fn linear_resample_interpolates_between_endpoints() {
        let samples = [0.0f32, 1.0];
        let resampled = resample_channel_linear(samples.len(), 0, 5, &|_, index| samples[index]);
        assert_eq!(resampled, vec![0.0, 0.25, 0.5, 0.75, 1.0]);
    }
}
