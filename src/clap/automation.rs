//! Automation helpers for routing GUI parameter edits to the host.

use std::collections::HashSet;
use std::sync::Mutex;

use clack_plugin::events::io::OutputEvents;
use clack_plugin::events::Pckn;
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

/// Configuration for which parameters should emit automation events.
#[derive(Clone, Debug)]
pub struct AutomationConfig {
    default_enabled: bool,
    enabled: HashSet<ClapId>,
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
#[derive(Default)]
pub struct AutomationQueue {
    events: Mutex<Vec<AutomationEvent>>,
}

impl AutomationQueue {
    /// Enqueue a parameter value update if automation is enabled.
    pub fn push_value(&self, config: &AutomationConfig, param_id: ClapId, value: f64) {
        if !config.is_enabled(param_id) {
            return;
        }
        if let Ok(mut events) = self.events.lock() {
            events.push(AutomationEvent::Value(param_id, value));
        }
    }

    /// Enqueue a gesture begin event if automation is enabled.
    pub fn push_gesture_begin(&self, config: &AutomationConfig, param_id: ClapId) {
        if !config.is_enabled(param_id) {
            return;
        }
        if let Ok(mut events) = self.events.lock() {
            events.push(AutomationEvent::GestureBegin(param_id));
        }
    }

    /// Enqueue a gesture end event if automation is enabled.
    pub fn push_gesture_end(&self, config: &AutomationConfig, param_id: ClapId) {
        if !config.is_enabled(param_id) {
            return;
        }
        if let Ok(mut events) = self.events.lock() {
            events.push(AutomationEvent::GestureEnd(param_id));
        }
    }

    /// Drain queued automation events into an output buffer.
    ///
    /// The caller supplies a scratch buffer to avoid allocations in realtime
    /// threads. Returns `true` if any events were drained.
    pub fn drain_to_output(
        &self,
        output: &mut OutputEvents<'_>,
        scratch: &mut Vec<AutomationEvent>,
    ) -> bool {
        let Ok(mut events) = self.events.try_lock() else {
            return false;
        };
        if events.is_empty() {
            return false;
        }
        scratch.clear();
        scratch.extend(events.drain(..));
        drop(events);

        for event in scratch.drain(..) {
            match event {
                AutomationEvent::GestureBegin(param_id) => {
                    let _ = push_param_gesture_begin(output, 0, param_id);
                }
                AutomationEvent::GestureEnd(param_id) => {
                    let _ = push_param_gesture_end(output, 0, param_id);
                }
                AutomationEvent::Value(param_id, value) => {
                    let _ = push_param_value(
                        output,
                        0,
                        param_id,
                        value,
                        Pckn::match_all(),
                        Cookie::empty(),
                    );
                }
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clack_plugin::events::io::EventBuffer;

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

        assert!(queue.drain_to_output(&mut output, &mut scratch));
    }
}
