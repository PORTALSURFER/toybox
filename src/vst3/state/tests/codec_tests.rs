use crate::vst3::state::{
    check_payload_length, decode_versioned_payload, encode_versioned_payload,
    try_encode_versioned_payload, StreamError, MAX_STATE_PAYLOAD_BYTES,
};

const MAGIC: u32 = 0x4D475354;

#[test]
fn encode_decode_round_trip() {
    let payload = encode_versioned_payload(MAGIC, 2, &[1, 2, 3, 4]);
    assert_eq!(
        payload.len(),
        16,
        "encoding should include u32 magic+version+len header"
    );
    let decoded =
        decode_versioned_payload(&payload, MAGIC, &[1, 2]).expect("payload should decode");
    assert_eq!(decoded.version, 2);
    assert_eq!(decoded.payload, vec![1, 2, 3, 4]);
}

#[test]
#[should_panic(expected = "VST3 state payload is too large")]
fn encode_panics_on_oversized_payload() {
    let payload = vec![0u8; MAX_STATE_PAYLOAD_BYTES + 1];
    let _ = encode_versioned_payload(MAGIC, 2, &payload);
}

#[test]
fn encode_rejects_oversized_payload() {
    let payload = vec![0u8; MAX_STATE_PAYLOAD_BYTES + 1];
    let result = try_encode_versioned_payload(0x4D475354, 2, &payload);
    assert_eq!(result, Err(StreamError::PayloadTooLarge));
}

#[test]
fn decode_rejects_oversized_header_length() {
    let mut header = [0x02u8; 12];
    header[0..4].copy_from_slice(&MAGIC.to_le_bytes());
    header[4..8].copy_from_slice(&2u32.to_le_bytes());
    header[8..12].copy_from_slice(&0x10000000u32.to_le_bytes());
    let error = decode_versioned_payload(&header, MAGIC, &[1, 2])
        .expect_err("oversized header should fail");
    assert_eq!(error, StreamError::PayloadTooLarge);
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

#[test]
fn encode_rejects_max_limit_boundary() {
    assert!(check_payload_length(MAX_STATE_PAYLOAD_BYTES).is_ok());
    let result = check_payload_length(MAX_STATE_PAYLOAD_BYTES + 1);
    assert_eq!(result, Err(StreamError::PayloadTooLarge));
}
