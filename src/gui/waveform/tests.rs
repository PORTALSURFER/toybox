use super::*;

fn collect_lines_by_color(commands: &[SurfaceCommand], color: Color) -> Vec<(Point, Point)> {
    commands
        .iter()
        .filter_map(|command| match command {
            SurfaceCommand::Line {
                start,
                end,
                color: command_color,
            } if *command_color == color => Some((*start, *end)),
            _ => None,
        })
        .collect()
}

fn collect_lines_by_rgb(commands: &[SurfaceCommand], color: Color) -> Vec<(Point, Point, Color)> {
    commands
        .iter()
        .filter_map(|command| match command {
            SurfaceCommand::Line {
                start,
                end,
                color: command_color,
            } if command_color.r == color.r
                && command_color.g == color.g
                && command_color.b == color.b =>
            {
                Some((*start, *end, *command_color))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn tempo_locked_grid_lines_are_phase_stable() {
    let one_bar = WaveformGridMode::TempoLocked {
        beats_visible: 4.0,
        beats_per_bar: 4.0,
        subdivisions_per_beat: 2,
    };
    let a = grid::vertical_grid_lines(one_bar, 320);
    let b = grid::vertical_grid_lines(one_bar, 320);
    assert_eq!(a, b);
}

#[test]
fn linear_resample_interpolates_between_endpoints() {
    let samples = [0.0f32, 1.0];
    let resampled =
        sampling::resample_channel_linear(samples.len(), 0, 5, &|_, index| samples[index]);
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

    let left_overlay = collect_lines_by_color(&overlay_commands, styles[0].color);
    let right_overlay = collect_lines_by_color(&overlay_commands, styles[1].color);
    assert_eq!(left_overlay.len(), width as usize);
    assert_eq!(right_overlay.len(), width as usize);

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

    let left_split = collect_lines_by_color(&split_commands, styles[0].color);
    let right_split = collect_lines_by_color(&split_commands, styles[1].color);
    assert_eq!(left_split.len(), width as usize);
    assert_eq!(right_split.len(), width as usize);

    for (start, end) in left_split {
        assert!((0..=50).contains(&start.y));
        assert!((0..=50).contains(&end.y));
    }
    for (start, end) in right_split {
        assert!((50..=100).contains(&start.y));
        assert!((50..=100).contains(&end.y));
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
