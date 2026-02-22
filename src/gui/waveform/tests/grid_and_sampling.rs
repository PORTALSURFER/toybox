use super::*;

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
