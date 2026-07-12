//! Realtime-safe VST3 parameter and input-event timeline ingestion.

use std::mem::MaybeUninit;

use toybox_vst3_ffi::ComRef;
use toybox_vst3_ffi::Steinberg::Vst::{
    Event, IEventList, IEventListTrait, IParameterChanges, ParamID, ParamValue,
};
use toybox_vst3_ffi::Steinberg::kResultOk;

use crate::events::{BlockEventTimeline, TimelineIngestReport};
use crate::vst3::params::for_each_param_point;

/// One normalized VST3 parameter point, excluding its scheduled sample offset.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vst3ParameterPoint {
    /// Host-visible VST3 parameter identifier.
    pub param_id: ParamID,
    /// Normalized parameter value in VST3's `[0.0, 1.0]` space.
    pub normalized_value: ParamValue,
}

/// Preallocated timeline type used by the standard VST3 ingestion adapter.
pub type Vst3EventTimeline = BlockEventTimeline<Vst3ParameterPoint, Event>;

/// Collect VST3 parameter queues and input events into one chronological timeline.
///
/// Parameter queues are read in host queue/point order and input events are read
/// in list order. After chronological sorting, parameter points precede input
/// events at equal offsets, so a point on a note boundary affects that note.
/// Invalid queue entries are skipped. Signed offsets are clamped to the
/// inclusive `0..=frame_count` boundary by [`BlockEventTimeline`].
///
/// The timeline must have been allocated before entering the audio callback.
/// This function resets, fills, and prepares it without growing its storage.
///
/// # Safety
///
/// Non-null `parameter_changes` and `input_events` pointers must remain valid
/// for the duration of this synchronous call.
pub unsafe fn collect_vst3_timeline(
    parameter_changes: *mut IParameterChanges,
    input_events: *mut IEventList,
    frame_count: usize,
    timeline: &mut Vst3EventTimeline,
) -> TimelineIngestReport {
    timeline.begin_block(frame_count);
    let mut report = TimelineIngestReport::default();

    unsafe {
        for_each_param_point(
            parameter_changes,
            |param_id, sample_offset, normalized_value| {
                report.record(timeline.push_parameter(
                    i64::from(sample_offset),
                    Vst3ParameterPoint {
                        param_id,
                        normalized_value,
                    },
                ));
            },
        );
    }

    if let Some(events) = unsafe { ComRef::from_raw(input_events) } {
        let count = unsafe { events.getEventCount() }.max(0);
        for index in 0..count {
            let mut event = MaybeUninit::<Event>::uninit();
            if unsafe { events.getEvent(index, event.as_mut_ptr()) } != kResultOk {
                continue;
            }
            let event = unsafe { event.assume_init() };
            report.record(timeline.push_event(i64::from(event.sampleOffset), event));
        }
    }

    timeline.prepare();
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use clack_plugin::events::Pckn;
    use clack_plugin::events::event_types::{NoteOnEvent as ClapNoteOnEvent, ParamValueEvent};
    use clack_plugin::events::io::InputEvents;
    use clack_plugin::events::spaces::CoreEventSpace;
    use clack_plugin::utils::{ClapId, Cookie};
    use toybox_vst3_ffi::Steinberg::Vst::{
        Event_, Event__type0, IEventListTrait, IParamValueQueue, IParamValueQueueTrait,
        IParameterChangesTrait, NoteOnEvent,
    };
    use toybox_vst3_ffi::Steinberg::{kInvalidArgument, kNotImplemented, kResultFalse};
    use toybox_vst3_ffi::{Class, ComPtr, ComWrapper};

    use crate::clap::events::collect_clap_timeline;
    use crate::events::{BlockEvent, ScheduledBlockEvent};
    use crate::test_alloc::assert_realtime_safe;

    /// Read-only parameter queue used by host-ingestion tests.
    struct TestParamQueue {
        /// Parameter identifier returned to the plugin.
        param_id: ParamID,
        /// Host-provided `(offset, normalized value)` points.
        points: Vec<(i32, ParamValue)>,
    }

    impl Class for TestParamQueue {
        type Interfaces = (IParamValueQueue,);
    }

    impl IParamValueQueueTrait for TestParamQueue {
        unsafe fn getParameterId(&self) -> ParamID {
            self.param_id
        }

        unsafe fn getPointCount(&self) -> i32 {
            i32::try_from(self.points.len()).unwrap_or(i32::MAX)
        }

        unsafe fn getPoint(
            &self,
            index: i32,
            sample_offset: *mut i32,
            value: *mut ParamValue,
        ) -> i32 {
            if sample_offset.is_null() || value.is_null() {
                return kInvalidArgument;
            }
            let Some((point_offset, point_value)) = usize::try_from(index)
                .ok()
                .and_then(|index| self.points.get(index))
                .copied()
            else {
                return kResultFalse;
            };
            unsafe {
                *sample_offset = point_offset;
                *value = point_value;
            }
            kResultOk
        }

        unsafe fn addPoint(
            &self,
            _sample_offset: i32,
            _value: ParamValue,
            _index: *mut i32,
        ) -> i32 {
            kNotImplemented
        }
    }

    /// Read-only parameter change collection used by host-ingestion tests.
    struct TestParameterChanges {
        /// Retained COM parameter queues.
        queues: Vec<ComPtr<IParamValueQueue>>,
        /// Optional queue index that simulates a null host entry.
        null_queue_index: Option<usize>,
    }

    impl Class for TestParameterChanges {
        type Interfaces = (IParameterChanges,);
    }

    impl IParameterChangesTrait for TestParameterChanges {
        unsafe fn getParameterCount(&self) -> i32 {
            i32::try_from(self.queues.len()).unwrap_or(i32::MAX)
        }

        unsafe fn getParameterData(&self, index: i32) -> *mut IParamValueQueue {
            let Some(index) = usize::try_from(index).ok() else {
                return std::ptr::null_mut();
            };
            if self.null_queue_index == Some(index) {
                return std::ptr::null_mut();
            }
            self.queues
                .get(index)
                .map_or(std::ptr::null_mut(), ComPtr::as_ptr)
        }

        unsafe fn addParameterData(
            &self,
            _id: *const ParamID,
            _index: *mut i32,
        ) -> *mut IParamValueQueue {
            std::ptr::null_mut()
        }
    }

    /// Read-only input event list used by host-ingestion tests.
    struct TestEventList {
        /// Host-provided VST3 input events.
        events: Vec<Event>,
    }

    impl Class for TestEventList {
        type Interfaces = (IEventList,);
    }

    impl IEventListTrait for TestEventList {
        unsafe fn getEventCount(&self) -> i32 {
            i32::try_from(self.events.len()).unwrap_or(i32::MAX)
        }

        unsafe fn getEvent(&self, index: i32, event: *mut Event) -> i32 {
            if event.is_null() {
                return kInvalidArgument;
            }
            let Some(value) = usize::try_from(index)
                .ok()
                .and_then(|index| self.events.get(index))
                .copied()
            else {
                return kResultFalse;
            };
            unsafe { *event = value };
            kResultOk
        }

        unsafe fn addEvent(&self, _event: *mut Event) -> i32 {
            kNotImplemented
        }
    }

    /// Build retained fake host parameter changes.
    fn parameter_changes(
        queues: impl IntoIterator<Item = (ParamID, Vec<(i32, ParamValue)>)>,
    ) -> ComWrapper<TestParameterChanges> {
        let queues = queues
            .into_iter()
            .map(|(param_id, points)| {
                ComWrapper::new(TestParamQueue { param_id, points })
                    .to_com_ptr::<IParamValueQueue>()
                    .expect("test queue interface")
            })
            .collect();
        ComWrapper::new(TestParameterChanges {
            queues,
            null_queue_index: None,
        })
    }

    /// Build one VST3 note-on event at a host sample offset.
    fn note_on(sample_offset: i32, pitch: i16) -> Event {
        Event {
            busIndex: 0,
            sampleOffset: sample_offset,
            ppqPosition: 0.0,
            flags: 0,
            r#type: Event_::EventTypes_::kNoteOnEvent as u16,
            __field0: Event__type0 {
                noteOn: NoteOnEvent {
                    channel: 0,
                    pitch,
                    tuning: 0.0,
                    velocity: 1.0,
                    length: -1,
                    noteId: -1,
                },
            },
        }
    }

    /// Extract a test-friendly logical event from a VST3 timeline item.
    fn logical_vst3_event(
        event: &ScheduledBlockEvent<Vst3ParameterPoint, Event>,
    ) -> (usize, u32, u32) {
        match event.event() {
            BlockEvent::Parameter(point) => (
                event.sample_offset(),
                point.param_id,
                (point.normalized_value * 100.0) as u32,
            ),
            BlockEvent::Event(event) => {
                let note = unsafe { event.__field0.noteOn };
                (event.sampleOffset as usize, u32::MAX, note.pitch as u32)
            }
        }
    }

    #[test]
    fn vst3_merges_unsorted_queues_and_notes_with_parameter_first_ties() {
        let changes = parameter_changes([
            (2, vec![(12, 0.2), (4, 0.4)]),
            (1, vec![(8, 0.5), (8, 0.6)]),
        ]);
        let events = ComWrapper::new(TestEventList {
            events: vec![note_on(8, 60), note_on(2, 48)],
        });
        let changes_ref = changes
            .as_com_ref::<IParameterChanges>()
            .expect("parameter changes interface");
        let events_ref = events
            .as_com_ref::<IEventList>()
            .expect("event list interface");
        let mut timeline = Vst3EventTimeline::with_capacity(6);

        let report = assert_realtime_safe(|| unsafe {
            collect_vst3_timeline(changes_ref.as_ptr(), events_ref.as_ptr(), 16, &mut timeline)
        });

        assert_eq!(report.overflowed(), 0);
        assert_eq!(
            timeline
                .events()
                .iter()
                .map(logical_vst3_event)
                .collect::<Vec<_>>(),
            vec![
                (2, u32::MAX, 48),
                (4, 2, 40),
                (8, 1, 50),
                (8, 1, 60),
                (8, u32::MAX, 60),
                (12, 2, 20),
            ]
        );
    }

    #[test]
    fn vst3_clamps_offsets_across_required_block_size_matrix() {
        for frame_count in [1_usize, 16, 64, 512, 2_048] {
            let midpoint = i32::try_from(frame_count / 2).expect("test block size");
            let beyond = i32::try_from(frame_count + 100).expect("test block size");
            let changes = parameter_changes([(1, vec![(-2, 0.1), (midpoint, 0.2), (beyond, 0.3)])]);
            let changes_ref = changes
                .as_com_ref::<IParameterChanges>()
                .expect("parameter changes interface");
            let mut timeline = Vst3EventTimeline::with_capacity(3);

            unsafe {
                collect_vst3_timeline(
                    changes_ref.as_ptr(),
                    std::ptr::null_mut(),
                    frame_count,
                    &mut timeline,
                );
            }

            assert_eq!(
                timeline
                    .events()
                    .iter()
                    .map(ScheduledBlockEvent::sample_offset)
                    .collect::<Vec<_>>(),
                vec![0, frame_count / 2, frame_count]
            );
        }
    }

    #[test]
    fn vst3_skips_null_queues_and_reports_deterministic_overflow() {
        let first = ComWrapper::new(TestParamQueue {
            param_id: 1,
            points: vec![(12, 0.1), (14, 0.2)],
        })
        .to_com_ptr::<IParamValueQueue>()
        .expect("first test queue");
        let skipped = ComWrapper::new(TestParamQueue {
            param_id: 2,
            points: vec![(0, 0.9)],
        })
        .to_com_ptr::<IParamValueQueue>()
        .expect("skipped test queue");
        let changes = ComWrapper::new(TestParameterChanges {
            queues: vec![first, skipped],
            null_queue_index: Some(1),
        });
        let events = ComWrapper::new(TestEventList {
            events: vec![note_on(2, 48)],
        });
        let changes_ref = changes
            .as_com_ref::<IParameterChanges>()
            .expect("parameter changes interface");
        let events_ref = events
            .as_com_ref::<IEventList>()
            .expect("event list interface");
        let mut timeline = Vst3EventTimeline::with_capacity(2);

        let report = unsafe {
            collect_vst3_timeline(changes_ref.as_ptr(), events_ref.as_ptr(), 16, &mut timeline)
        };

        assert_eq!(report.stored, 2);
        assert_eq!(report.replaced, 1);
        assert_eq!(report.dropped, 0);
        assert_eq!(
            timeline
                .events()
                .iter()
                .map(logical_vst3_event)
                .collect::<Vec<_>>(),
            vec![(2, u32::MAX, 48), (12, 1, 10)]
        );
    }

    #[test]
    fn clap_and_vst3_emit_the_same_logical_timeline() {
        let clap_before =
            ParamValueEvent::new(2, ClapId::new(7), Pckn::match_all(), 0.25, Cookie::empty());
        let clap_note = ClapNoteOnEvent::new(8, Pckn::match_all(), 1.0);
        let clap_at =
            ParamValueEvent::new(8, ClapId::new(7), Pckn::match_all(), 0.5, Cookie::empty());
        let clap_after =
            ParamValueEvent::new(12, ClapId::new(7), Pckn::match_all(), 0.75, Cookie::empty());
        let clap_source = [
            clap_before.as_ref(),
            clap_note.as_ref(),
            clap_at.as_ref(),
            clap_after.as_ref(),
        ];
        let clap_input = InputEvents::from_buffer(&clap_source);
        let mut clap_timeline = BlockEventTimeline::<(u32, u32), u32>::with_capacity(4);
        collect_clap_timeline(&clap_input, 16, &mut clap_timeline, |event| {
            match event.as_core_event()? {
                CoreEventSpace::ParamValue(param) => Some(BlockEvent::Parameter((
                    param.param_id()?.get(),
                    (param.value() * 100.0) as u32,
                ))),
                CoreEventSpace::NoteOn(_) => Some(BlockEvent::Event(60)),
                _ => None,
            }
        });

        let changes = parameter_changes([(7, vec![(2, 0.25), (8, 0.5), (12, 0.75)])]);
        let vst3_events = ComWrapper::new(TestEventList {
            events: vec![note_on(8, 60)],
        });
        let changes_ref = changes
            .as_com_ref::<IParameterChanges>()
            .expect("parameter changes interface");
        let events_ref = vst3_events
            .as_com_ref::<IEventList>()
            .expect("event list interface");
        let mut vst3_timeline = Vst3EventTimeline::with_capacity(4);
        unsafe {
            collect_vst3_timeline(
                changes_ref.as_ptr(),
                events_ref.as_ptr(),
                16,
                &mut vst3_timeline,
            );
        }

        let clap_logical = clap_timeline
            .events()
            .iter()
            .map(|event| match event.event() {
                BlockEvent::Parameter((id, value)) => (event.sample_offset(), *id, *value),
                BlockEvent::Event(pitch) => (event.sample_offset(), u32::MAX, *pitch),
            })
            .collect::<Vec<_>>();
        let vst3_logical = vst3_timeline
            .events()
            .iter()
            .map(logical_vst3_event)
            .collect::<Vec<_>>();
        assert_eq!(clap_logical, vst3_logical);
    }
}
