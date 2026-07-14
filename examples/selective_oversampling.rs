//! Downstream-style selective oversampling without oversampling a whole voice.

use toybox::dsp::{MonoOversampler, OversamplingFactor, SourceDecimator4x};

/// Plugin-owned policy and sound-design stages.
struct SelectiveStages {
    /// Oversampler used only around a nonlinear drive stage.
    drive: MonoOversampler,
    /// Decimator used only for a high-rate oscillator/source stage.
    folded_source: SourceDecimator4x,
}

impl SelectiveStages {
    /// Construct the stages using plugin-selected quality policy.
    fn new() -> Self {
        Self {
            drive: MonoOversampler::new(OversamplingFactor::Four),
            folded_source: SourceDecimator4x::new(),
        }
    }

    /// Process one voice sample while leaving the intentionally aliased crusher direct.
    fn process(
        &mut self,
        input: f32,
        generated_folded_samples: [f32; 4],
        crusher_enabled: bool,
    ) -> f32 {
        let driven = self.drive.process(input, |sample| (sample * 2.5).tanh());
        let folded = self.folded_source.process(generated_folded_samples);
        let mixed = driven + folded * 0.35;
        if crusher_enabled {
            // Plugin policy intentionally keeps this stage aliased.
            (mixed * 16.0).round() / 16.0
        } else {
            mixed
        }
    }
}

/// Exercise the selective stage wiring.
fn main() {
    let mut stages = SelectiveStages::new();
    let output = stages.process(0.25, [0.1, 0.2, -0.1, -0.2], false);
    assert!(output.is_finite());
}
