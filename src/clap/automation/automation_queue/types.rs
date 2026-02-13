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
}

impl Default for AutomationQueue {
    fn default() -> Self {
        Self::with_config(AutomationQueueConfig::default())
    }
}
