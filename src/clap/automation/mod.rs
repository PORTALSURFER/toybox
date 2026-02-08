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

mod automation_config;
mod automation_drain_buffer;
mod automation_event_types;
mod automation_queue;

pub use automation_config::AutomationConfig;
pub use automation_drain_buffer::AutomationDrainBuffer;
pub use automation_event_types::{
    AutomationDrainStats, AutomationDropPolicy, AutomationEnqueueStatus, AutomationEvent,
    AutomationQueueConfig, DEFAULT_AUTOMATION_QUEUE_MAX_EVENTS,
};
pub use automation_queue::AutomationQueue;
