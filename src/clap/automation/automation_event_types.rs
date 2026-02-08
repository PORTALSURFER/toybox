//! Event and queue configuration types for GUI automation.

use clack_plugin::utils::ClapId;

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
