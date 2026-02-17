//! Time and unit conversion helpers for DSP code.
//! 
//! This module also hosts transport timing primitives shared by multiple plugin
//! implementations.

/// Transport snapshot used to update DSP timing logic from host information.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransportState {
    /// Host tempo in beats per minute.
    pub tempo_bpm: f32,
    /// Whether host playback is running.
    pub is_playing: bool,
    /// Current host song position in quarter-note beats when available.
    pub song_pos_beats: Option<f64>,
}

impl Default for TransportState {
    fn default() -> Self {
        Self {
            tempo_bpm: 120.0,
            is_playing: false,
            song_pos_beats: None,
        }
    }
}

/// Transport position information emitted by one sample tick.
#[derive(Debug, Clone, Copy)]
pub struct ClockFrame {
    /// Beat position in quarter-note units.
    pub beat_position: f64,
}

impl ClockFrame {
    /// Convert beat position into a normalized cycle phase.
    pub fn phase_for_cycle(self, beats_per_cycle: f32, phase_offset: f32) -> f32 {
        phase_from_beats(self.beat_position, beats_per_cycle, phase_offset)
    }
}

/// Running transport clock used by sample-accurate automation and modulation logic.
#[derive(Debug, Clone)]
pub struct TransportClock {
    /// Effective sample rate used to convert BPM into beat increments.
    sample_rate: f32,
    /// Fallback beat position used when host transport position is unavailable.
    fallback_beat_position: f64,
}

impl TransportClock {
    /// Create a clock for the given sample rate.
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate: sample_rate.max(1.0),
            fallback_beat_position: 0.0,
        }
    }

    /// Advance one sample and return the current transport position.
    pub fn tick(&mut self, transport: TransportState) -> ClockFrame {
        let tempo_bpm = transport.tempo_bpm.clamp(20.0, 320.0);
        let beat_increment = tempo_bpm as f64 / (self.sample_rate as f64 * 60.0);

        let beat_position = transport
            .song_pos_beats
            .unwrap_or(self.fallback_beat_position);
        if transport.is_playing {
            self.fallback_beat_position = beat_position + beat_increment;
        } else {
            self.fallback_beat_position = beat_position;
        }

        ClockFrame { beat_position }
    }
}

/// Convert beats into cycle phase in `[0, 1)`.
///
/// A small epsilon is used to avoid division by zero when plugins emit degenerate
/// cycle lengths.
pub fn phase_from_beats(beat_position: f64, beats_per_cycle: f32, phase_offset: f32) -> f32 {
    let cycle = beats_per_cycle.max(1.0e-4) as f64;
    let base = (beat_position / cycle).fract() as f32;
    (base + phase_offset).rem_euclid(1.0)
}

/// Convert milliseconds to samples for the given sample rate.
///
/// Returns at least 1 sample to avoid zero-length delays.
pub fn ms_to_samples(ms: f32, sample_rate: f32) -> usize {
    ((ms / 1000.0) * sample_rate).round().max(1.0) as usize
}

/// Convert a control-rate frequency to a sample interval.
///
/// Returns at least 1 sample to avoid zero-length intervals.
pub fn hz_to_samples(rate_hz: f32, sample_rate: f32) -> usize {
    if rate_hz <= 0.0 {
        return 1;
    }
    (sample_rate / rate_hz).round().max(1.0) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ms_to_samples_rounds_up_to_one() {
        assert_eq!(ms_to_samples(0.0, 48_000.0), 1);
    }

    #[test]
    fn hz_to_samples_rounds_up_to_one() {
        assert_eq!(hz_to_samples(0.0, 48_000.0), 1);
        assert_eq!(hz_to_samples(1.0, 48_000.0), 48_000);
    }

    #[test]
    fn transport_clock_advances_when_playing_without_song_position() {
        let mut clock = TransportClock::new(48_000.0);
        let a = clock.tick(TransportState {
            tempo_bpm: 120.0,
            is_playing: true,
            song_pos_beats: None,
        });
        let b = clock.tick(TransportState {
            tempo_bpm: 120.0,
            is_playing: true,
            song_pos_beats: None,
        });
        assert!(b.beat_position > a.beat_position);
    }

    #[test]
    fn phase_from_beats_wraps_to_unit_interval() {
        let phase = phase_from_beats(9.0, 1.0, 0.75);
        assert!((0.0..1.0).contains(&phase));
    }

    #[test]
    fn transport_state_formats_debug_output() {
        let state = TransportState::default();
        assert!(format!("{state:?}").contains("tempo_bpm"));
    }
}
