//! State serialization helpers for VST3 plugins.

use std::fmt;

pub use crate::state::MAX_STATE_PAYLOAD_BYTES;

/// Internal codec helpers for VST3 payload encoding and decoding.
mod codec;
/// Internal stream adapters for reading/writing versioned VST3 state payloads.
mod io;

pub use codec::{decode_versioned_payload, try_encode_versioned_payload};
pub use io::{read_exact, read_versioned_payload, write_all, write_versioned_payload};

/// Panic message used by the compatibility wrapper when payload encoding exceeds
/// the supported size.
const VST3_STATE_PAYLOAD_TOO_LARGE_ERROR: &str = "VST3 state payload is too large";

/// Serialized state payload read from a versioned stream.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionedPayload {
    /// Version stored in the serialized payload header.
    pub version: u32,
    /// Opaque payload bytes.
    pub payload: Vec<u8>,
}

/// Error returned by VST3 state stream helpers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StreamError {
    /// The host supplied a null stream pointer.
    NullStream,
    /// The VST3 stream method returned a failure code.
    IoFailure,
    /// The stream ended before all requested bytes were read.
    UnexpectedEof,
    /// The payload magic did not match the expected value.
    InvalidMagic,
    /// The payload version is not supported.
    UnsupportedVersion,
    /// The serialized state header is malformed.
    InvalidHeader,
    /// The payload length exceeds `MAX_STATE_PAYLOAD_BYTES`.
    PayloadTooLarge,
}

impl fmt::Display for StreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StreamError::NullStream => write!(f, "state stream pointer is null"),
            StreamError::IoFailure => write!(f, "state stream I/O failed"),
            StreamError::UnexpectedEof => write!(f, "state stream ended unexpectedly"),
            StreamError::InvalidMagic => write!(f, "state payload magic mismatch"),
            StreamError::UnsupportedVersion => write!(f, "state payload version is unsupported"),
            StreamError::InvalidHeader => write!(f, "state payload header is invalid"),
            StreamError::PayloadTooLarge => {
                write!(f, "state payload exceeds max supported size")
            }
        }
    }
}

impl std::error::Error for StreamError {}

/// Validate that a payload length is within the supported state limit.
///
/// Returns [`StreamError::PayloadTooLarge`] when the length exceeds
/// [`MAX_STATE_PAYLOAD_BYTES`].
fn check_payload_length(length: usize) -> Result<(), StreamError> {
    if length > MAX_STATE_PAYLOAD_BYTES {
        return Err(StreamError::PayloadTooLarge);
    }
    Ok(())
}

/// Compatibility entrypoint for writing a versioned payload.
///
/// This remains a compatibility API for existing callers. It will panic if the
/// payload length exceeds [`MAX_STATE_PAYLOAD_BYTES`]; prefer
/// [`try_encode_versioned_payload`] for fallible handling.
///
/// # Panics
///
/// Panics with [`VST3_STATE_PAYLOAD_TOO_LARGE_ERROR`] when encoding would
/// exceed [`MAX_STATE_PAYLOAD_BYTES`].
pub fn encode_versioned_payload(magic: u32, version: u32, payload: &[u8]) -> Vec<u8> {
    let Ok(bytes) = try_encode_versioned_payload(magic, version, payload) else {
        panic!("{VST3_STATE_PAYLOAD_TOO_LARGE_ERROR}");
    };

    bytes
}

#[cfg(test)]
mod tests {
    mod codec_tests;
    mod io_helpers;
    mod io_tests;
}
