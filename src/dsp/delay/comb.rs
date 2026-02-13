/// A feedback comb filter used for late reverb tails.
#[derive(Debug, Clone)]
pub struct FeedbackComb {
    /// Delay line used for comb feedback.
    delay: DelayLine,
    /// Feedback gain applied to the damped delayed sample.
    feedback: f32,
    /// Damping low-pass filter inside the feedback path.
    damping: super::filters::OnePole,
}

impl FeedbackComb {
    /// Create a new comb filter with the given delay line.
    pub fn new(delay: DelayLine) -> Self {
        Self {
            delay,
            feedback: 0.7,
            damping: super::filters::OnePole::new(0.2),
        }
    }

    /// Set the feedback gain (0.0..1.0).
    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(0.0, 0.995);
    }

    /// Set the damping coefficient (0.0..1.0).
    pub fn set_damping(&mut self, coefficient: f32) {
        self.damping.set_coefficient(coefficient);
    }

    /// Set the delay length in samples.
    pub fn set_delay_samples(&mut self, delay_samples: usize) {
        self.delay.set_delay_samples(delay_samples);
    }

    /// Process a sample through the comb filter.
    pub fn process(&mut self, input: f32) -> f32 {
        let delayed = self.delay.read();
        let filtered = self.damping.process(delayed);
        let feedback = filtered * self.feedback;
        self.delay.write(input + feedback);
        delayed
    }
}
