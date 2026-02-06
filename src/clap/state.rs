//! CLAP plugin-state helpers for versioned binary payloads.
//!
//! These helpers provide a consistent wire format for plugin state blobs:
//! `magic:u32`, `version:u32`, `payload_len:u32`, and `payload` bytes.

use std::io::{Read, Write};

use clack_plugin::plugin::PluginError;
use clack_plugin::stream::{InputStream, OutputStream};

/// Maximum accepted state payload size in bytes.
///
/// This limit protects hosts/plugins from accidentally allocating unbounded
/// memory when a payload header is corrupted.
pub const MAX_STATE_PAYLOAD_BYTES: usize = 16 * 1024 * 1024;

/// Decoded versioned state payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionedStatePayload {
    /// Payload version from the serialized state header.
    pub version: u32,
    /// Raw payload bytes following the state header.
    pub payload: Vec<u8>,
}

/// Write a versioned state payload to a CLAP output stream.
///
/// # Errors
/// Returns an error if the stream write fails, the payload exceeds
/// [`MAX_STATE_PAYLOAD_BYTES`], or the payload length does not fit into a
/// `u32` header.
pub fn write_versioned_payload(
    output: &mut OutputStream,
    magic: u32,
    version: u32,
    payload: &[u8],
) -> Result<(), PluginError> {
    if payload.len() > MAX_STATE_PAYLOAD_BYTES {
        return Err(PluginError::Message(
            "Plugin state payload exceeds max size",
        ));
    }
    let payload_len = u32::try_from(payload.len())
        .map_err(|_| PluginError::Message("Plugin state payload exceeds u32 header size"))?;

    output.write_all(&magic.to_le_bytes())?;
    output.write_all(&version.to_le_bytes())?;
    output.write_all(&payload_len.to_le_bytes())?;
    output.write_all(payload)?;
    Ok(())
}

/// Read and validate a versioned state payload from a CLAP input stream.
///
/// The payload is accepted only when:
/// - `magic` matches `expected_magic`,
/// - `version` is listed in `supported_versions`,
/// - `payload_len` does not exceed [`MAX_STATE_PAYLOAD_BYTES`].
///
/// # Errors
/// Returns a descriptive [`PluginError::Message`] when validation fails, or an
/// I/O derived error if the stream read fails.
pub fn read_versioned_payload(
    input: &mut InputStream,
    expected_magic: u32,
    supported_versions: &[u32],
) -> Result<VersionedStatePayload, PluginError> {
    let magic = read_u32(input)?;
    let version = read_u32(input)?;
    let payload_len = read_u32(input)? as usize;

    if magic != expected_magic {
        return Err(PluginError::Message("Invalid plugin state payload"));
    }
    if !supported_versions.contains(&version) {
        return Err(PluginError::Message("Unsupported plugin state version"));
    }
    if payload_len > MAX_STATE_PAYLOAD_BYTES {
        return Err(PluginError::Message("Invalid plugin state payload"));
    }

    let mut payload = vec![0u8; payload_len];
    input.read_exact(&mut payload)?;

    Ok(VersionedStatePayload { version, payload })
}

/// Read a little-endian `u32` from a CLAP input stream.
fn read_u32(input: &mut InputStream) -> Result<u32, PluginError> {
    let mut bytes = [0u8; 4];
    input.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use clack_common::stream::{InputStream, OutputStream};
    use clack_plugin::plugin::PluginError;

    use super::{MAX_STATE_PAYLOAD_BYTES, read_versioned_payload, write_versioned_payload};

    #[test]
    fn versioned_payload_roundtrip() {
        let mut data = Vec::new();
        let mut output = OutputStream::from_writer(&mut data);
        let payload = [1u8, 2u8, 3u8, 4u8];
        write_versioned_payload(&mut output, u32::from_le_bytes(*b"TEST"), 3, &payload)
            .expect("should write payload");

        let mut cursor = data.as_slice();
        let mut input = InputStream::from_reader(&mut cursor);
        let decoded = read_versioned_payload(&mut input, u32::from_le_bytes(*b"TEST"), &[2, 3])
            .expect("should read payload");
        assert_eq!(decoded.version, 3);
        assert_eq!(decoded.payload, payload);
    }

    #[test]
    fn write_rejects_oversized_payload() {
        let mut data = Vec::new();
        let mut output = OutputStream::from_writer(&mut data);
        let payload = vec![0u8; MAX_STATE_PAYLOAD_BYTES + 1];
        let error = write_versioned_payload(&mut output, u32::from_le_bytes(*b"TEST"), 1, &payload)
            .expect_err("expected payload size check");
        match error {
            PluginError::Message(message) => {
                assert_eq!(message, "Plugin state payload exceeds max size");
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }
}
