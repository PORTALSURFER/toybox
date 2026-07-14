//! External compile and behavior coverage for the reusable oversampling API.

use toybox::dsp::{
    DryPathAligner, HalfBandDecimator2x, HalfBandInterpolator2x, MonoOversampler,
    MonoOversampler2x, MonoOversampler4x, OversamplingFactor, SourceDecimator2x, SourceDecimator4x,
};

/// Prove downstream plugins can name and exercise every public primitive.
#[test]
fn downstream_can_compose_fixed_oversampling_stages() {
    let mut interpolator = HalfBandInterpolator2x::new();
    let mut decimator = HalfBandDecimator2x::new();
    let high = interpolator.process(0.25);
    assert!(decimator.process(high).is_finite());

    let mut two = MonoOversampler2x::new();
    let mut four = MonoOversampler4x::new();
    assert!(two.process(0.1, f32::tanh).is_finite());
    assert!(four.process(0.1, f32::tanh).is_finite());

    let mut selected = MonoOversampler::new(OversamplingFactor::Four);
    assert_eq!(selected.factor(), OversamplingFactor::Four);
    assert_eq!(selected.latency().numerator(), 165);
    assert_eq!(selected.latency().denominator(), 2);
    assert!(selected.process(0.1, f32::tanh).is_finite());

    let mut source_two = SourceDecimator2x::new();
    let mut source_four = SourceDecimator4x::new();
    assert!(source_two.process([0.0, 0.1]).is_finite());
    assert!(source_four.process([0.0, 0.1, 0.2, 0.3]).is_finite());

    let mut dry = DryPathAligner::new(OversamplingFactor::Four);
    assert_eq!(dry.latency(), selected.latency());
    assert!(dry.process(0.25).is_finite());
}
