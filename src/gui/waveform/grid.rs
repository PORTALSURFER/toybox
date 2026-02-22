//! Grid generation helpers for waveform rendering.

use super::WaveformGridMode;

/// Classifies vertical grid line emphasis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum GridTone {
    /// Bar boundary line.
    Bar,
    /// Beat boundary line.
    Beat,
    /// Beat subdivision line.
    Subdivision,
}

/// Build vertical grid lines for fixed or tempo-locked modes.
pub(super) fn vertical_grid_lines(mode: WaveformGridMode, width: i32) -> Vec<(i32, GridTone)> {
    match mode {
        WaveformGridMode::Fixed { line_count } => fixed_grid_lines(line_count, width),
        WaveformGridMode::TempoLocked {
            beats_visible,
            beats_per_bar,
            subdivisions_per_beat,
            start_beat,
        } => tempo_locked_grid_lines(
            beats_visible,
            beats_per_bar,
            subdivisions_per_beat,
            start_beat,
            width,
        ),
    }
}

/// Build evenly spaced fixed grid lines across the waveform width.
fn fixed_grid_lines(line_count: u32, width: i32) -> Vec<(i32, GridTone)> {
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

/// Build beat-domain grid lines anchored to a visible beat span.
fn tempo_locked_grid_lines(
    beats_visible: f64,
    beats_per_bar: f64,
    subdivisions_per_beat: u32,
    start_beat: f64,
    width: i32,
) -> Vec<(i32, GridTone)> {
    if !beats_visible.is_finite() || beats_visible <= 0.0 || !start_beat.is_finite() {
        return vec![(0, GridTone::Bar), (width, GridTone::Bar)];
    }

    let subdivisions = subdivisions_per_beat.max(1) as f64;
    let step_beats = 1.0 / subdivisions;
    let end_beat = start_beat + beats_visible;
    let first_step = (start_beat / step_beats).floor() as i64 - 1;
    let last_step = (end_beat / step_beats).ceil() as i64 + 1;
    let bar_period = if beats_per_bar.is_finite() && beats_per_bar > 0.0 {
        Some(beats_per_bar)
    } else {
        None
    };
    let mut lines = Vec::new();
    const EPSILON: f64 = 1.0e-6;

    for step in first_step..=last_step {
        let beat = step as f64 * step_beats;
        let x_norm = (beat - start_beat) / beats_visible;
        if !(-EPSILON..=1.0 + EPSILON).contains(&x_norm) {
            continue;
        }
        let x = (x_norm * width as f64).round() as i32;
        let tone = if bar_period.is_some_and(|period| is_multiple(beat, period, EPSILON)) {
            GridTone::Bar
        } else if is_multiple(beat, 1.0, EPSILON) {
            GridTone::Beat
        } else {
            GridTone::Subdivision
        };
        lines.push((x.clamp(0, width), tone));
    }

    if lines.is_empty() {
        return vec![(0, GridTone::Bar), (width, GridTone::Bar)];
    }

    lines.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| tone_priority(right.1).cmp(&tone_priority(left.1)))
    });
    lines.dedup_by(|left, right| left.0 == right.0);
    lines
}

/// Return a stable precedence rank for tone deduplication.
fn tone_priority(tone: GridTone) -> u8 {
    match tone {
        GridTone::Bar => 3,
        GridTone::Beat => 2,
        GridTone::Subdivision => 1,
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
