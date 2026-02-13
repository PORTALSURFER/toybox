//! State serialization helpers for VST3 plugins.

use std::fmt;

use toybox_vst3_ffi::ComRef;
use toybox_vst3_ffi::Steinberg::{IBStream, IBStreamTrait, int32, kResultOk};

/// Shared maximum accepted state payload size in bytes.
///
/// Re-exported here to keep VST3 and CLAP helper limits consistent.
pub use crate::state::MAX_STATE_PAYLOAD_BYTES;

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

/// Write all bytes to a VST3 `IBStream`.
///
/// # Safety
///
/// `stream` must be a valid VST3 stream pointer.
pub unsafe fn write_all(stream: *mut IBStream, bytes: &[u8]) -> Result<(), StreamError> {
    let Some(stream) = (unsafe { ComRef::from_raw(stream) }) else {
        return Err(StreamError::NullStream);
    };

    let mut written_total = 0usize;
    while written_total < bytes.len() {
        let remaining = &bytes[written_total..];
        let chunk = remaining.len().min(i32::MAX as usize);
        let mut written: int32 = 0;
        let result = unsafe {
            stream.write(
                remaining.as_ptr().cast_mut().cast(),
                chunk as int32,
                &mut written,
            )
        };
        if result != kResultOk {
            return Err(StreamError::IoFailure);
        }
        if written <= 0 {
            return Err(StreamError::UnexpectedEof);
        }
        written_total += written as usize;
    }

    Ok(())
}

/// Read exactly `bytes.len()` bytes from a VST3 `IBStream`.
///
/// # Safety
///
/// `stream` must be a valid VST3 stream pointer.
pub unsafe fn read_exact(stream: *mut IBStream, bytes: &mut [u8]) -> Result<(), StreamError> {
    let Some(stream) = (unsafe { ComRef::from_raw(stream) }) else {
        return Err(StreamError::NullStream);
    };

    let mut read_total = 0usize;
    while read_total < bytes.len() {
        let remaining = &mut bytes[read_total..];
        let chunk = remaining.len().min(i32::MAX as usize);
        let mut read: int32 = 0;
        let result = unsafe {
            stream.read(
                remaining[..chunk].as_mut_ptr().cast(),
                chunk as int32,
                &mut read,
            )
        };
        if result != kResultOk {
            return Err(StreamError::IoFailure);
        }
        if read <= 0 {
            return Err(StreamError::UnexpectedEof);
        }
        read_total += read as usize;
    }

    Ok(())
}

/// Encode a versioned state payload into a byte vector.
///
/// This remains a compatibility API for existing callers. It will panic if the
/// payload length exceeds [`MAX_STATE_PAYLOAD_BYTES`]; use
/// [`try_encode_versioned_payload`] for fallible handling.
///
/// # Panics
///
/// Panics with [`VST3_STATE_PAYLOAD_TOO_LARGE_ERROR`] when encoding would
/// exceed [`MAX_STATE_PAYLOAD_BYTES`].
///
/// The format is little-endian:
/// `magic:u32 | version:u32 | payload_len:u32 | payload`.
pub fn encode_versioned_payload(magic: u32, version: u32, payload: &[u8]) -> Vec<u8> {
    let Ok(bytes) = try_encode_versioned_payload(magic, version, payload) else {
        panic!("{VST3_STATE_PAYLOAD_TOO_LARGE_ERROR}");
    };

    bytes
}

/// Encode a versioned state payload into a byte vector.
///
/// Returns [`StreamError::PayloadTooLarge`] when payload length exceeds
/// [`MAX_STATE_PAYLOAD_BYTES`].
pub fn try_encode_versioned_payload(
    magic: u32,
    version: u32,
    payload: &[u8],
) -> Result<Vec<u8>, StreamError> {
    check_payload_length(payload.len())?;
    let payload_len = u32::try_from(payload.len()).map_err(|_| StreamError::PayloadTooLarge)?;

    let mut bytes = Vec::with_capacity(12 + payload.len());
    bytes.extend_from_slice(&magic.to_le_bytes());
    bytes.extend_from_slice(&version.to_le_bytes());
    bytes.extend_from_slice(&payload_len.to_le_bytes());
    bytes.extend_from_slice(payload);
    Ok(bytes)
}

/// Decode a versioned state payload from bytes.
pub fn decode_versioned_payload(
    bytes: &[u8],
    expected_magic: u32,
    accepted_versions: &[u32],
) -> Result<VersionedPayload, StreamError> {
    if bytes.len() < 12 {
        return Err(StreamError::InvalidHeader);
    }

    let magic = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    if magic != expected_magic {
        return Err(StreamError::InvalidMagic);
    }

    let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    if !accepted_versions.contains(&version) {
        return Err(StreamError::UnsupportedVersion);
    }

    let payload_len = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
    check_payload_length(payload_len)?;
    if bytes.len() != 12 + payload_len {
        return Err(StreamError::InvalidHeader);
    }

    Ok(VersionedPayload {
        version,
        payload: bytes[12..].to_vec(),
    })
}

/// Write a versioned payload to a VST3 stream.
///
/// # Safety
///
/// `stream` must be a valid VST3 stream pointer.
pub unsafe fn write_versioned_payload(
    stream: *mut IBStream,
    magic: u32,
    version: u32,
    payload: &[u8],
) -> Result<(), StreamError> {
    let bytes = try_encode_versioned_payload(magic, version, payload)?;
    // SAFETY: caller guarantees `stream` points to a valid IBStream.
    unsafe { write_all(stream, &bytes) }
}

/// Read and decode a versioned payload from a VST3 stream.
///
/// # Safety
///
/// `stream` must be a valid VST3 stream pointer.
pub unsafe fn read_versioned_payload(
    stream: *mut IBStream,
    expected_magic: u32,
    accepted_versions: &[u32],
) -> Result<VersionedPayload, StreamError> {
    let mut header = [0u8; 12];
    // SAFETY: caller guarantees `stream` points to a valid IBStream.
    unsafe { read_exact(stream, &mut header) }?;

    let magic = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);
    if magic != expected_magic {
        return Err(StreamError::InvalidMagic);
    }

    let version = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
    if !accepted_versions.contains(&version) {
        return Err(StreamError::UnsupportedVersion);
    }

    let payload_len = u32::from_le_bytes([header[8], header[9], header[10], header[11]]) as usize;
    check_payload_length(payload_len)?;

    let mut payload = vec![0u8; payload_len];
    // SAFETY: caller guarantees `stream` points to a valid IBStream.
    unsafe { read_exact(stream, &mut payload) }?;

    Ok(VersionedPayload { version, payload })
}

#[cfg(test)]
mod tests;
