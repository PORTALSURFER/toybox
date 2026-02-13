//! Shared constants used by plugin state serialization helpers.

/// Maximum accepted versioned state payload size in bytes.
///
/// This constant is shared by CLAP and VST3 state helpers to keep state size
/// limits consistent across host formats.
pub const MAX_STATE_PAYLOAD_BYTES: usize = 16 * 1024 * 1024;
