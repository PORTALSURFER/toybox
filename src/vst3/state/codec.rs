use super::{check_payload_length, StreamError, VersionedPayload};

/// Encode a versioned state payload into a byte vector.
///
/// Returns [`StreamError::PayloadTooLarge`] when payload length exceeds
/// the configured payload limit.
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
