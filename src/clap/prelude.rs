//! Common CLAP imports for downstream plugins.
//!
//! This prelude collects clack and toybox helpers used by most plugins so
//! plugin crates can avoid depending on clack directly.
//!
//! # Example
//! ```
//! use toybox::clap::prelude::*;
//! use toybox::clap::params::ParamBuilder;
//!
//! const PARAM_GAIN: ClapId = ClapId::new(0);
//!
//! let _spec = ParamBuilder::new(PARAM_GAIN, b"Gain", b"Gain")
//!     .automatable()
//!     .range(0.0, 2.0)
//!     .default(1.0)
//!     .build();
//! ```

pub use clack_extensions::audio_ports::*;
#[cfg(feature = "gui")]
pub use clack_extensions::gui::*;
pub use clack_extensions::note_ports::*;
pub use clack_extensions::params::*;
pub use clack_extensions::state::*;
pub use clack_plugin::events::{self, Pckn, io as events_io, spaces as event_spaces};
pub use clack_plugin::prelude::*;
pub use clack_plugin::utils::{ClapId, Cookie};

pub use crate::clap::automation::{
    AutomationConfig, AutomationDrainBuffer, AutomationDrainStats, AutomationDropPolicy,
    AutomationEnqueueStatus, AutomationEvent, AutomationQueue, AutomationQueueConfig,
    DEFAULT_AUTOMATION_QUEUE_MAX_EVENTS,
};
pub use crate::clap::entry::PluginEntry;
pub use crate::clap::events::{EventRouter, bounds_to_range};
pub use crate::clap::params::{
    ParamBuilder, ParamSpec, apply_param_events, apply_param_events_from_unknown, bool_to_param,
    param_to_bool, parse_toggle_text, push_param_gesture_begin, push_param_gesture_end,
    push_param_mod, push_param_value, write_toggle_text,
};
pub use crate::clap::process::ProcessContext;
pub use crate::clap::registration::register_default_extensions;
#[cfg(feature = "gui")]
pub use crate::clap::registration::register_default_extensions_with_gui;
pub use crate::clap::state::{
    MAX_STATE_PAYLOAD_BYTES, VersionedStatePayload, read_versioned_payload, write_versioned_payload,
};
pub use crate::clap_plugin_entry;
