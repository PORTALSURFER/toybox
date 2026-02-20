impl AutomationQueue {
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

    /// Enqueue a parameter value update and return the enqueue status.
    pub fn push_value(
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

    /// Enqueue a gesture begin event and return the enqueue status.
    pub fn push_gesture_begin(
        &self,
        config: &AutomationConfig,
        param_id: ClapId,
    ) -> AutomationEnqueueStatus {
        if !config.is_enabled(param_id) {
            return AutomationEnqueueStatus::Disabled;
        }
        self.try_enqueue(AutomationEvent::GestureBegin(param_id))
    }

    /// Enqueue a gesture end event and return the enqueue status.
    pub fn push_gesture_end(
        &self,
        config: &AutomationConfig,
        param_id: ClapId,
    ) -> AutomationEnqueueStatus {
        if !config.is_enabled(param_id) {
            return AutomationEnqueueStatus::Disabled;
        }
        self.try_enqueue(AutomationEvent::GestureEnd(param_id))
    }
}
