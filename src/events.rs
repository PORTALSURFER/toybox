//! Format-neutral, realtime-safe block event scheduling.
//!
//! [`BlockEventTimeline`] reserves all storage at construction time and never
//! grows it while ingesting or ordering a process block. Events are ordered by
//! sample offset, with parameter changes before other events at the same
//! offset, followed by their original source order.
//!
//! A processor renders up to each batch boundary, applies the batch in order,
//! and then continues rendering. A batch at `frame_count` carries final state
//! changes without indexing past the audio buffer.
//!
//! ```
//! use toybox::events::{BlockEvent, BlockEventTimeline};
//!
//! let mut timeline = BlockEventTimeline::<(u32, f32), u8>::with_capacity(16);
//! timeline.begin_block(64);
//! timeline.push_event(32, 60); // note
//! timeline.push_parameter(32, (7, 0.5));
//! timeline.prepare();
//!
//! let batch = timeline.next_batch().expect("event boundary");
//! assert_eq!(batch.sample_offset(), 32);
//! assert!(matches!(batch.events()[0].event(), BlockEvent::Parameter(_)));
//! assert!(matches!(batch.events()[1].event(), BlockEvent::Event(60)));
//! ```

use std::collections::TryReserveError;

/// The payload carried by one scheduled block event.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BlockEvent<P, E> {
    /// A parameter change, applied before non-parameter events at the same offset.
    Parameter(P),
    /// A note, MIDI, transport, or other non-parameter event.
    Event(E),
}

impl<P, E> BlockEvent<P, E> {
    /// Return the deterministic equal-offset ordering priority.
    const fn priority(&self) -> u8 {
        match self {
            Self::Parameter(_) => 0,
            Self::Event(_) => 1,
        }
    }
}

/// One event with its clamped sample offset inside the current block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScheduledBlockEvent<P, E> {
    /// Clamped offset in `0..=frame_count`.
    sample_offset: usize,
    /// Stable source-order sequence number.
    sequence: u64,
    /// Parameter or non-parameter payload.
    event: BlockEvent<P, E>,
}

impl<P, E> ScheduledBlockEvent<P, E> {
    /// Return the clamped sample offset.
    pub const fn sample_offset(&self) -> usize {
        self.sample_offset
    }

    /// Return the parameter or non-parameter payload.
    pub const fn event(&self) -> &BlockEvent<P, E> {
        &self.event
    }

    /// Return the stable source-order sequence number.
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Return the total deterministic ordering key.
    const fn ordering_key(&self) -> (usize, u8, u64) {
        (self.sample_offset, self.event.priority(), self.sequence)
    }
}

/// Result of adding an event to a bounded timeline.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimelinePushStatus {
    /// The event fit in unused preallocated storage.
    Stored,
    /// The event was earlier than the latest retained event and replaced it.
    ReplacedLaterEvent,
    /// The event was later than every retained event and was dropped.
    Dropped,
}

impl TimelinePushStatus {
    /// Return whether preallocated capacity was exhausted for this push.
    pub const fn overflowed(self) -> bool {
        !matches!(self, Self::Stored)
    }
}

/// Aggregate result from one host-format ingestion pass.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TimelineIngestReport {
    /// Events stored without exhausting capacity.
    pub stored: usize,
    /// Earlier events that replaced later retained events after capacity filled.
    pub replaced: usize,
    /// Events discarded because they were later than the retained timeline.
    pub dropped: usize,
}

impl TimelineIngestReport {
    /// Record one bounded push result.
    pub fn record(&mut self, status: TimelinePushStatus) {
        match status {
            TimelinePushStatus::Stored => self.stored = self.stored.saturating_add(1),
            TimelinePushStatus::ReplacedLaterEvent => {
                self.replaced = self.replaced.saturating_add(1);
            }
            TimelinePushStatus::Dropped => self.dropped = self.dropped.saturating_add(1),
        }
    }

    /// Return the number of pushes that encountered full storage.
    pub const fn overflowed(&self) -> usize {
        self.replaced.saturating_add(self.dropped)
    }
}

/// A chronological group of events sharing one sample offset.
pub struct TimelineBatch<'a, P, E> {
    /// Shared sample offset for this group.
    sample_offset: usize,
    /// Parameter-first, source-stable events at this offset.
    events: &'a [ScheduledBlockEvent<P, E>],
}

impl<'a, P, E> TimelineBatch<'a, P, E> {
    /// Return the sample boundary at which this batch takes effect.
    pub const fn sample_offset(&self) -> usize {
        self.sample_offset
    }

    /// Return events in deterministic application order.
    pub const fn events(&self) -> &'a [ScheduledBlockEvent<P, E>] {
        self.events
    }
}

/// Reusable, fixed-capacity storage for one process block's event timeline.
///
/// `P` and `E` must be `Copy` so resetting the timeline cannot run arbitrary
/// destructors on the audio thread. Construction may allocate; [`begin_block`](Self::begin_block),
/// pushes, [`prepare`](Self::prepare), and batch iteration do not allocate.
pub struct BlockEventTimeline<P, E> {
    /// Preallocated event storage.
    events: Vec<ScheduledBlockEvent<P, E>>,
    /// Hard event bound, independent of allocator over-reservation.
    max_events: usize,
    /// Current audio-frame count and valid end boundary.
    frame_count: usize,
    /// Stable source sequence assigned before chronological sorting.
    next_sequence: u64,
    /// Next sorted event returned by [`next_batch`](Self::next_batch).
    next_event: usize,
    /// Whether the current events have been ordered for iteration.
    prepared: bool,
}

impl<P: Copy, E: Copy> BlockEventTimeline<P, E> {
    /// Allocate reusable storage for at most `capacity` events.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            events: Vec::with_capacity(capacity),
            max_events: capacity,
            frame_count: 0,
            next_sequence: 0,
            next_event: 0,
            prepared: false,
        }
    }

    /// Try to allocate reusable event storage without panicking on reservation failure.
    pub fn try_with_capacity(capacity: usize) -> Result<Self, TryReserveError> {
        let mut events = Vec::new();
        events.try_reserve_exact(capacity)?;
        Ok(Self {
            events,
            max_events: capacity,
            frame_count: 0,
            next_sequence: 0,
            next_event: 0,
            prepared: false,
        })
    }

    /// Reset retained state and set the valid end boundary for a new block.
    pub fn begin_block(&mut self, frame_count: usize) {
        self.events.clear();
        self.frame_count = frame_count;
        self.next_sequence = 0;
        self.next_event = 0;
        self.prepared = false;
    }

    /// Add a parameter point at a signed host offset.
    pub fn push_parameter(&mut self, sample_offset: i64, parameter: P) -> TimelinePushStatus {
        self.push(sample_offset, BlockEvent::Parameter(parameter))
    }

    /// Add a non-parameter event at a signed host offset.
    pub fn push_event(&mut self, sample_offset: i64, event: E) -> TimelinePushStatus {
        self.push(sample_offset, BlockEvent::Event(event))
    }

    /// Add a classified event at a signed host offset.
    pub fn push(&mut self, sample_offset: i64, event: BlockEvent<P, E>) -> TimelinePushStatus {
        let scheduled = ScheduledBlockEvent {
            sample_offset: clamp_sample_offset(sample_offset, self.frame_count),
            sequence: self.next_sequence,
            event,
        };
        self.next_sequence = self.next_sequence.saturating_add(1);
        self.prepared = false;

        if self.events.len() < self.max_events {
            self.events.push(scheduled);
            return TimelinePushStatus::Stored;
        }

        let Some((latest_index, latest)) = self
            .events
            .iter()
            .enumerate()
            .max_by_key(|(_, retained)| retained.ordering_key())
        else {
            return TimelinePushStatus::Dropped;
        };

        if scheduled.ordering_key() < latest.ordering_key() {
            self.events[latest_index] = scheduled;
            TimelinePushStatus::ReplacedLaterEvent
        } else {
            TimelinePushStatus::Dropped
        }
    }

    /// Sort chronologically and reset iteration to the first event.
    pub fn prepare(&mut self) {
        self.events
            .sort_unstable_by_key(ScheduledBlockEvent::ordering_key);
        self.next_event = 0;
        self.prepared = true;
    }

    /// Return the next same-offset batch, or `None` before preparation or at the end.
    pub fn next_batch(&mut self) -> Option<TimelineBatch<'_, P, E>> {
        if !self.prepared || self.next_event >= self.events.len() {
            return None;
        }
        let start = self.next_event;
        let sample_offset = self.events[start].sample_offset;
        let mut end = start + 1;
        while end < self.events.len() && self.events[end].sample_offset == sample_offset {
            end += 1;
        }
        self.next_event = end;
        Some(TimelineBatch {
            sample_offset,
            events: &self.events[start..end],
        })
    }

    /// Return all retained events in their current order.
    ///
    /// Call [`prepare`](Self::prepare) first when chronological order is required.
    pub fn events(&self) -> &[ScheduledBlockEvent<P, E>] {
        &self.events
    }

    /// Return the hard maximum number of retained events.
    pub const fn capacity(&self) -> usize {
        self.max_events
    }

    /// Return the current block's audio-frame count.
    pub const fn frame_count(&self) -> usize {
        self.frame_count
    }
}

/// Clamp a signed host sample offset to the inclusive block boundary.
fn clamp_sample_offset(sample_offset: i64, frame_count: usize) -> usize {
    usize::try_from(sample_offset)
        .unwrap_or_default()
        .min(frame_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_alloc::assert_realtime_safe;

    #[test]
    fn timeline_orders_parameters_before_events_and_keeps_source_order() {
        let mut timeline = BlockEventTimeline::with_capacity(8);
        timeline.begin_block(16);
        timeline.push_event(8, 'a');
        timeline.push_parameter(8, 1_u32);
        timeline.push_parameter(8, 2_u32);
        timeline.push_event(8, 'b');
        timeline.prepare();

        let batch = timeline.next_batch().expect("same-offset batch");
        assert_eq!(batch.sample_offset(), 8);
        assert_eq!(
            batch
                .events()
                .iter()
                .map(ScheduledBlockEvent::event)
                .copied()
                .collect::<Vec<_>>(),
            vec![
                BlockEvent::Parameter(1),
                BlockEvent::Parameter(2),
                BlockEvent::Event('a'),
                BlockEvent::Event('b'),
            ]
        );
    }

    #[test]
    fn timeline_clamps_offsets_to_inclusive_block_boundaries() {
        let mut timeline = BlockEventTimeline::<u32, u32>::with_capacity(3);
        timeline.begin_block(16);
        timeline.push_parameter(-4, 1);
        timeline.push_parameter(8, 2);
        timeline.push_parameter(99, 3);
        timeline.prepare();

        assert_eq!(
            timeline
                .events()
                .iter()
                .map(ScheduledBlockEvent::sample_offset)
                .collect::<Vec<_>>(),
            vec![0, 8, 16]
        );
    }

    #[test]
    fn overflow_retains_earliest_events_independent_of_ingestion_order() {
        let mut timeline = BlockEventTimeline::<u32, char>::with_capacity(2);
        timeline.begin_block(16);
        assert_eq!(timeline.push_event(12, 'z'), TimelinePushStatus::Stored);
        assert_eq!(timeline.push_event(8, 'y'), TimelinePushStatus::Stored);
        assert_eq!(
            timeline.push_parameter(8, 1),
            TimelinePushStatus::ReplacedLaterEvent
        );
        assert_eq!(timeline.push_parameter(14, 2), TimelinePushStatus::Dropped);
        timeline.prepare();

        assert_eq!(
            timeline
                .events()
                .iter()
                .map(|event| (event.sample_offset(), *event.event()))
                .collect::<Vec<_>>(),
            vec![(8, BlockEvent::Parameter(1)), (8, BlockEvent::Event('y'))]
        );
    }

    #[test]
    fn zero_capacity_and_end_boundary_are_safe_and_explicit() {
        let mut empty = BlockEventTimeline::<u32, u32>::with_capacity(0);
        empty.begin_block(1);
        assert_eq!(empty.push_parameter(0, 1), TimelinePushStatus::Dropped);
        empty.prepare();
        assert!(empty.next_batch().is_none());

        let mut timeline = BlockEventTimeline::<u32, u32>::with_capacity(1);
        timeline.begin_block(1);
        timeline.push_parameter(99, 7);
        timeline.prepare();
        let batch = timeline.next_batch().expect("end-boundary state batch");
        assert_eq!(batch.sample_offset(), 1);
        assert_eq!(batch.events()[0].event(), &BlockEvent::Parameter(7));
    }

    #[test]
    fn reset_push_prepare_and_iteration_do_not_touch_allocator() {
        let mut timeline = BlockEventTimeline::<u32, u32>::with_capacity(32);
        assert_realtime_safe(|| {
            timeline.begin_block(64);
            for offset in (0..32).rev() {
                timeline.push_parameter(offset, offset as u32);
            }
            timeline.prepare();
            while let Some(batch) = timeline.next_batch() {
                std::hint::black_box(batch.events());
            }
            timeline.begin_block(64);
        });
        assert_eq!(timeline.capacity(), 32);
    }
}
