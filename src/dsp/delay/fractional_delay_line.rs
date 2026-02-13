/// A delay line that supports fractional delay lengths with linear interpolation.
#[derive(Debug, Clone)]
pub struct FractionalDelayLine {
    /// Circular sample buffer storing delayed samples.
    buffer: Vec<f32>,
    /// Current write position inside the circular buffer.
    write_idx: usize,
    /// Active fractional delay length in samples.
    delay_samples: f32,
}

impl FractionalDelayLine {
    /// Create a new fractional delay line with a maximum delay length in samples.
    ///
    /// The buffer is sized to `max_delay_samples + 1` to ensure at least
    /// a single-sample delay is always possible.
    pub fn new(max_delay_samples: usize) -> Self {
        let size = max_delay_samples.max(1) + 1;
        Self {
            buffer: vec![0.0; size],
            write_idx: 0,
            delay_samples: 1.0,
        }
    }

    /// Set the current delay length in samples, supporting fractional values.
    pub fn set_delay_samples(&mut self, delay_samples: f32) {
        let max = (self.buffer.len().saturating_sub(1)).max(1) as f32;
        self.delay_samples = delay_samples.clamp(1.0, max);
    }

    /// Read the delayed sample without advancing the write head.
    pub fn read(&self) -> f32 {
        let len = self.buffer.len() as f32;
        let mut read_pos = self.write_idx as f32 - self.delay_samples;
        if read_pos < 0.0 {
            read_pos += len;
        }
        let base = read_pos.floor();
        let frac = read_pos - base;
        let idx0 = (base as usize) % self.buffer.len();
        let idx1 = (idx0 + 1) % self.buffer.len();
        let a = self.buffer[idx0];
        let b = self.buffer[idx1];
        a + (b - a) * frac
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
