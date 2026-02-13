//! VST3 state stream IO tests.

use std::ptr;

use super::io_helpers::{FakeStream, MAGIC};
use crate::vst3::state::{
    encode_versioned_payload, read_versioned_payload, write_all, write_versioned_payload,
    StreamError, MAX_STATE_PAYLOAD_BYTES as STATE_LIMIT,
};

fn header_with_payload_len(payload_len: u32) -> Vec<u8> {
    let mut header = Vec::with_capacity(12);
    header.extend_from_slice(&MAGIC.to_le_bytes());
    header.extend_from_slice(&2u32.to_le_bytes());
    header.extend_from_slice(&payload_len.to_le_bytes());
    header
}

#[test]
fn write_all_rejects_null_stream() {
    let error = unsafe { write_all(ptr::null_mut(), &[1, 2, 3]) };
    assert_eq!(error, Err(StreamError::NullStream));
}

#[test]
fn write_all_round_trip() {
    let mut stream = FakeStream::write();
    let payload = [1u8, 2, 3, 4, 5];
    unsafe { write_all(stream.as_ptr(), &payload).expect("write should succeed") };
    assert_eq!(stream.data, payload);
}

#[test]
fn write_versioned_payload_round_trip() {
    let mut stream = FakeStream::write();
    let payload = [10u8, 20, 30, 40, 50];

    unsafe {
        write_versioned_payload(stream.as_ptr(), MAGIC, 2, &payload).expect("write should pass")
    };

    let mut stream = FakeStream::read(stream.data);
    let decoded = unsafe {
        read_versioned_payload(stream.as_ptr(), MAGIC, &[2]).expect("read back should pass")
    };
    assert_eq!(decoded.payload, payload);
    assert_eq!(decoded.version, 2);
}

#[test]
fn write_versioned_payload_round_trip_empty_payload() {
    let mut stream = FakeStream::write();
    let payload = [];

    unsafe {
        write_versioned_payload(stream.as_ptr(), MAGIC, 2, &payload).expect("write should pass")
    };

    let mut stream = FakeStream::read(stream.data);
    let decoded = unsafe {
        read_versioned_payload(stream.as_ptr(), MAGIC, &[2]).expect("read back should pass")
    };
    assert_eq!(decoded.payload, payload);
    assert_eq!(decoded.version, 2);
}

#[test]
fn write_versioned_payload_rejects_null_stream() {
    let error = unsafe { write_versioned_payload(ptr::null_mut(), MAGIC, 1, &[1, 2, 3]) };
    assert_eq!(error, Err(StreamError::NullStream));
}

#[test]
fn read_versioned_payload_round_trip() {
    let encoded = encode_versioned_payload(MAGIC, 2, &[1, 2, 3, 4]);
    let mut stream = FakeStream::read(encoded);
    let decoded = unsafe { read_versioned_payload(stream.as_ptr(), MAGIC, &[1, 2]) }
        .expect("stream should decode");

    assert_eq!(decoded.version, 2);
    assert_eq!(decoded.payload, vec![1, 2, 3, 4]);
}

#[test]
fn read_versioned_payload_rejects_unsupported_version() {
    let encoded = encode_versioned_payload(MAGIC, 9, &[]);
    let mut stream = FakeStream::read(encoded);
    let err = unsafe { read_versioned_payload(stream.as_ptr(), MAGIC, &[1, 2]) }
        .expect_err("unsupported version should fail");
    assert_eq!(err, StreamError::UnsupportedVersion);
}

#[test]
fn read_versioned_payload_rejects_empty_version_list() {
    let encoded = encode_versioned_payload(MAGIC, 1, &[]);
    let mut stream = FakeStream::read(encoded);
    let err = unsafe { read_versioned_payload(stream.as_ptr(), MAGIC, &[]) }
        .expect_err("empty accepted list should fail");
    assert_eq!(err, StreamError::UnsupportedVersion);
}

#[test]
fn read_versioned_payload_rejects_bad_magic() {
    let encoded = encode_versioned_payload(0x11111111, 2, &[]);
    let mut stream = FakeStream::read(encoded);
    let err = unsafe { read_versioned_payload(stream.as_ptr(), MAGIC, &[1, 2]) }
        .expect_err("magic mismatch should fail");
    assert_eq!(err, StreamError::InvalidMagic);
}

#[test]
fn read_versioned_payload_rejects_oversized_header_length() {
    let payload =
        header_with_payload_len(u32::try_from(STATE_LIMIT + 1).expect("limit fits in u32"));
    let mut stream = FakeStream::read(payload);
    let err = unsafe { read_versioned_payload(stream.as_ptr(), MAGIC, &[1, 2]) }
        .expect_err("oversized header should fail");
    assert_eq!(err, StreamError::PayloadTooLarge);
}

#[test]
fn read_versioned_payload_rejects_truncated_payload() {
    let mut header = header_with_payload_len(4);
    header.extend([1u8, 2, 3]);
    let mut stream = FakeStream::read(header);
    let err = unsafe { read_versioned_payload(stream.as_ptr(), MAGIC, &[1, 2]) }
        .expect_err("truncated payload should fail");
    assert_eq!(err, StreamError::UnexpectedEof);
}
