//! Queue and drain logic for GUI automation events.

use std::collections::VecDeque;
use std::sync::Mutex;

use clack_plugin::events::Pckn;
use clack_plugin::events::io::OutputEvents;
use clack_plugin::utils::{ClapId, Cookie};

use crate::clap::params::{
    ParamEventContext, push_param_gesture_begin, push_param_gesture_end, push_param_value,
};

use super::{
    AutomationConfig, AutomationDrainStats, AutomationDropPolicy, AutomationEnqueueStatus,
    AutomationEvent, AutomationQueueConfig,
};

include!("automation_queue/types.rs");
include!("automation_queue/enqueue.rs");
include!("automation_queue/drain.rs");

#[cfg(test)]
#[path = "automation_queue_tests.rs"]
mod tests;
