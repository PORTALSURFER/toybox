use super::*;

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
        .map(|index| build::channel_waveform_budget(100, 3, index))
        .collect();
    assert_eq!(budgets, vec![34, 33, 33]);

    let zero_channel_budget = build::channel_waveform_budget(100, 0, 0);
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
