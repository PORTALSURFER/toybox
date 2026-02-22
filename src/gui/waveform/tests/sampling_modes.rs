use super::*;

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
fn envelope_mode_phase_repeats_when_start_sample_shifts_by_columns_across_sample_wrap() {
    let styles = [WaveformChannelStyle {
        visible: true,
        color: Color::rgb(200, 160, 96),
    }];
    let mut config = WaveformViewConfig::new(&styles);
    config.sampling_mode = WaveformSamplingMode::EnvelopeMinMax;
    config.render_quality = WaveformRenderQuality::LegacyCpuOnly;

    let sample_count = 97usize;
    let width = 53u32;
    let columns = width.max(2) as u64;
    let samples: Vec<f32> = (0..sample_count)
        .map(|index| ((index as f32 * 0.123).sin() * 0.9).clamp(-1.0, 1.0))
        .collect();

    config.start_sample = 60;
    let a = build_waveform_surface_commands(width, 80, sample_count, 1, |_, i| samples[i], &config);
    config.start_sample = 60 + columns;
    let b = build_waveform_surface_commands(width, 80, sample_count, 1, |_, i| samples[i], &config);

    assert_eq!(
        a, b,
        "phase alignment should depend on start_sample modulo output columns"
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
