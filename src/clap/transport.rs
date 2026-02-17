//! Transport event conversion helpers for CLAP-aware DSP paths.

use clack_plugin::events::event_types::{TransportEvent, TransportFlags};

use crate::dsp::TransportState;

/// Convert an optional CLAP transport event into a framework transport snapshot.
pub fn transport_state_from_transport(transport: Option<TransportEvent>) -> TransportState {
    match transport {
        Some(event) => TransportState {
            tempo_bpm: if event.flags.contains(TransportFlags::HAS_TEMPO) {
                event.tempo as f32
            } else {
                120.0
            },
            is_playing: event.flags.contains(TransportFlags::IS_PLAYING),
            song_pos_beats: if event.flags.contains(TransportFlags::HAS_BEATS_TIMELINE) {
                Some(event.song_pos_beats.to_float())
            } else {
                None
            },
        },
        None => TransportState::default(),
    }
}
