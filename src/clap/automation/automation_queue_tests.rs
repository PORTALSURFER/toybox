//! Unit tests for the GUI automation event queue.

use clack_plugin::events::io::EventBuffer;
use clack_plugin::events::spaces::CoreEventSpace;
use clack_plugin::utils::ClapId;

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

    let stats = queue.drain_to_output(&mut output, &mut scratch);
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

    let stats = drain_buffer.drain(&queue, &mut output);
    assert_eq!(stats.attempted, 1);
}

#[test]
fn push_respects_disabled_params() {
    let queue = AutomationQueue::default();
    let mut config = AutomationConfig::default();
    let param_id = ClapId::new(9);
    config.disable_param(param_id);

    let status = queue.push_value(&config, param_id, 0.7);
    assert_eq!(status, AutomationEnqueueStatus::Disabled);

    let mut output_buffer = EventBuffer::new();
    let mut output = output_buffer.as_output();
    let mut scratch = Vec::new();
    let stats = queue.drain_to_output(&mut output, &mut scratch);
    assert_eq!(stats.attempted, 0);
}

#[test]
fn push_reports_poisoned_queue() {
    let queue = AutomationQueue::default();
    let config = AutomationConfig::default();
    let param_id = ClapId::new(3);

    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = queue.events.lock().expect("lock should succeed");
        panic!("poison queue");
    }));

    let status = queue.push_value(&config, param_id, 0.25);
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
        queue.push_value(&config, param_id, 0.1),
        AutomationEnqueueStatus::Enqueued
    );
    assert_eq!(
        queue.push_value(&config, param_id, 0.2),
        AutomationEnqueueStatus::QueueFull
    );

    let mut output_buffer = EventBuffer::new();
    let mut output = output_buffer.as_output();
    let mut scratch = Vec::new();
    let stats = queue.drain_to_output(&mut output, &mut scratch);
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
        queue.push_value(&config, first, 0.1),
        AutomationEnqueueStatus::Enqueued
    );
    assert_eq!(
        queue.push_value(&config, second, 0.2),
        AutomationEnqueueStatus::Enqueued
    );
    assert_eq!(
        queue.push_value(&config, third, 0.3),
        AutomationEnqueueStatus::Enqueued
    );

    let mut output_buffer = EventBuffer::new();
    let mut output = output_buffer.as_output();
    let mut scratch = Vec::new();
    let stats = queue.drain_to_output(&mut output, &mut scratch);
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
