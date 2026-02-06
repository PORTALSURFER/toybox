//! State serialization helpers for VST3 plugins.

use std::fmt;

use toybox_vst3_ffi::ComRef;
use toybox_vst3_ffi::Steinberg::{IBStream, IBStreamTrait, int32, kResultOk};

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
        }
    }
}

impl std::error::Error for StreamError {}

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
        let mut written: int32 = 0;
        let result = unsafe {
            stream.write(
                remaining.as_ptr().cast_mut().cast(),
                remaining.len() as int32,
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
        let mut read: int32 = 0;
        let result = unsafe {
            stream.read(
                remaining.as_mut_ptr().cast(),
                remaining.len() as int32,
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
/// The format is little-endian:
/// `magic:u32 | version:u32 | payload_len:u32 | payload`.
pub fn encode_versioned_payload(magic: u32, version: u32, payload: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(12 + payload.len());
    bytes.extend_from_slice(&magic.to_le_bytes());
    bytes.extend_from_slice(&version.to_le_bytes());
    bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    bytes.extend_from_slice(payload);
    bytes
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
    let bytes = encode_versioned_payload(magic, version, payload);
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
    let payload_len = u32::from_le_bytes([header[8], header[9], header[10], header[11]]) as usize;

    let mut payload = vec![0u8; payload_len];
    // SAFETY: caller guarantees `stream` points to a valid IBStream.
    unsafe { read_exact(stream, &mut payload) }?;

    let mut bytes = Vec::with_capacity(12 + payload_len);
    bytes.extend_from_slice(&header);
    bytes.extend_from_slice(&payload);

    decode_versioned_payload(&bytes, expected_magic, accepted_versions)
}

#[cfg(test)]
mod tests {
    use super::{StreamError, decode_versioned_payload, encode_versioned_payload};

    #[test]
    fn encode_decode_round_trip() {
        let payload = encode_versioned_payload(0x4D475354, 2, &[1, 2, 3, 4]);
        let decoded =
            decode_versioned_payload(&payload, 0x4D475354, &[1, 2]).expect("payload should decode");
        assert_eq!(decoded.version, 2);
        assert_eq!(decoded.payload, vec![1, 2, 3, 4]);
    }

    #[test]
    fn rejects_invalid_magic() {
        let payload = encode_versioned_payload(0x11111111, 1, &[]);
        let err = decode_versioned_payload(&payload, 0x22222222, &[1])
            .expect_err("magic mismatch should fail");
        assert_eq!(err, StreamError::InvalidMagic);
    }

    #[test]
    fn rejects_unsupported_version() {
        let payload = encode_versioned_payload(0x11111111, 9, &[]);
        let err = decode_versioned_payload(&payload, 0x11111111, &[1, 2])
            .expect_err("unsupported version should fail");
        assert_eq!(err, StreamError::UnsupportedVersion);
    }
}
