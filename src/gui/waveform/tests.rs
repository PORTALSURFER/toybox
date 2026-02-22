use super::*;
use std::cell::Cell;

fn collect_lines_by_color(commands: &[SurfaceCommand], color: Color) -> Vec<(Point, Point)> {
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

fn collect_lines_by_rgb(commands: &[SurfaceCommand], color: Color) -> Vec<(Point, Point, Color)> {
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

fn count_polyline_commands_by_rgb(commands: &[SurfaceCommand], color: Color) -> usize {
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

fn collect_fill_rects_by_rgb(commands: &[SurfaceCommand], color: Color) -> Vec<(Rect, Color)> {
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

fn assert_grid_invariants(lines: &[(i32, GridTone)], width: i32) {
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

fn has_beat_aligned_line(lines: &[(i32, GridTone)], x: i32) -> bool {
    lines
        .iter()
        .any(|(line_x, tone)| *line_x == x && matches!(tone, GridTone::Bar | GridTone::Beat))
}

#[test]
fn tempo_locked_grid_lines_are_phase_stable() {
    let one_bar = WaveformGridMode::TempoLocked {
        beats_visible: 4.0,
        beats_per_bar: 4.0,
        subdivisions_per_beat: 2,
        start_beat: 12.25,
    };
    let a = grid::vertical_grid_lines(one_bar, 320);
    let b = grid::vertical_grid_lines(one_bar, 320);
    assert_eq!(a, b);
}

#[test]
fn tempo_locked_grid_one_beat_shift_moves_absolute_beat_line_by_expected_pixels() {
    let width = 320i32;
    let beats_visible = 4.0f64;
    let base_start = 15.25f64;
    let shifted_start = base_start + 1.0;
    let tracked_beat = 17.0f64;
    let expected_px_per_beat = (width as f64 / beats_visible).round() as i32;

    let base = grid::vertical_grid_lines(
        WaveformGridMode::TempoLocked {
            beats_visible,
            beats_per_bar: 4.0,
            subdivisions_per_beat: 2,
            start_beat: base_start,
        },
        width,
    );
    let shifted = grid::vertical_grid_lines(
        WaveformGridMode::TempoLocked {
            beats_visible,
            beats_per_bar: 4.0,
            subdivisions_per_beat: 2,
            start_beat: shifted_start,
        },
        width,
    );

    let base_x = (((tracked_beat - base_start) / beats_visible) * width as f64).round() as i32;
    let shifted_x =
        (((tracked_beat - shifted_start) / beats_visible) * width as f64).round() as i32;
    assert!(
        has_beat_aligned_line(&base, base_x),
        "expected beat-aligned line at x={base_x} for base grid"
    );
    assert!(
        has_beat_aligned_line(&shifted, shifted_x),
        "expected beat-aligned line at x={shifted_x} for shifted grid"
    );
    assert_eq!(
        base_x - shifted_x,
        expected_px_per_beat,
        "one-beat phase shift should move absolute beat line by one-beat pixel span"
    );
}

#[test]
fn tempo_locked_grid_bar_shift_preserves_pattern_and_classification() {
    let base = WaveformGridMode::TempoLocked {
        beats_visible: 8.0,
        beats_per_bar: 4.0,
        subdivisions_per_beat: 4,
        start_beat: 2.375,
    };
    let shifted = WaveformGridMode::TempoLocked {
        beats_visible: 8.0,
        beats_per_bar: 4.0,
        subdivisions_per_beat: 4,
        start_beat: 2.375 + 4.0,
    };
    let a = grid::vertical_grid_lines(base, 320);
    let b = grid::vertical_grid_lines(shifted, 320);
    assert_eq!(a, b);
}

#[test]
fn tempo_locked_grid_renders_expected_bar_cadence_for_common_signatures() {
    let scenarios = [
        (3.0, 6.0, 420, 210),
        (4.0, 8.0, 400, 200),
        (3.5, 7.0, 560, 280),
    ];
    for (beats_per_bar, beats_visible, width, expected_spacing) in scenarios {
        let lines = grid::vertical_grid_lines(
            WaveformGridMode::TempoLocked {
                beats_visible,
                beats_per_bar,
                subdivisions_per_beat: 2,
                start_beat: 0.0,
            },
            width,
        );
        let bar_xs: Vec<i32> = lines
            .iter()
            .filter_map(|(x, tone)| (*tone == GridTone::Bar).then_some(*x))
            .collect();
        assert!(
            bar_xs.len() >= 3,
            "expected at least 3 bar lines for beats_per_bar={beats_per_bar}"
        );
        for pair in bar_xs.windows(2) {
            if let [left, right] = pair {
                assert_eq!(
                    right - left,
                    expected_spacing,
                    "unexpected bar spacing for beats_per_bar={beats_per_bar}"
                );
            }
        }
    }
}

#[test]
fn tempo_locked_grid_loop_wrap_realigns_phase_without_misclassification() {
    let width = 384;
    let beats_visible = 4.0;
    let beats_per_bar = 4.0;
    let subdivisions_per_beat = 4;
    let loop_length_beats = 4.0;

    let before_wrap = grid::vertical_grid_lines(
        WaveformGridMode::TempoLocked {
            beats_visible,
            beats_per_bar,
            subdivisions_per_beat,
            start_beat: 126.75,
        },
        width,
    );
    let after_wrap = grid::vertical_grid_lines(
        WaveformGridMode::TempoLocked {
            beats_visible,
            beats_per_bar,
            subdivisions_per_beat,
            start_beat: 126.75 - loop_length_beats,
        },
        width,
    );
    assert_eq!(before_wrap, after_wrap);
}

#[test]
fn tempo_locked_grid_handles_negative_and_large_start_beats() {
    let starts = [-12345.375, 1_000_000_000_000.375];
    for start_beat in starts {
        let mode = WaveformGridMode::TempoLocked {
            beats_visible: 4.0,
            beats_per_bar: 4.0,
            subdivisions_per_beat: 4,
            start_beat,
        };
        let a = grid::vertical_grid_lines(mode, 384);
        let b = grid::vertical_grid_lines(mode, 384);
        assert_eq!(a, b);
        assert_grid_invariants(&a, 384);
    }
}

#[test]
fn tempo_locked_grid_output_is_sorted_deduped_and_clamped() {
    let lines = grid::vertical_grid_lines(
        WaveformGridMode::TempoLocked {
            beats_visible: 3.7,
            beats_per_bar: 4.0,
            subdivisions_per_beat: 64,
            start_beat: -19.8125,
        },
        97,
    );
    assert_grid_invariants(&lines, 97);
}

#[test]
fn tempo_locked_grid_invalid_bar_period_emits_no_bar_lines() {
    let lines = grid::vertical_grid_lines(
        WaveformGridMode::TempoLocked {
            beats_visible: 4.0,
            beats_per_bar: 0.0,
            subdivisions_per_beat: 2,
            start_beat: 0.0,
        },
        320,
    );
    assert!(
        lines.iter().all(|(_, tone)| *tone != GridTone::Bar),
        "invalid beats_per_bar should not emit bar lines"
    );
    assert!(
        lines.iter().any(|(_, tone)| *tone == GridTone::Beat),
        "expected beat lines when bar period is invalid"
    );
}

#[test]
fn linear_resample_interpolates_between_endpoints() {
    let samples = [0.0f32, 1.0];
    let mut resampled = Vec::new();
    sampling::resample_channel_linear_into(
        samples.len(),
        0,
        5,
        &|_, index| samples[index],
        &mut resampled,
    );
    assert_eq!(resampled, vec![0.0, 0.25, 0.5, 0.75, 1.0]);
}

#[test]
fn envelope_mode_is_deterministic_for_identical_inputs() {
    let styles = [WaveformChannelStyle {
        visible: true,
        color: Color::rgb(240, 80, 70),
    }];
    let mut config = WaveformViewConfig::new(&styles);
    config.sampling_mode = WaveformSamplingMode::EnvelopeMinMax;

    let sample_count = 4096;
    let samples: Vec<f32> = (0..sample_count)
        .map(|index| ((index as f32 * 0.073).sin() * 0.9).clamp(-1.0, 1.0))
        .collect();

    let a = build_waveform_surface_commands(256, 96, sample_count, 1, |_, i| samples[i], &config);
    let b = build_waveform_surface_commands(256, 96, sample_count, 1, |_, i| samples[i], &config);

    assert_eq!(a, b);
}

#[test]
fn envelope_mode_start_sample_phase_is_deterministic_and_affects_column_assignment() {
    let styles = [WaveformChannelStyle {
        visible: true,
        color: Color::rgb(220, 120, 90),
    }];
    let mut config = WaveformViewConfig::new(&styles);
    config.sampling_mode = WaveformSamplingMode::EnvelopeMinMax;
    config.render_quality = WaveformRenderQuality::LegacyCpuOnly;

    let sample_count = 257usize;
    let width = 37u32;
    // Choose an index near a phase-sensitive column boundary so `start_sample`
    // shifts envelope bin assignment.
    let impulse_index = 131usize;
    let mut samples = vec![0.0f32; sample_count];
    samples[impulse_index] = 1.0;

    config.start_sample = 0;
    let a0 =
        build_waveform_surface_commands(width, 72, sample_count, 1, |_, i| samples[i], &config);
    let b0 =
        build_waveform_surface_commands(width, 72, sample_count, 1, |_, i| samples[i], &config);
    assert_eq!(a0, b0, "start_sample=0 should be deterministic");

    config.start_sample = 1;
    let a1 =
        build_waveform_surface_commands(width, 72, sample_count, 1, |_, i| samples[i], &config);
    let b1 =
        build_waveform_surface_commands(width, 72, sample_count, 1, |_, i| samples[i], &config);
    assert_eq!(a1, b1, "start_sample=1 should be deterministic");
    assert_ne!(
        a0, a1,
        "phase-aligned start_sample should affect envelope column assignment"
    );
}

#[test]
fn envelope_mode_phase_repeats_when_start_sample_shifts_by_columns() {
    let styles = [WaveformChannelStyle {
        visible: true,
        color: Color::rgb(210, 150, 100),
    }];
    let mut config = WaveformViewConfig::new(&styles);
    config.sampling_mode = WaveformSamplingMode::EnvelopeMinMax;
    config.render_quality = WaveformRenderQuality::LegacyCpuOnly;

    let sample_count = 409usize;
    let width = 53u32;
    let columns = width.max(2) as u64;
    let samples: Vec<f32> = (0..sample_count)
        .map(|index| ((index as f32 * 0.037).sin() * 0.95).clamp(-1.0, 1.0))
        .collect();

    config.start_sample = 7;
    let a = build_waveform_surface_commands(width, 80, sample_count, 1, |_, i| samples[i], &config);
    config.start_sample = 7 + columns;
    let b = build_waveform_surface_commands(width, 80, sample_count, 1, |_, i| samples[i], &config);

    assert_eq!(
        a, b,
        "phase-aligned envelope binning should repeat every output-column shift"
    );
}

#[test]
fn envelope_mode_preserves_sharp_transients() {
    let styles = [WaveformChannelStyle {
        visible: true,
        color: Color::rgb(255, 12, 12),
    }];
    let mut config = WaveformViewConfig::new(&styles);
    config.sampling_mode = WaveformSamplingMode::EnvelopeMinMax;

    let sample_count = 256usize;
    let width = 32u32;
    let height = 64u32;
    let transient_index = 111usize;

    let mut samples = vec![0.0f32; sample_count];
    samples[transient_index] = 1.0;

    let commands =
        build_waveform_surface_commands(width, height, sample_count, 1, |_, i| samples[i], &config);
    let waveform_lines = collect_lines_by_rgb(&commands, styles[0].color);
    let columns = width.max(2) as usize;
    let transient_column = (transient_index * columns) / sample_count;
    let x_max = width.max(2) as i32 - 1;
    let expected_x =
        ((transient_column as f32 / (columns - 1) as f32) * x_max as f32).round() as i32;

    let mut column_top: Option<i32> = None;
    for (start, end, _) in waveform_lines {
        if start.x == expected_x && end.x == expected_x {
            let top_y = start.y.min(end.y);
            column_top = Some(column_top.map_or(top_y, |existing| existing.min(top_y)));
        }
    }

    assert!(
        column_top.is_some(),
        "expected envelope line at x={expected_x}"
    );
    assert!(
        column_top.unwrap_or(i32::MAX) <= 6,
        "expected transient to reach near lane top"
    );
}

#[test]
fn linear_mode_retains_polyline_invariants() {
    let styles = [WaveformChannelStyle {
        visible: true,
        color: Color::rgb(23, 201, 132),
    }];
    let mut config = WaveformViewConfig::new(&styles);
    config.sampling_mode = WaveformSamplingMode::Linear;
    config.render_quality = WaveformRenderQuality::LegacyCpuOnly;

    let sample_count = 64usize;
    let width = 24u32;
    let samples: Vec<f32> = (0..sample_count)
        .map(|index| ((index as f32 / sample_count as f32) * 2.0 - 1.0).clamp(-1.0, 1.0))
        .collect();

    let commands =
        build_waveform_surface_commands(width, 40, sample_count, 1, |_, i| samples[i], &config);
    let waveform_lines = collect_lines_by_color(&commands, styles[0].color);

    assert_eq!(waveform_lines.len(), width.max(2) as usize - 1);
    assert_eq!(waveform_lines.first().map(|(start, _)| start.x), Some(0));
    assert_eq!(
        waveform_lines.last().map(|(_, end)| end.x),
        Some(width.max(2) as i32 - 1)
    );

    let mut previous_x = i32::MIN;
    for (start, end) in waveform_lines {
        assert!(start.x >= previous_x);
        assert!(end.x >= start.x);
        previous_x = start.x;
    }
}

#[test]
fn envelope_mode_renders_in_overlay_and_split_layouts() {
    let styles = [
        WaveformChannelStyle {
            visible: true,
            color: Color::rgb(230, 40, 40),
        },
        WaveformChannelStyle {
            visible: true,
            color: Color::rgb(40, 120, 250),
        },
    ];

    let left: Vec<f32> = (0..200)
        .map(|index| if index % 13 == 0 { 1.0 } else { 0.2 })
        .collect();
    let right: Vec<f32> = (0..200)
        .map(|index| if index % 17 == 0 { -1.0 } else { -0.2 })
        .collect();

    let mut overlay_config = WaveformViewConfig::new(&styles);
    overlay_config.display_mode = WaveformDisplayMode::Overlay;
    overlay_config.sampling_mode = WaveformSamplingMode::EnvelopeMinMax;
    overlay_config.render_quality = WaveformRenderQuality::LegacyCpuOnly;

    let width = 20u32;
    let height = 100u32;

    let overlay_commands = build_waveform_surface_commands(
        width,
        height,
        left.len(),
        2,
        |channel, index| {
            if channel == 0 {
                left[index]
            } else {
                right[index]
            }
        },
        &overlay_config,
    );

    let left_overlay_lines = collect_lines_by_color(&overlay_commands, styles[0].color);
    let right_overlay_lines = collect_lines_by_color(&overlay_commands, styles[1].color);
    let left_overlay_rects = collect_fill_rects_by_rgb(&overlay_commands, styles[0].color);
    let right_overlay_rects = collect_fill_rects_by_rgb(&overlay_commands, styles[1].color);
    assert_eq!(
        left_overlay_lines.len() + left_overlay_rects.len(),
        width as usize
    );
    assert_eq!(
        right_overlay_lines.len() + right_overlay_rects.len(),
        width as usize
    );

    let mut split_config = overlay_config.clone();
    split_config.display_mode = WaveformDisplayMode::Split;

    let split_commands = build_waveform_surface_commands(
        width,
        height,
        left.len(),
        2,
        |channel, index| {
            if channel == 0 {
                left[index]
            } else {
                right[index]
            }
        },
        &split_config,
    );

    let left_split_lines = collect_lines_by_color(&split_commands, styles[0].color);
    let right_split_lines = collect_lines_by_color(&split_commands, styles[1].color);
    let left_split_rects = collect_fill_rects_by_rgb(&split_commands, styles[0].color);
    let right_split_rects = collect_fill_rects_by_rgb(&split_commands, styles[1].color);
    assert_eq!(
        left_split_lines.len() + left_split_rects.len(),
        width as usize
    );
    assert_eq!(
        right_split_lines.len() + right_split_rects.len(),
        width as usize
    );

    for (start, end) in left_split_lines {
        assert!((0..=50).contains(&start.y));
        assert!((0..=50).contains(&end.y));
    }
    for (rect, _) in left_split_rects {
        assert!((0..=50).contains(&rect.origin.y));
    }
    for (start, end) in right_split_lines {
        assert!((50..=100).contains(&start.y));
        assert!((50..=100).contains(&end.y));
    }
    for (rect, _) in right_split_rects {
        assert!((50..=100).contains(&rect.origin.y));
    }

    let lane_divider_present = split_commands.iter().any(|command| {
        matches!(
            command,
            SurfaceCommand::Line {
                start: Point { x: 0, y: 50 },
                end: Point { x: 20, y: 50 },
                color,
            } if *color == split_config.style.lane_divider
        )
    });
    assert!(lane_divider_present, "expected split lane divider at y=50");
}

#[test]
fn styled_envelope_mode_emits_body_and_glow_alpha_layers() {
    let styles = [WaveformChannelStyle {
        visible: true,
        color: Color::rgb(180, 220, 250),
    }];
    let mut config = WaveformViewConfig::new(&styles);
    config.sampling_mode = WaveformSamplingMode::EnvelopeMinMax;
    config.render_quality = WaveformRenderQuality::AutoVectorPreferred;

    let samples: Vec<f32> = (0..512)
        .map(|index| ((index as f32 * 0.041).sin() * 0.8).clamp(-1.0, 1.0))
        .collect();
    let commands =
        build_waveform_surface_commands(96, 48, samples.len(), 1, |_, i| samples[i], &config);
    let waveform_lines = collect_lines_by_rgb(&commands, styles[0].color);
    assert!(
        !waveform_lines.is_empty(),
        "styled envelope mode should emit waveform lines"
    );

    let mut saw_body_alpha = false;
    let mut saw_outline_inner_alpha = false;
    let mut saw_faded_alpha = false;
    for (_, _, color) in waveform_lines {
        if color.a == config.style.waveform_body_alpha {
            saw_body_alpha = true;
        }
        if color.a == config.style.waveform_outline_alpha_inner {
            saw_outline_inner_alpha = true;
        }
        if color.a > 0 && color.a < config.style.waveform_outline_alpha_inner {
            saw_faded_alpha = true;
        }
    }

    assert!(saw_body_alpha, "expected envelope body alpha layer");
    assert!(
        saw_outline_inner_alpha,
        "expected inner outline alpha layer"
    );
    assert!(saw_faded_alpha, "expected at least one faded glow layer");
}

#[test]
fn styled_envelope_flat_signal_emits_single_pixel_body_marks() {
    let styles = [WaveformChannelStyle {
        visible: true,
        color: Color::rgb(180, 220, 250),
    }];
    let mut config = WaveformViewConfig::new(&styles);
    config.sampling_mode = WaveformSamplingMode::EnvelopeMinMax;
    config.render_quality = WaveformRenderQuality::AutoVectorPreferred;

    let samples = vec![0.0f32; 1024];
    let commands =
        build_waveform_surface_commands(96, 48, samples.len(), 1, |_, i| samples[i], &config);
    let body_rects = collect_fill_rects_by_rgb(&commands, styles[0].color)
        .into_iter()
        .filter(|(_, color)| color.a == config.style.waveform_body_alpha)
        .collect::<Vec<_>>();
    assert!(
        !body_rects.is_empty(),
        "flat envelope should emit 1px body marks instead of dropping columns"
    );
}

#[test]
fn legacy_envelope_flat_signal_emits_single_pixel_body_marks() {
    let styles = [WaveformChannelStyle {
        visible: true,
        color: Color::rgb(180, 220, 250),
    }];
    let mut config = WaveformViewConfig::new(&styles);
    config.sampling_mode = WaveformSamplingMode::EnvelopeMinMax;
    config.render_quality = WaveformRenderQuality::LegacyCpuOnly;

    let samples = vec![0.0f32; 1024];
    let commands =
        build_waveform_surface_commands(96, 48, samples.len(), 1, |_, i| samples[i], &config);
    let body_rects = collect_fill_rects_by_rgb(&commands, styles[0].color);
    assert!(
        !body_rects.is_empty(),
        "legacy flat envelope should emit 1px body marks instead of zero-length lines"
    );
}

#[test]
fn styled_envelope_mode_budget_reduces_glow_polyline_count() {
    let styles = [WaveformChannelStyle {
        visible: true,
        color: Color::rgb(120, 190, 250),
    }];
    let mut high_budget = WaveformViewConfig::new(&styles);
    high_budget.sampling_mode = WaveformSamplingMode::EnvelopeMinMax;
    high_budget.render_quality = WaveformRenderQuality::AutoVectorPreferred;
    high_budget.style.waveform_outline_layers = 8;
    high_budget.max_waveform_commands = usize::MAX;
    high_budget.max_glow_points_per_channel = usize::MAX;

    let mut low_budget = high_budget.clone();
    low_budget.max_waveform_commands = 64;
    low_budget.max_glow_points_per_channel = 96;

    let samples: Vec<f32> = (0..4096)
        .map(|index| ((index as f32 * 0.017).sin() * 0.9).clamp(-1.0, 1.0))
        .collect();

    let high = build_waveform_surface_commands(
        512,
        140,
        samples.len(),
        1,
        |_, i| samples[i],
        &high_budget,
    );
    let low_a =
        build_waveform_surface_commands(512, 140, samples.len(), 1, |_, i| samples[i], &low_budget);
    let low_b =
        build_waveform_surface_commands(512, 140, samples.len(), 1, |_, i| samples[i], &low_budget);
    assert_eq!(
        low_a, low_b,
        "budgeted glow planning must stay deterministic"
    );

    let high_count = count_polyline_commands_by_rgb(&high, styles[0].color);
    let low_count = count_polyline_commands_by_rgb(&low_a, styles[0].color);
    assert!(
        low_count < high_count,
        "expected reduced polyline glow count under constrained budget ({low_count} !< {high_count})"
    );
}

#[test]
fn channel_budget_distribution_is_index_stable() {
    let budgets: Vec<usize> = (0..3)
        .map(|index| channel_waveform_budget(100, 3, index))
        .collect();
    assert_eq!(budgets, vec![34, 33, 33]);

    let zero_channel_budget = channel_waveform_budget(100, 0, 0);
    assert_eq!(zero_channel_budget, 0);
}

#[test]
fn styled_envelope_glow_polyline_count_is_content_invariant_for_fixed_geometry_budget() {
    let styles = [WaveformChannelStyle {
        visible: true,
        color: Color::rgb(140, 210, 250),
    }];
    let mut config = WaveformViewConfig::new(&styles);
    config.sampling_mode = WaveformSamplingMode::EnvelopeMinMax;
    config.render_quality = WaveformRenderQuality::AutoVectorPreferred;
    config.style.waveform_outline_layers = 6;
    config.max_waveform_commands = 144;
    config.max_glow_points_per_channel = 2048;

    let width = 128u32;
    let height = 96u32;
    let sample_count = 4096usize;
    let flat = vec![0.0f32; sample_count];
    let dense: Vec<f32> = (0..sample_count)
        .map(|index| ((index as f32 * 0.019).sin() * 0.95).clamp(-1.0, 1.0))
        .collect();

    let flat_commands =
        build_waveform_surface_commands(width, height, sample_count, 1, |_, i| flat[i], &config);
    let dense_commands =
        build_waveform_surface_commands(width, height, sample_count, 1, |_, i| dense[i], &config);

    let flat_glow_polylines = count_polyline_commands_by_rgb(&flat_commands, styles[0].color);
    let dense_glow_polylines = count_polyline_commands_by_rgb(&dense_commands, styles[0].color);
    assert_eq!(
        flat_glow_polylines, dense_glow_polylines,
        "glow polyline command count should remain stable for fixed geometry/config"
    );
}

#[test]
fn horizontal_grid_deduplicates_center_line_to_single_pixel_row() {
    let styles = [WaveformChannelStyle {
        visible: false,
        color: Color::rgb(0, 0, 0),
    }];
    let mut config = WaveformViewConfig::new(&styles);
    config.horizontal_grid_lines = 128;

    let commands = build_waveform_surface_commands(64, 8, 0, 1, |_, _| 0.0, &config);
    let center_lines = collect_lines_by_color(&commands, config.style.grid_horizontal_center);
    assert_eq!(
        center_lines.len(),
        1,
        "center horizontal line should be emitted once even when grid steps collapse to repeated y rows"
    );
    let (start, end) = center_lines[0];
    assert_eq!(start.y, end.y);
}

#[test]
fn default_center_line_color_is_subtle_gray() {
    assert_eq!(
        WaveformViewStyle::default().grid_horizontal_center,
        Color::rgb(36, 40, 45)
    );
}

#[test]
fn slice_builder_matches_callback_builder_output() {
    let styles = [WaveformChannelStyle {
        visible: true,
        color: Color::rgb(78, 190, 236),
    }];
    let mut config = WaveformViewConfig::new(&styles);
    config.sampling_mode = WaveformSamplingMode::EnvelopeMinMax;
    config.render_quality = WaveformRenderQuality::AutoVectorPreferred;

    let samples: Vec<f32> = (0..2048)
        .map(|index| ((index as f32 * 0.029).sin() * 0.85).clamp(-1.0, 1.0))
        .collect();
    let slices: [&[f32]; 1] = [&samples];

    let callback_commands =
        build_waveform_surface_commands(360, 120, samples.len(), 1, |_, i| samples[i], &config);
    let slice_commands = build_waveform_surface_commands_from_slices(360, 120, &slices, &config);
    assert_eq!(callback_commands, slice_commands);
}

#[test]
fn callback_context_reuses_samples_for_identical_revision() {
    let styles = [WaveformChannelStyle {
        visible: true,
        color: Color::rgb(210, 120, 80),
    }];
    let mut config = WaveformViewConfig::new(&styles);
    config.sampling_mode = WaveformSamplingMode::EnvelopeMinMax;
    config.render_quality = WaveformRenderQuality::AutoVectorPreferred;

    let sample_count = 1024usize;
    let samples: Vec<f32> = (0..sample_count)
        .map(|index| ((index as f32 * 0.011).sin() * 0.9).clamp(-1.0, 1.0))
        .collect();
    let callback_count = Cell::new(0usize);
    let sample_at = |_: usize, index: usize| {
        callback_count.set(callback_count.get().saturating_add(1));
        samples[index]
    };

    let mut context = WaveformRenderContext::new();
    let _ = build_waveform_surface_commands_with_context(
        256,
        96,
        sample_count,
        1,
        sample_at,
        42,
        &config,
        &mut context,
    );
    let first_count = callback_count.get();
    assert_eq!(first_count, sample_count);

    let _ = build_waveform_surface_commands_with_context(
        256,
        96,
        sample_count,
        1,
        sample_at,
        42,
        &config,
        &mut context,
    );
    assert_eq!(
        callback_count.get(),
        first_count,
        "expected identical revision to reuse callback materialization"
    );

    let _ = build_waveform_surface_commands_with_context(
        256,
        96,
        sample_count,
        1,
        sample_at,
        43,
        &config,
        &mut context,
    );
    assert_eq!(
        callback_count.get(),
        first_count + sample_count,
        "expected changed revision to trigger rematerialization"
    );
}

#[test]
fn envelope_temporal_smoothing_limits_inward_release_per_timed_step() {
    let styles = [WaveformChannelStyle {
        visible: true,
        color: Color::rgb(255, 80, 80),
    }];
    let mut config = WaveformViewConfig::new(&styles);
    config.sampling_mode = WaveformSamplingMode::EnvelopeMinMax;
    config.render_quality = WaveformRenderQuality::LegacyCpuOnly;
    config.envelope_temporal_smoothing = true;
    config.envelope_release_ms_per_pixel = 16.0;
    config.envelope_frame_delta_ms = 16.0;

    let sample_count = 256usize;
    let width = 64u32;
    let height = 64u32;
    let impulse_index = 100usize;

    let mut active = vec![0.0f32; sample_count];
    active[impulse_index] = 1.0;
    let flat = vec![0.0f32; sample_count];
    let columns = width.max(2) as usize;
    let transient_column = (impulse_index * columns) / sample_count;
    let x_max = width.max(2) as i32 - 1;
    let expected_x =
        ((transient_column as f32 / (columns - 1) as f32) * x_max as f32).round() as i32;

    let mut context = WaveformRenderContext::new();
    let frame_a = build_waveform_surface_commands_with_context(
        width,
        height,
        sample_count,
        1,
        |_, i| active[i],
        10,
        &config,
        &mut context,
    );
    let frame_b = build_waveform_surface_commands_with_context(
        width,
        height,
        sample_count,
        1,
        |_, i| flat[i],
        11,
        &config,
        &mut context,
    );

    let mut frame_a_top = None;
    for (start, end) in collect_lines_by_color(&frame_a, styles[0].color) {
        if start.x == expected_x && end.x == expected_x {
            frame_a_top =
                Some(frame_a_top.map_or(start.y.min(end.y), |v: i32| v.min(start.y.min(end.y))));
        }
    }
    let mut frame_b_top = None;
    for (start, end) in collect_lines_by_color(&frame_b, styles[0].color) {
        if start.x == expected_x && end.x == expected_x {
            frame_b_top =
                Some(frame_b_top.map_or(start.y.min(end.y), |v: i32| v.min(start.y.min(end.y))));
        }
    }

    let top_a = frame_a_top.expect("expected transient line in first frame");
    let top_b = frame_b_top.expect("expected aligned line in second frame");
    assert!(
        top_b > top_a,
        "second frame should begin releasing inward from transient top"
    );
    assert!(
        top_b - top_a <= 1,
        "inward release should be limited to configured timed pixel step"
    );
}
