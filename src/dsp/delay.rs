//! Delay-line building blocks for reverb and modulation effects.

/// A fixed-size delay line with adjustable delay length.
#[derive(Debug, Clone)]
pub struct DelayLine {
    buffer: Vec<f32>,
    write_idx: usize,
    delay_samples: usize,
}

impl DelayLine {
    /// Create a new delay line with a maximum delay length in samples.
    ///
    /// The buffer is sized to `max_delay_samples + 1` to ensure at least
    /// a single-sample delay is always possible.
    pub fn new(max_delay_samples: usize) -> Self {
        let size = max_delay_samples.max(1) + 1;
        Self {
            buffer: vec![0.0; size],
            write_idx: 0,
            delay_samples: 1,
        }
    }

    /// Set the current delay length in samples.
    pub fn set_delay_samples(&mut self, delay_samples: usize) {
        self.delay_samples = delay_samples.clamp(1, self.buffer.len().saturating_sub(1));
    }

    /// Read the delayed sample without advancing the write head.
    pub fn read(&self) -> f32 {
        let len = self.buffer.len();
        let read_idx = (self.write_idx + len - self.delay_samples) % len;
        self.buffer[read_idx]
    }

    /// Write a sample into the delay buffer and advance the write head.
    pub fn write(&mut self, sample: f32) {
        self.buffer[self.write_idx] = sample;
        self.write_idx = (self.write_idx + 1) % self.buffer.len();
    }

    /// Convenience method for a pure delay pass-through (no feedback).
    pub fn process(&mut self, input: f32) -> f32 {
        let out = self.read();
        self.write(input);
        out
    }
}

/// A feedback comb filter used for late reverb tails.
#[derive(Debug, Clone)]
pub struct FeedbackComb {
    delay: DelayLine,
    feedback: f32,
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

/// A stereo pair of feedback comb filters for widening effects.
#[derive(Debug, Clone)]
pub struct StereoComb {
    left: FeedbackComb,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delay_line_delays_by_one_sample() {
        let mut delay = DelayLine::new(8);
        delay.set_delay_samples(1);

        let out0 = delay.process(1.0);
        let out1 = delay.process(0.0);

        assert_eq!(out0, 0.0);
        assert_eq!(out1, 1.0);
    }

    #[test]
    fn comb_feedback_accumulates_energy() {
        let mut comb = FeedbackComb::new(DelayLine::new(8));
        comb.set_delay_samples(1);
        comb.set_feedback(0.9);

        let first = comb.process(1.0);
        let second = comb.process(0.0);

        assert!(first.abs() <= 1.0);
        assert!(second.abs() > 0.0);
    }
}
