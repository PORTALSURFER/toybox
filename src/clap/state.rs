//! CLAP plugin-state helpers.
//!
//! This module centralizes serialization/deserialization utilities for versioned
//! plugin state payloads used by CLAP hosts.

pub mod payload;

pub use crate::state::MAX_STATE_PAYLOAD_BYTES;
pub use payload::{VersionedStatePayload, read_versioned_payload, write_versioned_payload};
