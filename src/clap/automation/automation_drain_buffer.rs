//! Per-thread scratch storage for draining automation queues.

use clack_plugin::events::io::OutputEvents;

use super::{AutomationDrainStats, AutomationEvent, AutomationQueue};

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
    /// Returns detailed drain statistics so callers can track lock contention
    /// and host push failures.
    pub fn drain(
        &mut self,
        queue: &AutomationQueue,
        output: &mut OutputEvents<'_>,
    ) -> AutomationDrainStats {
        queue.drain_to_output(output, &mut self.scratch)
    }
}
