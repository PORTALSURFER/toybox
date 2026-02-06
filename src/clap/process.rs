//! CLAP process context helpers for audio buffers and events.

use clack_plugin::events::io::{EventBatch, InputEvents, OutputEvents};
use clack_plugin::process::{Audio, Events, Process};

use crate::clap::events::{EventRouter, bounds_to_range};

/// Bundles the CLAP process data passed to `PluginAudioProcessor::process`.
pub struct ProcessContext<'a> {
    /// Per-call transport metadata.
    pub process: Process<'a>,
    /// Input/output audio buffers.
    pub audio: Audio<'a>,
    /// Input/output event buffers.
    pub events: Events<'a>,
}

impl<'a> ProcessContext<'a> {
    /// Create a new process context wrapper.
    pub fn new(process: Process<'a>, audio: Audio<'a>, events: Events<'a>) -> Self {
        Self {
            process,
            audio,
            events,
        }
    }

    /// Access the input event list.
    pub fn input_events(&self) -> &InputEvents<'a> {
        self.events.input
    }

    /// Access the output event list.
    pub fn output_events(&mut self) -> &mut OutputEvents<'a> {
        self.events.output
    }

    /// Create an `EventRouter` for this process call.
    pub fn event_router(&self) -> EventRouter<'a> {
        EventRouter::new(self.events.input)
    }

    /// Convert a batch's sample bounds to a concrete range for a buffer length.
    pub fn batch_range(&self, batch: &EventBatch<'_>, buffer_len: usize) -> Option<(usize, usize)> {
        bounds_to_range(batch.sample_bounds(), buffer_len)
    }
}
