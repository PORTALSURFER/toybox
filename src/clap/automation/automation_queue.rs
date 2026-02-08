//! Queue and drain logic for GUI automation events.

use std::collections::VecDeque;
use std::sync::Mutex;

use clack_plugin::events::Pckn;
use clack_plugin::events::io::OutputEvents;
use clack_plugin::utils::{ClapId, Cookie};

use crate::clap::params::{push_param_gesture_begin, push_param_gesture_end, push_param_value};

use super::{
    AutomationConfig, AutomationDrainStats, AutomationDropPolicy, AutomationEnqueueStatus,
    AutomationEvent, AutomationQueueConfig,
};

/// Thread-safe queue for GUI-originated automation events.
///
/// The queue is bounded and enforces an overflow policy from
/// [`AutomationQueueConfig`].
pub struct AutomationQueue {
    /// Pending automation events in enqueue order.
    events: Mutex<VecDeque<AutomationEvent>>,
    /// Immutable queue sizing and overflow policy.
    config: AutomationQueueConfig,
}

impl AutomationQueue {
    /// Create an automation queue with the supplied bounded queue config.
    pub fn with_config(config: AutomationQueueConfig) -> Self {
        Self {
            events: Mutex::new(VecDeque::new()),
            config,
        }
    }

    /// Return the queue configuration.
    pub fn config(&self) -> AutomationQueueConfig {
        self.config
    }

    /// Try to enqueue an automation event according to queue policy.
    fn try_enqueue(&self, event: AutomationEvent) -> AutomationEnqueueStatus {
        let Ok(mut events) = self.events.lock() else {
            return AutomationEnqueueStatus::QueuePoisoned;
        };

        if self.config.max_events == 0 {
            return AutomationEnqueueStatus::QueueFull;
        }
        if events.len() >= self.config.max_events
            && matches!(self.config.drop_policy, AutomationDropPolicy::DropNewest)
        {
            return AutomationEnqueueStatus::QueueFull;
        }
        if events.len() >= self.config.max_events {
            let _ = events.pop_front();
        }

        events.push_back(event);
        AutomationEnqueueStatus::Enqueued
    }

    /// Try to enqueue a parameter value update and return the enqueue status.
    pub fn try_push_value(
        &self,
        config: &AutomationConfig,
        param_id: ClapId,
        value: f64,
    ) -> AutomationEnqueueStatus {
        if !config.is_enabled(param_id) {
            return AutomationEnqueueStatus::Disabled;
        }
        self.try_enqueue(AutomationEvent::Value(param_id, value))
    }

    /// Enqueue a parameter value update if automation is enabled.
    pub fn push_value(&self, config: &AutomationConfig, param_id: ClapId, value: f64) {
        let _ = self.try_push_value(config, param_id, value);
    }

    /// Try to enqueue a gesture begin event and return the enqueue status.
    pub fn try_push_gesture_begin(
        &self,
        config: &AutomationConfig,
        param_id: ClapId,
    ) -> AutomationEnqueueStatus {
        if !config.is_enabled(param_id) {
            return AutomationEnqueueStatus::Disabled;
        }
        self.try_enqueue(AutomationEvent::GestureBegin(param_id))
    }

    /// Enqueue a gesture begin event if automation is enabled.
    pub fn push_gesture_begin(&self, config: &AutomationConfig, param_id: ClapId) {
        let _ = self.try_push_gesture_begin(config, param_id);
    }

    /// Try to enqueue a gesture end event and return the enqueue status.
    pub fn try_push_gesture_end(
        &self,
        config: &AutomationConfig,
        param_id: ClapId,
    ) -> AutomationEnqueueStatus {
        if !config.is_enabled(param_id) {
            return AutomationEnqueueStatus::Disabled;
        }
        self.try_enqueue(AutomationEvent::GestureEnd(param_id))
    }

    /// Enqueue a gesture end event if automation is enabled.
    pub fn push_gesture_end(&self, config: &AutomationConfig, param_id: ClapId) {
        let _ = self.try_push_gesture_end(config, param_id);
    }

    /// Drain queued automation events into an output buffer.
    ///
    /// The caller supplies a scratch buffer to avoid allocations in realtime
    /// threads. Returns `true` if any events were drained. If the queue is
    /// temporarily locked by another thread, no events are drained and the
    /// caller should try again on the next cycle.
    pub fn drain_to_output(
        &self,
        output: &mut OutputEvents<'_>,
        scratch: &mut Vec<AutomationEvent>,
    ) -> bool {
        let stats = self.drain_to_output_with_stats(output, scratch);
        stats.attempted > 0
    }

    /// Drain queued automation events into an output buffer with stats.
    ///
    /// The caller supplies a scratch buffer to avoid allocations in realtime
    /// threads. Events that fail to push are dropped and counted in the stats.
    /// If the queue is temporarily locked by another thread, `locked` is set
    /// and no events are drained.
    pub fn drain_to_output_with_stats(
        &self,
        output: &mut OutputEvents<'_>,
        scratch: &mut Vec<AutomationEvent>,
    ) -> AutomationDrainStats {
        let Ok(mut events) = self.events.try_lock() else {
            return AutomationDrainStats {
                locked: true,
                ..AutomationDrainStats::default()
            };
        };
        if events.is_empty() {
            return AutomationDrainStats::default();
        }

        scratch.clear();
        scratch.extend(events.drain(..));
        drop(events);

        drain_scratch_to_output(scratch, output)
    }
}

fn drain_scratch_to_output(
    scratch: &mut Vec<AutomationEvent>,
    output: &mut OutputEvents<'_>,
) -> AutomationDrainStats {
    let mut stats = AutomationDrainStats {
        attempted: scratch.len(),
        ..AutomationDrainStats::default()
    };

    for event in scratch.drain(..) {
        if push_automation_event(output, event) {
            stats.pushed += 1;
        } else {
            stats.failed += 1;
        }
    }

    stats
}

fn push_automation_event(output: &mut OutputEvents<'_>, event: AutomationEvent) -> bool {
    match event {
        AutomationEvent::GestureBegin(param_id) => {
            push_param_gesture_begin(output, 0, param_id).is_ok()
        }
        AutomationEvent::GestureEnd(param_id) => {
            push_param_gesture_end(output, 0, param_id).is_ok()
        }
        AutomationEvent::Value(param_id, value) => push_param_value(
            output,
            0,
            param_id,
            value,
            Pckn::match_all(),
            Cookie::empty(),
        )
        .is_ok(),
    }
}

impl Default for AutomationQueue {
    fn default() -> Self {
        Self::with_config(AutomationQueueConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use clack_plugin::events::io::EventBuffer;
    use clack_plugin::events::spaces::CoreEventSpace;

    use super::*;

    #[test]
    fn config_respects_overrides() {
        let mut config = AutomationConfig::new(true);
        let param_a = ClapId::new(1);
        let param_b = ClapId::new(2);
        assert!(config.is_enabled(param_a));
        config.disable_param(param_a);
        assert!(!config.is_enabled(param_a));
        config.enable_param(param_b);
        assert!(config.is_enabled(param_b));
    }

    #[test]
    fn queue_drains_to_output() {
        let queue = AutomationQueue::default();
        let config = AutomationConfig::default();
        let param_id = ClapId::new(5);
        queue.push_gesture_begin(&config, param_id);
        queue.push_value(&config, param_id, 0.5);
        queue.push_gesture_end(&config, param_id);

        let mut buffer = EventBuffer::new();
        let mut output = buffer.as_output();
        let mut scratch = Vec::new();

        let stats = queue.drain_to_output_with_stats(&mut output, &mut scratch);
        assert_eq!(stats.attempted, 3);
        assert_eq!(stats.pushed, 3);
        assert_eq!(stats.failed, 0);
    }

    #[test]
    fn drain_buffer_drains_without_external_scratch() {
        let queue = AutomationQueue::default();
        let config = AutomationConfig::default();
        let param_id = ClapId::new(11);
        queue.push_value(&config, param_id, 0.42);

        let mut output_buffer = EventBuffer::new();
        let mut output = output_buffer.as_output();
        let mut drain_buffer = crate::clap::automation::AutomationDrainBuffer::default();

        let drained = drain_buffer.drain(&queue, &mut output);
        assert!(drained);
    }

    #[test]
    fn try_push_respects_disabled_params() {
        let queue = AutomationQueue::default();
        let mut config = AutomationConfig::default();
        let param_id = ClapId::new(9);
        config.disable_param(param_id);

        let status = queue.try_push_value(&config, param_id, 0.7);
        assert_eq!(status, AutomationEnqueueStatus::Disabled);

        let mut output_buffer = EventBuffer::new();
        let mut output = output_buffer.as_output();
        let mut scratch = Vec::new();
        let stats = queue.drain_to_output_with_stats(&mut output, &mut scratch);
        assert_eq!(stats.attempted, 0);
    }

    #[test]
    fn try_push_reports_poisoned_queue() {
        let queue = AutomationQueue::default();
        let config = AutomationConfig::default();
        let param_id = ClapId::new(3);

        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = queue.events.lock().expect("lock should succeed");
            panic!("poison queue");
        }));

        let status = queue.try_push_value(&config, param_id, 0.25);
        assert_eq!(status, AutomationEnqueueStatus::QueuePoisoned);
    }

    #[test]
    fn queue_full_drop_newest_rejects_extra_events() {
        let queue = AutomationQueue::with_config(AutomationQueueConfig::new(
            1,
            AutomationDropPolicy::DropNewest,
        ));
        let config = AutomationConfig::default();
        let param_id = ClapId::new(13);

        assert_eq!(
            queue.try_push_value(&config, param_id, 0.1),
            AutomationEnqueueStatus::Enqueued
        );
        assert_eq!(
            queue.try_push_value(&config, param_id, 0.2),
            AutomationEnqueueStatus::QueueFull
        );

        let mut output_buffer = EventBuffer::new();
        let mut output = output_buffer.as_output();
        let mut scratch = Vec::new();
        let stats = queue.drain_to_output_with_stats(&mut output, &mut scratch);
        assert_eq!(stats.attempted, 1);
        assert_eq!(stats.pushed, 1);
        assert_eq!(output_buffer.len(), 1);
    }

    #[test]
    fn queue_full_drop_oldest_keeps_newest_events() {
        let queue = AutomationQueue::with_config(AutomationQueueConfig::new(
            2,
            AutomationDropPolicy::DropOldest,
        ));
        let config = AutomationConfig::default();
        let first = ClapId::new(21);
        let second = ClapId::new(22);
        let third = ClapId::new(23);

        assert_eq!(
            queue.try_push_value(&config, first, 0.1),
            AutomationEnqueueStatus::Enqueued
        );
        assert_eq!(
            queue.try_push_value(&config, second, 0.2),
            AutomationEnqueueStatus::Enqueued
        );
        assert_eq!(
            queue.try_push_value(&config, third, 0.3),
            AutomationEnqueueStatus::Enqueued
        );

        let mut output_buffer = EventBuffer::new();
        let mut output = output_buffer.as_output();
        let mut scratch = Vec::new();
        let stats = queue.drain_to_output_with_stats(&mut output, &mut scratch);
        assert_eq!(stats.attempted, 2);
        assert_eq!(stats.pushed, 2);

        let mut observed_ids = Vec::new();
        for index in 0..output_buffer.len() {
            let event = output_buffer.get(index as u32).expect("event should exist");
            if let Some(CoreEventSpace::ParamValue(value)) = event.as_core_event()
                && let Some(param_id) = value.param_id()
            {
                observed_ids.push(param_id);
            }
        }
        assert_eq!(observed_ids, vec![second, third]);
    }
}
