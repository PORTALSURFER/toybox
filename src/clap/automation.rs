//! Automation helpers for routing GUI parameter edits to the host.
//!
//! The automation queue is bounded by default
//! ([`DEFAULT_AUTOMATION_QUEUE_MAX_EVENTS`]) to avoid unbounded memory growth
//! when GUI producers outpace host event drains.
//!
//! # Example
//! ```
//! use toybox::clack_plugin::utils::ClapId;
//! use toybox::clap::automation::{AutomationConfig, AutomationDrainBuffer, AutomationQueue};
//! use toybox::clack_plugin::events::io::EventBuffer;
//!
//! let mut config = AutomationConfig::default();
//! config.disable_param(ClapId::new(2));
//!
//! let queue = AutomationQueue::default();
//! queue.push_gesture_begin(&config, ClapId::new(1));
//! queue.push_value(&config, ClapId::new(1), 0.5);
//! queue.push_gesture_end(&config, ClapId::new(1));
//!
//! let mut buffer = EventBuffer::new();
//! let mut output = buffer.as_output();
//! let mut drain_buffer = AutomationDrainBuffer::default();
//! drain_buffer.drain(&queue, &mut output);
//! ```

use std::collections::{HashSet, VecDeque};
use std::sync::Mutex;

use clack_plugin::events::Pckn;
use clack_plugin::events::io::OutputEvents;
use clack_plugin::utils::{ClapId, Cookie};

use crate::clap::params::{push_param_gesture_begin, push_param_gesture_end, push_param_value};

/// GUI-originated automation event.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AutomationEvent {
    /// Begin a parameter gesture for automation recording.
    GestureBegin(ClapId),
    /// End a parameter gesture for automation recording.
    GestureEnd(ClapId),
    /// Send a parameter value update.
    Value(ClapId, f64),
}

/// Status returned when enqueueing automation events.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutomationEnqueueStatus {
    /// The event was accepted by the queue.
    Enqueued,
    /// The queue reached its configured capacity and dropped/rejected the event.
    QueueFull,
    /// The parameter was disabled in the automation config.
    Disabled,
    /// The queue mutex was poisoned and the event could not be enqueued.
    QueuePoisoned,
}

/// Policy for handling automation events when the queue reaches capacity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutomationDropPolicy {
    /// Reject the newly enqueued event and keep older queued events.
    DropNewest,
    /// Drop the oldest queued event and keep the newly enqueued event.
    DropOldest,
}

/// Queue configuration used to bound automation memory growth.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AutomationQueueConfig {
    /// Maximum number of queued automation events.
    ///
    /// When this value is `0`, all incoming events are treated as overflow.
    pub max_events: usize,
    /// Overflow strategy used when `max_events` has been reached.
    pub drop_policy: AutomationDropPolicy,
}

impl AutomationQueueConfig {
    /// Build a new queue config with explicit capacity and overflow policy.
    pub fn new(max_events: usize, drop_policy: AutomationDropPolicy) -> Self {
        Self {
            max_events,
            drop_policy,
        }
    }
}

impl Default for AutomationQueueConfig {
    fn default() -> Self {
        Self {
            max_events: DEFAULT_AUTOMATION_QUEUE_MAX_EVENTS,
            drop_policy: AutomationDropPolicy::DropNewest,
        }
    }
}

/// Default maximum number of queued automation events.
pub const DEFAULT_AUTOMATION_QUEUE_MAX_EVENTS: usize = 4096;

/// Configuration for which parameters should emit automation events.
#[derive(Clone, Debug)]
pub struct AutomationConfig {
    /// Fallback enable state used when a parameter has no explicit override.
    default_enabled: bool,
    /// Parameter ids that are always automation-enabled.
    enabled: HashSet<ClapId>,
    /// Parameter ids that are always automation-disabled.
    disabled: HashSet<ClapId>,
}

impl AutomationConfig {
    /// Create a new automation config with a default enabled/disabled state.
    pub fn new(default_enabled: bool) -> Self {
        Self {
            default_enabled,
            enabled: HashSet::new(),
            disabled: HashSet::new(),
        }
    }

    /// Return true if the parameter should emit automation events.
    pub fn is_enabled(&self, param_id: ClapId) -> bool {
        if self.enabled.contains(&param_id) {
            return true;
        }
        if self.disabled.contains(&param_id) {
            return false;
        }
        self.default_enabled
    }

    /// Enable automation for a specific parameter.
    pub fn enable_param(&mut self, param_id: ClapId) {
        self.disabled.remove(&param_id);
        self.enabled.insert(param_id);
    }

    /// Disable automation for a specific parameter.
    pub fn disable_param(&mut self, param_id: ClapId) {
        self.enabled.remove(&param_id);
        self.disabled.insert(param_id);
    }
}

impl Default for AutomationConfig {
    fn default() -> Self {
        Self::new(true)
    }
}

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

/// Summary information from draining automation events.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AutomationDrainStats {
    /// Total events removed from the queue.
    pub attempted: usize,
    /// Events successfully pushed to the output buffer.
    pub pushed: usize,
    /// Events that failed to push to the output buffer.
    pub failed: usize,
    /// Whether the queue lock was unavailable.
    pub locked: bool,
}

/// Per-thread scratch storage for draining automation to host output.
///
/// Plugins should keep one instance in each thread object that emits host
/// parameter events (for example main-thread and audio-thread parameter flush
/// implementations). Reusing this scratch buffer avoids allocating while
/// draining queued GUI automation events.
#[derive(Default)]
pub struct AutomationDrainBuffer {
    /// Reusable scratch storage used to drain queued events without reallocating.
    scratch: Vec<AutomationEvent>,
}

impl AutomationDrainBuffer {
    /// Drain queued automation events into an output buffer.
    ///
    /// Returns `true` when at least one queued event was consumed.
    pub fn drain(&mut self, queue: &AutomationQueue, output: &mut OutputEvents<'_>) -> bool {
        queue.drain_to_output(output, &mut self.scratch)
    }

    /// Drain queued automation events into an output buffer with stats.
    pub fn drain_with_stats(
        &mut self,
        queue: &AutomationQueue,
        output: &mut OutputEvents<'_>,
    ) -> AutomationDrainStats {
        queue.drain_to_output_with_stats(output, &mut self.scratch)
    }
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

        if events.len() >= self.config.max_events {
            match self.config.drop_policy {
                AutomationDropPolicy::DropNewest => {
                    return AutomationEnqueueStatus::QueueFull;
                }
                AutomationDropPolicy::DropOldest => {
                    let _ = events.pop_front();
                }
            }
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

        let mut stats = AutomationDrainStats {
            attempted: scratch.len(),
            ..AutomationDrainStats::default()
        };

        for event in scratch.drain(..) {
            let pushed = match event {
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
            };
            if pushed {
                stats.pushed += 1;
            } else {
                stats.failed += 1;
            }
        }

        stats
    }
}

impl Default for AutomationQueue {
    fn default() -> Self {
        Self::with_config(AutomationQueueConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clack_plugin::events::io::EventBuffer;
    use clack_plugin::events::spaces::CoreEventSpace;

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
        let mut drain_buffer = AutomationDrainBuffer::default();

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
