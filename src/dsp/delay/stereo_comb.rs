/// A stereo pair of feedback comb filters for widening effects.
#[derive(Debug, Clone)]
pub struct StereoComb {
    /// Left comb filter instance.
    left: FeedbackComb,
    /// Right comb filter instance.
    right: FeedbackComb,
}

impl StereoComb {
    /// Create a new stereo comb pair with the given max delay size.
    pub fn new(max_delay_samples: usize) -> Self {
        Self {
            left: FeedbackComb::new(DelayLine::new(max_delay_samples)),
            right: FeedbackComb::new(DelayLine::new(max_delay_samples)),
        }
    }

    /// Set delay lengths for left/right channels.
    pub fn set_delay_samples(&mut self, left: usize, right: usize) {
        self.left.set_delay_samples(left);
        self.right.set_delay_samples(right);
    }

    /// Set shared feedback amount.
    pub fn set_feedback(&mut self, feedback: f32) {
        self.left.set_feedback(feedback);
        self.right.set_feedback(feedback);
    }

    /// Set shared damping coefficient.
    pub fn set_damping(&mut self, coefficient: f32) {
        self.left.set_damping(coefficient);
        self.right.set_damping(coefficient);
    }

    /// Process a sample and return stereo output.
    pub fn process(&mut self, input: f32) -> (f32, f32) {
        let left = self.left.process(input);
        let right = self.right.process(input);
        (left, right)
    }
}
