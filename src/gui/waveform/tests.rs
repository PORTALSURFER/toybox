use super::*;

mod context_and_smoothing;
mod grid_and_sampling;
mod sampling_modes;
mod styled_and_budget;

pub(super) fn collect_lines_by_color(
    commands: &[SurfaceCommand],
    color: Color,
) -> Vec<(Point, Point)> {
    let mut lines = Vec::new();
    for command in commands {
        match command {
            SurfaceCommand::Line {
                start,
                end,
                color: command_color,
            } if *command_color == color => lines.push((*start, *end)),
            SurfaceCommand::Polyline {
                points,
                color: command_color,
                ..
            } if *command_color == color => {
                for segment in points.windows(2) {
                    if let [start, end] = segment {
                        lines.push((*start, *end));
                    }
                }
            }
            _ => {}
        }
    }
    lines
}

pub(super) fn collect_lines_by_rgb(
    commands: &[SurfaceCommand],
    color: Color,
) -> Vec<(Point, Point, Color)> {
    let mut lines = Vec::new();
    for command in commands {
        match command {
            SurfaceCommand::Line {
                start,
                end,
                color: command_color,
            } if command_color.r == color.r
                && command_color.g == color.g
                && command_color.b == color.b =>
            {
                lines.push((*start, *end, *command_color));
            }
            SurfaceCommand::Polyline {
                points,
                color: command_color,
                ..
            } if command_color.r == color.r
                && command_color.g == color.g
                && command_color.b == color.b =>
            {
                for segment in points.windows(2) {
                    if let [start, end] = segment {
                        lines.push((*start, *end, *command_color));
                    }
                }
            }
            _ => {}
        }
    }
    lines
}

pub(super) fn count_polyline_commands_by_rgb(commands: &[SurfaceCommand], color: Color) -> usize {
    commands
        .iter()
        .filter(|command| match command {
            SurfaceCommand::Polyline {
                color: command_color,
                ..
            } => {
                command_color.r == color.r
                    && command_color.g == color.g
                    && command_color.b == color.b
            }
            _ => false,
        })
        .count()
}

pub(super) fn collect_fill_rects_by_rgb(
    commands: &[SurfaceCommand],
    color: Color,
) -> Vec<(Rect, Color)> {
    commands
        .iter()
        .filter_map(|command| match command {
            SurfaceCommand::FillRect {
                rect,
                color: command_color,
            } if command_color.r == color.r
                && command_color.g == color.g
                && command_color.b == color.b =>
            {
                Some((*rect, *command_color))
            }
            _ => None,
        })
        .collect()
}

pub(super) fn assert_grid_invariants(lines: &[(i32, GridTone)], width: i32) {
    let mut previous_x = i32::MIN;
    for (x, _) in lines {
        assert!(*x >= 0 && *x <= width, "x={x} outside 0..={width}");
        assert!(
            *x > previous_x,
            "grid x positions must be strictly increasing"
        );
        previous_x = *x;
    }
}

pub(super) fn has_beat_aligned_line(lines: &[(i32, GridTone)], x: i32) -> bool {
    lines
        .iter()
        .any(|(line_x, tone)| *line_x == x && matches!(tone, GridTone::Bar | GridTone::Beat))
}
