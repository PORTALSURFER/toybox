//! Tests for versioned CLAP payload serialization helpers.

use clack_common::stream::{InputStream, OutputStream};
use clack_plugin::plugin::PluginError;

use super::{
    MAX_STATE_PAYLOAD_BYTES, VERSIONED_STATE_PAYLOAD_HEADER_ERROR,
    VERSIONED_STATE_PAYLOAD_TOO_LARGE_ERROR, check_payload_length, read_versioned_payload,
    write_versioned_payload,
};

const MAGIC: u32 = u32::from_le_bytes(*b"TEST");

#[test]
fn versioned_payload_roundtrip() {
    let mut data = Vec::new();
    let mut output = OutputStream::from_writer(&mut data);
    let payload = [1u8, 2u8, 3u8, 4u8];
    write_versioned_payload(&mut output, MAGIC, 3, &payload).expect("should write payload");

    let mut cursor = data.as_slice();
    let mut input = InputStream::from_reader(&mut cursor);
    let decoded = read_versioned_payload(&mut input, MAGIC, &[2, 3]).expect("should read payload");
    assert_eq!(decoded.version, 3);
    assert_eq!(decoded.payload, payload);
}

#[test]
fn versioned_payload_roundtrip_empty_payload() {
    let mut data = Vec::new();
    let mut output = OutputStream::from_writer(&mut data);
    let payload: [u8; 0] = [];
    write_versioned_payload(&mut output, MAGIC, 1, &payload).expect("should write empty payload");

    let mut cursor = data.as_slice();
    let mut input = InputStream::from_reader(&mut cursor);
    let decoded = read_versioned_payload(&mut input, MAGIC, &[1]).expect("should read payload");
    assert_eq!(decoded.version, 1);
    assert!(decoded.payload.is_empty());
}

#[test]
fn write_rejects_oversized_payload() {
    let mut data = Vec::new();
    let mut output = OutputStream::from_writer(&mut data);
    let payload = vec![0u8; MAX_STATE_PAYLOAD_BYTES + 1];
    let error = write_versioned_payload(&mut output, MAGIC, 1, &payload)
        .expect_err("expected payload size check");
    match error {
        PluginError::Message(message) => {
            assert_eq!(message, VERSIONED_STATE_PAYLOAD_TOO_LARGE_ERROR);
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn read_rejects_unsupported_version() {
    let mut data = Vec::new();
    let mut output = OutputStream::from_writer(&mut data);
    write_versioned_payload(&mut output, MAGIC, 1, &[1, 2, 3, 4]).expect("should write payload");

    let mut cursor = data.as_slice();
    let mut input = InputStream::from_reader(&mut cursor);
    let error =
        read_versioned_payload(&mut input, MAGIC, &[2, 3]).expect_err("expected version check");
    match error {
        PluginError::Message(message) => {
            assert_eq!(message, VERSIONED_STATE_PAYLOAD_HEADER_ERROR);
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn read_rejects_empty_version_list() {
    let mut data = Vec::new();
    let mut output = OutputStream::from_writer(&mut data);
    write_versioned_payload(&mut output, MAGIC, 1, &[1, 2, 3]).expect("should write payload");

    let mut cursor = data.as_slice();
    let mut input = InputStream::from_reader(&mut cursor);
    let error =
        read_versioned_payload(&mut input, MAGIC, &[]).expect_err("expected version list check");
    match error {
        PluginError::Message(message) => {
            assert_eq!(message, VERSIONED_STATE_PAYLOAD_HEADER_ERROR);
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn read_rejects_bad_magic() {
    let mut data = Vec::new();
    let mut output = OutputStream::from_writer(&mut data);
    write_versioned_payload(&mut output, MAGIC, 1, &[5, 6, 7]).expect("should write payload");

    let mut cursor = data.as_slice();
    let mut input = InputStream::from_reader(&mut cursor);
    let bad_magic = u32::from_le_bytes(*b"NOPE");
    let error =
        read_versioned_payload(&mut input, bad_magic, &[1]).expect_err("expected magic check");
    match error {
        PluginError::Message(message) => {
            assert_eq!(message, VERSIONED_STATE_PAYLOAD_HEADER_ERROR);
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn check_payload_length_matches_max() {
    assert!(check_payload_length(MAX_STATE_PAYLOAD_BYTES).is_ok());
    let error =
        check_payload_length(MAX_STATE_PAYLOAD_BYTES + 1).expect_err("expected payload size check");
    assert!(matches!(
        error,
        PluginError::Message(msg) if msg == VERSIONED_STATE_PAYLOAD_TOO_LARGE_ERROR
    ));
}

#[test]
fn read_rejects_oversized_header_length() {
    let payload_len = u32::try_from(MAX_STATE_PAYLOAD_BYTES + 1).expect("length fits u32");
    let mut data = Vec::new();
    data.extend_from_slice(&MAGIC.to_le_bytes());
    data.extend_from_slice(&1u32.to_le_bytes());
    data.extend_from_slice(&payload_len.to_le_bytes());

    let mut cursor = data.as_slice();
    let mut input = InputStream::from_reader(&mut cursor);
    let error = read_versioned_payload(&mut input, MAGIC, &[1]).expect_err("header too long");
    match error {
        PluginError::Message(message) => {
            assert_eq!(message, VERSIONED_STATE_PAYLOAD_TOO_LARGE_ERROR);
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn read_rejects_truncated_payload() {
    let mut data = Vec::new();
    write_versioned_payload(
        &mut OutputStream::from_writer(&mut data),
        MAGIC,
        1,
        &[1, 2, 3, 4],
    )
    .expect("write payload");
    data.truncate(6);

    let mut cursor = data.as_slice();
    let mut input = InputStream::from_reader(&mut cursor);
    let error = read_versioned_payload(&mut input, MAGIC, &[1]).expect_err("truncated payload");
    if let PluginError::Message(message) = error {
        assert_ne!(message, VERSIONED_STATE_PAYLOAD_HEADER_ERROR);
    }
}
