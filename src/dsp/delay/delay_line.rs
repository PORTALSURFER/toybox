/// A fixed-size delay line with adjustable delay length.
#[derive(Debug, Clone)]
pub struct DelayLine {
    /// Circular sample buffer storing delayed samples.
    buffer: Vec<f32>,
    /// Current write position inside the circular buffer.
    write_idx: usize,
    /// Active delay length in samples.
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
