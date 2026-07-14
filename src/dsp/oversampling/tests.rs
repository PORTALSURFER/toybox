use super::*;
use crate::test_alloc::assert_realtime_safe;
use std::f32::consts::TAU;

/// Length of coherent spectral test captures.
const SPECTRAL_LEN: usize = 4096;
/// Settling time discarded before spectral measurements.
const SETTLING_SAMPLES: usize = 1024;

/// Return one coefficient from the expanded half-band impulse response.
fn full_coefficient(index: usize) -> f64 {
    if index.is_multiple_of(2) {
        f64::from(EVEN_COEFFICIENTS[index / 2])
    } else if index == FILTER_TAP_COUNT / 2 {
        0.5
    } else {
        0.0
    }
}

/// Evaluate the FIR magnitude at a frequency normalized to high-rate Nyquist.
fn response_magnitude(normalized_nyquist: f64) -> f64 {
    let angular = std::f64::consts::PI * normalized_nyquist;
    let mut real = 0.0;
    let mut imaginary = 0.0;
    for index in 0..FILTER_TAP_COUNT {
        let coefficient = full_coefficient(index);
        real += coefficient * (angular * index as f64).cos();
        imaginary -= coefficient * (angular * index as f64).sin();
    }
    real.hypot(imaginary)
}

/// Convert a positive magnitude to decibels with a numerical floor.
fn magnitude_db(magnitude: f64) -> f64 {
    20.0 * magnitude.max(1.0e-15).log10()
}

/// Return the coherent single-bin power of a real signal.
fn bin_power(signal: &[f32], bin: usize) -> f64 {
    let angular = std::f64::consts::TAU * bin as f64 / signal.len() as f64;
    let mut real = 0.0;
    let mut imaginary = 0.0;
    for (index, sample) in signal.iter().enumerate() {
        let phase = angular * index as f64;
        real += f64::from(*sample) * phase.cos();
        imaginary -= f64::from(*sample) * phase.sin();
    }
    let scale = 2.0 / signal.len() as f64;
    (real * scale).powi(2) + (imaginary * scale).powi(2)
}

/// Sum coherent power in a set of unique bins.
fn bins_power(signal: &[f32], bins: &[usize]) -> f64 {
    bins.iter().map(|bin| bin_power(signal, *bin)).sum()
}

/// Fold an unbounded positive DFT bin into the real-signal Nyquist interval.
fn folded_bin(raw_bin: usize, length: usize) -> usize {
    let wrapped = raw_bin % length;
    wrapped.min(length - wrapped)
}

/// Collect unique aliased odd-harmonic bins that do not overlap the fundamental.
fn nonlinear_alias_bins(fundamental: usize, max_harmonic: usize) -> Vec<usize> {
    let mut bins = Vec::new();
    let stopband_start = (SPECTRAL_LEN as f64 * 0.55).ceil() as usize;
    for harmonic in (3..=max_harmonic).step_by(2) {
        let raw = fundamental * harmonic;
        if raw < stopband_start {
            continue;
        }
        let bin = folded_bin(raw, SPECTRAL_LEN);
        if bin != fundamental && bin != 0 && !bins.contains(&bin) {
            bins.push(bin);
        }
    }
    bins
}

/// Render a nonlinear base-rate reference and matching 2x/4x paths.
fn render_nonlinear(
    sample_rate: f64,
    fundamental_bin: usize,
    operation: fn(f32) -> f32,
) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let total = SETTLING_SAMPLES + SPECTRAL_LEN;
    let mut direct = Vec::with_capacity(SPECTRAL_LEN);
    let mut two_output = Vec::with_capacity(SPECTRAL_LEN);
    let mut four_output = Vec::with_capacity(SPECTRAL_LEN);
    let mut two = MonoOversampler2x::new();
    let mut four = MonoOversampler4x::new();
    let frequency_hz = fundamental_bin as f64 * sample_rate / SPECTRAL_LEN as f64;
    for index in 0..total {
        let phase = std::f64::consts::TAU * frequency_hz * index as f64 / sample_rate;
        let input = phase.sin() as f32;
        let direct_sample = operation(input);
        let two_sample = two.process(input, operation);
        let four_sample = four.process(input, operation);
        if index >= SETTLING_SAMPLES {
            direct.push(direct_sample);
            two_output.push(two_sample);
            four_output.push(four_sample);
        }
    }
    (direct, two_output, four_output)
}

/// Hard clip fixture used by spectral tests.
fn hard_clip(sample: f32) -> f32 {
    (sample * 2.5).clamp(-0.55, 0.55)
}

/// Symmetric triangle-fold fixture used by spectral tests.
fn foldback(sample: f32) -> f32 {
    let shifted = (sample * 3.25 + 1.0).rem_euclid(4.0);
    1.0 - (shifted - 2.0).abs()
}

#[test]
fn half_band_response_meets_documented_bounds() {
    let mut minimum_passband_db = f64::INFINITY;
    let mut maximum_passband_db = f64::NEG_INFINITY;
    let mut maximum_stopband_db = f64::NEG_INFINITY;
    for step in 0..=4000 {
        let pass_frequency = 0.45 * step as f64 / 4000.0;
        let pass_db = magnitude_db(response_magnitude(pass_frequency));
        minimum_passband_db = minimum_passband_db.min(pass_db);
        maximum_passband_db = maximum_passband_db.max(pass_db);

        let stop_frequency = 0.55 + 0.45 * step as f64 / 4000.0;
        maximum_stopband_db =
            maximum_stopband_db.max(magnitude_db(response_magnitude(stop_frequency)));
    }

    assert!(maximum_passband_db - minimum_passband_db < 0.001);
    assert!(maximum_stopband_db < -85.0);
    assert!((response_magnitude(0.0) - 1.0).abs() < 1.0e-7);
    assert!((magnitude_db(response_magnitude(0.5)) + 6.0206).abs() < 0.001);
}

#[test]
fn latency_contract_is_exact_and_exposes_fractional_delays() {
    assert_eq!(HalfBandInterpolator2x::latency(), SampleDelay::new(55, 2));
    assert_eq!(HalfBandDecimator2x::latency(), SampleDelay::new(55, 2));
    assert_eq!(MonoOversampler2x::latency(), SampleDelay::new(55, 1));
    assert_eq!(MonoOversampler4x::latency(), SampleDelay::new(165, 2));
    assert_eq!(SourceDecimator2x::latency(), SampleDelay::new(55, 2));
    assert_eq!(SourceDecimator4x::latency(), SampleDelay::new(165, 4));
    assert!(MonoOversampler2x::latency().is_integer());
    assert!(!MonoOversampler4x::latency().is_integer());
    assert_eq!(MonoOversampler4x::latency().whole_samples(), 82);
    assert_eq!(MonoOversampler4x::latency().fractional_numerator(), 1);
    assert!((SourceDecimator4x::latency().as_f64() - 41.25).abs() < f64::EPSILON);
}

#[test]
fn identity_impulses_land_at_reported_processing_delays() {
    let mut two = MonoOversampler2x::new();
    let two_impulse: Vec<_> = (0..120)
        .map(|index| two.process(f32::from(index == 0), |sample| sample))
        .collect();
    let two_peak = two_impulse
        .iter()
        .enumerate()
        .max_by(|(_, left), (_, right)| left.total_cmp(right))
        .map(|(index, _)| index)
        .unwrap();
    assert_eq!(two_peak, 55);

    let mut four = MonoOversampler4x::new();
    let four_impulse: Vec<_> = (0..160)
        .map(|index| four.process(f32::from(index == 0), |sample| sample))
        .collect();
    assert!((four_impulse[82] - four_impulse[83]).abs() < 1.0e-6);
    assert!(four_impulse[82] > four_impulse[81]);
    assert!(four_impulse[83] > four_impulse[84]);
}

#[test]
fn dry_path_alignment_matches_integer_and_half_sample_delays() {
    let mut two = DryPathAligner::new(OversamplingFactor::Two);
    let two_impulse: Vec<_> = (0..100)
        .map(|index| two.process(f32::from(index == 0)))
        .collect();
    assert_eq!(two_impulse[55], 1.0);
    assert_eq!(two_impulse.iter().sum::<f32>(), 1.0);

    let mut four = DryPathAligner::new(OversamplingFactor::Four);
    let four_impulse: Vec<_> = (0..120)
        .map(|index| four.process(f32::from(index == 0)))
        .collect();
    assert_eq!(four_impulse[82], 0.5);
    assert_eq!(four_impulse[83], 0.5);
    assert_eq!(four_impulse.iter().sum::<f32>(), 1.0);
}

#[test]
fn reset_restores_deterministic_interpolator_decimator_and_dry_state() {
    let input: Vec<_> = (0..256)
        .map(|index| (TAU * 0.071 * index as f32).sin())
        .collect();
    let mut oversampler = MonoOversampler::new(OversamplingFactor::Four);
    let first: Vec<_> = input
        .iter()
        .map(|sample| oversampler.process(*sample, foldback))
        .collect();
    oversampler.reset();
    let second: Vec<_> = input
        .iter()
        .map(|sample| oversampler.process(*sample, foldback))
        .collect();
    assert_eq!(first, second);

    let mut source = SourceDecimator4x::new();
    let source_first: Vec<_> = input
        .chunks_exact(4)
        .map(|chunk| source.process([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();
    source.reset();
    let source_second: Vec<_> = input
        .chunks_exact(4)
        .map(|chunk| source.process([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();
    assert_eq!(source_first, source_second);

    let mut dry = DryPathAligner::new(OversamplingFactor::Four);
    let dry_first: Vec<_> = input.iter().map(|sample| dry.process(*sample)).collect();
    dry.reset();
    let dry_second: Vec<_> = input.iter().map(|sample| dry.process(*sample)).collect();
    assert_eq!(dry_first, dry_second);
}

#[test]
fn sample_processing_is_independent_of_caller_block_partitioning() {
    let input: Vec<_> = (0..2048)
        .map(|index| {
            let phase = TAU * 0.037 * index as f32;
            phase.sin() * 0.8 + (phase * 3.1).cos() * 0.2
        })
        .collect();
    let mut contiguous = MonoOversampler4x::new();
    let expected: Vec<_> = input
        .iter()
        .map(|sample| contiguous.process(*sample, hard_clip))
        .collect();

    let mut partitioned = MonoOversampler4x::new();
    let mut actual = Vec::with_capacity(input.len());
    let block_sizes = [1, 16, 64, 512, 2048];
    let mut cursor = 0;
    let mut block = 0;
    while cursor < input.len() {
        let end = (cursor + block_sizes[block % block_sizes.len()]).min(input.len());
        actual.extend(
            input[cursor..end]
                .iter()
                .map(|sample| partitioned.process(*sample, hard_clip)),
        );
        cursor = end;
        block += 1;
    }
    assert_eq!(expected, actual);
}

#[test]
fn callbacks_run_exactly_once_per_high_rate_sample() {
    let mut two = MonoOversampler2x::new();
    let mut two_calls = 0;
    let _ = two.process(0.25, |sample| {
        two_calls += 1;
        sample
    });
    assert_eq!(two_calls, 2);

    let mut four = MonoOversampler4x::new();
    let mut four_calls = 0;
    let _ = four.process(0.25, |sample| {
        four_calls += 1;
        sample
    });
    assert_eq!(four_calls, 4);
}

#[test]
fn dc_and_low_frequency_gain_are_preserved_after_settling() {
    let mut two_source = SourceDecimator2x::new();
    let mut four_source = SourceDecimator4x::new();
    let mut two = 0.0;
    let mut four = 0.0;
    for _ in 0..512 {
        two = two_source.process([1.0, 1.0]);
        four = four_source.process([1.0, 1.0, 1.0, 1.0]);
    }
    assert!((two - 1.0).abs() < 1.0e-6);
    assert!((four - 1.0).abs() < 2.0e-6);

    let mut processor = MonoOversampler4x::new();
    let mut input_energy = 0.0_f64;
    let mut output_energy = 0.0_f64;
    for index in 0..(1024 + 8192) {
        let input = (std::f64::consts::TAU * 7.0 * index as f64 / 8192.0).sin() as f32;
        let output = processor.process(input, |sample| sample);
        if index >= 1024 {
            input_energy += f64::from(input).powi(2);
            output_energy += f64::from(output).powi(2);
        }
    }
    let gain = (output_energy / input_energy).sqrt();
    assert!((gain - 1.0).abs() < 2.0e-5, "low-frequency gain was {gain}");
}

#[test]
fn processing_remains_finite_at_supported_sample_rates() {
    for sample_rate in [44_100.0_f32, 48_000.0, 96_000.0, 192_000.0] {
        let mut two = MonoOversampler2x::new();
        let mut four = MonoOversampler4x::new();
        for index in 0..4096 {
            let phase = TAU * 9973.0 * index as f32 / sample_rate;
            let input = phase.sin();
            let two_output = two.process(input, foldback);
            let four_output = four.process(input, hard_clip);
            assert!(two_output.is_finite());
            assert!(four_output.is_finite());
            assert!(two_output.abs() < 2.0);
            assert!(four_output.abs() < 2.0);
        }
    }
}

#[test]
fn hard_clip_and_foldback_alias_energy_falls_with_oversampling() {
    for sample_rate in [44_100.0, 48_000.0, 96_000.0] {
        for (fundamental, fixture) in [
            (857, hard_clip as fn(f32) -> f32),
            (691, foldback as fn(f32) -> f32),
        ] {
            let aliases = nonlinear_alias_bins(fundamental, 21);
            let (direct, two, four) = render_nonlinear(sample_rate, fundamental, fixture);
            let direct_alias = bins_power(&direct, &aliases);
            let two_alias = bins_power(&two, &aliases);
            let four_alias = bins_power(&four, &aliases);
            assert!(
                two_alias < direct_alias * 0.08,
                "at {sample_rate} Hz, 2x alias ratio was {}; 4x was {}",
                two_alias / direct_alias,
                four_alias / direct_alias
            );
            assert!(
                four_alias < direct_alias * 0.02,
                "at {sample_rate} Hz, 4x alias ratio was {}",
                four_alias / direct_alias
            );
        }
    }
}

#[test]
fn fm_like_source_alias_energy_falls_after_4x_decimation() {
    let carrier_bin = 701_isize;
    let modulation_bin = 173_isize;
    let modulation_index = 8.0_f32;
    let mut legitimate = Vec::new();
    let mut aliases = Vec::new();
    for sideband in -20_isize..=20 {
        let raw = carrier_bin + sideband * modulation_bin;
        let absolute = raw.unsigned_abs();
        let folded = folded_bin(absolute, SPECTRAL_LEN);
        if absolute <= SPECTRAL_LEN / 2 {
            legitimate.push(folded);
        } else if absolute >= (SPECTRAL_LEN as f64 * 0.55).ceil() as usize
            && folded != 0
            && !legitimate.contains(&folded)
            && !aliases.contains(&folded)
        {
            aliases.push(folded);
        }
    }
    aliases.retain(|bin| !legitimate.contains(bin));

    for sample_rate in [44_100.0_f64, 48_000.0, 96_000.0] {
        let total = SETTLING_SAMPLES + SPECTRAL_LEN;
        let carrier_hz = carrier_bin as f64 * sample_rate / SPECTRAL_LEN as f64;
        let modulation_hz = modulation_bin as f64 * sample_rate / SPECTRAL_LEN as f64;
        let high_sample_rate = sample_rate * 4.0;
        let mut direct = Vec::with_capacity(SPECTRAL_LEN);
        let mut decimated = Vec::with_capacity(SPECTRAL_LEN);
        let mut source = SourceDecimator4x::new();

        for base_index in 0..total {
            let base_phase = std::f64::consts::TAU * carrier_hz * base_index as f64 / sample_rate;
            let base_mod = std::f64::consts::TAU * modulation_hz * base_index as f64 / sample_rate;
            let base_sample =
                (base_phase + f64::from(modulation_index) * base_mod.sin()).sin() as f32;
            let mut high = [0.0; 4];
            for (phase_index, sample) in high.iter_mut().enumerate() {
                let high_index = base_index * 4 + phase_index;
                let carrier =
                    std::f64::consts::TAU * carrier_hz * high_index as f64 / high_sample_rate;
                let modulation =
                    std::f64::consts::TAU * modulation_hz * high_index as f64 / high_sample_rate;
                *sample = (carrier + f64::from(modulation_index) * modulation.sin()).sin() as f32;
            }
            let output = source.process(high);
            if base_index >= SETTLING_SAMPLES {
                direct.push(base_sample);
                decimated.push(output);
            }
        }

        let direct_alias = bins_power(&direct, &aliases);
        let decimated_alias = bins_power(&decimated, &aliases);
        assert!(
            decimated_alias < direct_alias * 0.02,
            "at {sample_rate} Hz, FM-like source alias ratio was {}",
            decimated_alias / direct_alias
        );
    }
}

#[test]
fn post_construction_audio_operations_do_not_allocate_or_deallocate() {
    let mut two = MonoOversampler2x::new();
    let mut four = MonoOversampler4x::new();
    let mut selected = MonoOversampler::new(OversamplingFactor::Four);
    let mut source_two = SourceDecimator2x::new();
    let mut source_four = SourceDecimator4x::new();
    let mut dry = DryPathAligner::new(OversamplingFactor::Four);

    assert_realtime_safe(|| {
        for index in 0..2048 {
            let input = (index as f32 * 0.013).sin();
            let _ = two.process(input, hard_clip);
            let _ = four.process(input, foldback);
            let _ = selected.process(input, hard_clip);
            let _ = source_two.process([input, -input]);
            let _ = source_four.process([input, -input, input * 0.5, -input * 0.5]);
            let _ = dry.process(input);
        }
        two.reset();
        four.reset();
        selected.reset();
        source_two.reset();
        source_four.reset();
        dry.reset();
    });
}
