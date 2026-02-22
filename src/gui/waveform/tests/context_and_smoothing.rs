use super::*;
use std::cell::Cell;

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
