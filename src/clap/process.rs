//! CLAP process context helpers for audio buffers and events.

use clack_plugin::events::io::{EventBatch, InputEvents, OutputEvents};
use clack_plugin::process::{Audio, Events, Process};

use crate::clap::events::{EventRouter, bounds_to_range};
use clack_plugin::prelude::ChannelPair;

/// Bundles the CLAP process data passed to `PluginAudioProcessor::process`.
pub struct ProcessContext<'a> {
    /// Per-call transport metadata.
    pub process: Process<'a>,
    /// Input/output audio buffers.
    pub audio: Audio<'a>,
    /// Input/output event buffers.
    pub events: Events<'a>,
}

/// Return immutable input/output channel slices and optional output destination for a
/// single channel pair.
///
/// The boolean flag reports whether the output buffer aliases the output bus for
/// in-place processing.
pub fn split_channel<'a, T>(
    pair: ChannelPair<'a, T>,
) -> (Option<&'a [T]>, Option<&'a mut [T]>, bool) {
    match pair {
        ChannelPair::InputOnly(input) => (Some(input), None, false),
        ChannelPair::OutputOnly(output) => (None, Some(output), false),
        ChannelPair::InputOutput(input, output) => (Some(input), Some(output), false),
        ChannelPair::InPlace(output) => (None, Some(output), true),
    }
}

/// Return the smallest positive buffer length across a set of optional lengths.
pub fn min_len(lengths: &[Option<usize>]) -> Option<usize> {
    lengths
        .iter()
        .copied()
        .flatten()
        .filter(|len| *len > 0)
        .min()
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

#[cfg(test)]
mod tests {
    use super::{min_len, split_channel};
    use clack_plugin::prelude::ChannelPair;

    #[test]
    fn split_channel_marks_in_place_output_paths() {
        let input = [1.0_f32, 2.0, 3.0];
        let mut in_place = [4.0_f32, 5.0, 6.0];
        let mut output = [7.0_f32, 8.0, 9.0];

        let (left_input, left_output, left_in_place) =
            split_channel(ChannelPair::InputOnly(&input));
        assert_eq!(left_input.map(|buffer| buffer.len()), Some(3));
        assert!(left_output.is_none());
        assert!(!left_in_place);

        let (out_input, out_output, out_in_place) =
            split_channel(ChannelPair::InPlace(&mut in_place));
        assert!(out_input.is_none());
        assert_eq!(out_output.map(|buffer| buffer.len()), Some(3));
        assert!(out_in_place);

        let (io_input, io_output, io_in_place) =
            split_channel(ChannelPair::InputOutput(&input, &mut output));
        assert_eq!(io_input.map(|buffer| buffer.len()), Some(3));
        assert_eq!(io_output.map(|buffer| buffer.len()), Some(3));
        assert!(!io_in_place);
    }

    #[test]
    fn min_len_ignores_missing_and_zero_lengths() {
        assert_eq!(min_len(&[Some(4), None, Some(0), Some(7)]), Some(4));
        assert_eq!(min_len(&[None, Some(0)]), None);
        assert_eq!(min_len(&[None, None]), None);
    }
}
