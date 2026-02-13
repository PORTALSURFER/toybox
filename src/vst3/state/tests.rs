use super::{
    StreamError,
    decode_versioned_payload,
    encode_versioned_payload,
    read_versioned_payload,
    write_all,
    write_versioned_payload,
};
use super::{MAX_STATE_PAYLOAD_BYTES as STATE_LIMIT, try_encode_versioned_payload};
use std::ffi::c_void;
use std::ptr;
use toybox_vst3_ffi::Steinberg::{
    int32,
    int64,
    tresult,
    FUnknown,
    FUnknownVtbl,
    IBStream,
    IBStreamVtbl,
    TUID,
    kInvalidArgument,
    kNoInterface,
    kResultOk,
};

const MAGIC: u32 = 0x4D475354;

#[repr(C)]
struct FakeStream {
    base: IBStream,
    data: Vec<u8>,
    cursor: usize,
}

impl FakeStream {
    fn read(payload: Vec<u8>) -> Self {
        Self {
            base: IBStream {
                vtbl: &STREAM_VTABLE,
            },
            data: payload,
            cursor: 0,
        }
    }

    fn write() -> Self {
        Self {
            base: IBStream {
                vtbl: &STREAM_VTABLE,
            },
            data: Vec::new(),
            cursor: 0,
        }
    }

    fn as_ptr(&mut self) -> *mut IBStream {
        &mut self.base as *mut IBStream
    }
}

impl Default for FakeStream {
    fn default() -> Self {
        Self::write()
    }
}

static STREAM_VTABLE: IBStreamVtbl = IBStreamVtbl {
    base: FUnknownVtbl {
        queryInterface: fake_query_interface,
        addRef: fake_add_ref,
        release: fake_release,
    },
    read: fake_read,
    write: fake_write,
    seek: fake_seek,
    tell: fake_tell,
};

unsafe extern "system" fn fake_query_interface(
    _this: *mut FUnknown,
    _iid: *const TUID,
    obj: *mut *mut c_void,
) -> tresult {
    if !obj.is_null() {
        // No other interfaces are supported for this fake stream.
        unsafe { *obj = ptr::null_mut() };
    }
    kNoInterface
}

unsafe extern "system" fn fake_add_ref(_this: *mut FUnknown) -> u32 {
    1
}

unsafe extern "system" fn fake_release(_this: *mut FUnknown) -> u32 {
    1
}

unsafe extern "system" fn fake_read(
    this: *mut IBStream,
    buffer: *mut c_void,
    num_bytes: int32,
    num_bytes_read: *mut int32,
) -> tresult {
    if num_bytes <= 0 {
        if !num_bytes_read.is_null() {
            unsafe { *num_bytes_read = 0 };
        }
        return kInvalidArgument;
    }

    let stream = unsafe { &mut *(this as *mut FakeStream) };
    let available = stream.data.len().saturating_sub(stream.cursor);
    let to_read = (num_bytes as usize).min(available);
    if to_read > 0 {
        unsafe {
            std::ptr::copy_nonoverlapping(
                stream.data.as_ptr().add(stream.cursor),
                buffer as *mut u8,
                to_read,
            )
        };
        stream.cursor += to_read;
    }

    if !num_bytes_read.is_null() {
        unsafe { *num_bytes_read = to_read as int32 };
    }
    kResultOk
}

unsafe extern "system" fn fake_write(
    this: *mut IBStream,
    buffer: *mut c_void,
    num_bytes: int32,
    num_bytes_written: *mut int32,
) -> tresult {
    if num_bytes < 0 {
        if !num_bytes_written.is_null() {
            unsafe { *num_bytes_written = 0 };
        }
        return kInvalidArgument;
    }

    let stream = unsafe { &mut *(this as *mut FakeStream) };
    if num_bytes > 0 {
        let data = unsafe { std::slice::from_raw_parts(buffer as *const u8, num_bytes as usize) };
        stream.data.extend_from_slice(data);
        stream.cursor = stream.data.len();
    }

    if !num_bytes_written.is_null() {
        unsafe { *num_bytes_written = num_bytes };
    }
    kResultOk
}

unsafe extern "system" fn fake_seek(
    this: *mut IBStream,
    pos: int64,
    _mode: i32,
    _result: *mut int64,
) -> tresult {
    if pos < 0 {
        return kInvalidArgument;
    }

    let stream = unsafe { &mut *(this as *mut FakeStream) };
    stream.cursor = pos as usize;
    kResultOk
}

unsafe extern "system" fn fake_tell(this: *mut IBStream, result: *mut int64) -> tresult {
    let stream = unsafe { &mut *(this as *mut FakeStream) };
    if !result.is_null() {
        unsafe { *result = stream.cursor as int64 };
    }
    kResultOk
}

fn header_with_payload_len(payload_len: u32) -> Vec<u8> {
    let mut header = Vec::with_capacity(12);
    header.extend_from_slice(&MAGIC.to_le_bytes());
    header.extend_from_slice(&2u32.to_le_bytes());
    header.extend_from_slice(&payload_len.to_le_bytes());
    header
}

#[test]
fn encode_decode_round_trip() {
    let payload = encode_versioned_payload(MAGIC, 2, &[1, 2, 3, 4]);
    assert_eq!(
        payload.len(),
        16,
        "encoding should include u32 magic+version+len header"
    );
    let decoded = decode_versioned_payload(&payload, MAGIC, &[1, 2]).expect("payload should decode");
    assert_eq!(decoded.version, 2);
    assert_eq!(decoded.payload, vec![1, 2, 3, 4]);
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
fn encode_rejects_oversized_payload() {
    let payload = vec![0u8; STATE_LIMIT + 1];
    let result = try_encode_versioned_payload(0x4D475354, 2, &payload);
    assert_eq!(result, Err(StreamError::PayloadTooLarge));
}

#[test]
fn decode_rejects_oversized_header_length() {
    let mut header = [0x02u8; 12];
    header[0..4].copy_from_slice(&MAGIC.to_le_bytes());
    header[4..8].copy_from_slice(&2u32.to_le_bytes());
    header[8..12].copy_from_slice(&0x10000000u32.to_le_bytes());
    let error =
        decode_versioned_payload(&header, MAGIC, &[1, 2]).expect_err("oversized header should fail");
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
fn read_versioned_payload_rejects_bad_magic() {
    let encoded = encode_versioned_payload(0x11111111, 2, &[]);
    let mut stream = FakeStream::read(encoded);
    let err = unsafe { read_versioned_payload(stream.as_ptr(), MAGIC, &[1, 2]) }
        .expect_err("magic mismatch should fail");
    assert_eq!(err, StreamError::InvalidMagic);
}

#[test]
fn read_versioned_payload_rejects_oversized_header_length() {
    let payload = header_with_payload_len(u32::try_from(STATE_LIMIT + 1).expect("limit fits in u32"));
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

#[test]
fn write_versioned_payload_round_trip() {
    let mut stream = FakeStream::write();
    let payload = [10u8, 20, 30, 40, 50];

    unsafe { write_versioned_payload(stream.as_ptr(), MAGIC, 2, &payload).expect("write should pass") };

    let mut stream = FakeStream::read(stream.data);
    let decoded = unsafe { read_versioned_payload(stream.as_ptr(), MAGIC, &[2]).expect("read back should pass") };
    assert_eq!(decoded.payload, payload);
    assert_eq!(decoded.version, 2);
}

#[test]
fn write_versioned_payload_rejects_null_stream() {
    let error = unsafe { write_versioned_payload(ptr::null_mut(), MAGIC, 1, &[1, 2, 3]) };
    assert_eq!(error, Err(StreamError::NullStream));
}
