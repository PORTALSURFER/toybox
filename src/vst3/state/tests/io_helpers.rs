//! Utilities for VST3 state stream tests.

use std::ffi::c_void;
use std::ptr;
use toybox_vst3_ffi::Steinberg::{
    int32, int64, kInvalidArgument, kNoInterface, kResultOk, tresult, FUnknown, FUnknownVtbl,
    IBStream, IBStreamVtbl, TUID,
};

/// Fixed platform-independent marker used by stream test fixtures.
pub const MAGIC: u32 = 0x4D475354;

/// Fake VST3 stream implementation used in state IO tests.
#[repr(C)]
pub struct FakeStream {
    /// Backing COM interface for this fake stream.
    base: IBStream,
    /// Stream data buffer shared by read and write paths.
    pub data: Vec<u8>,
    /// Current cursor position in bytes.
    pub cursor: usize,
}

impl FakeStream {
    /// Construct a stream seeded with an existing payload.
    pub fn read(payload: Vec<u8>) -> Self {
        Self {
            base: IBStream {
                vtbl: &STREAM_VTABLE,
            },
            data: payload,
            cursor: 0,
        }
    }

    /// Construct an empty stream used by write tests.
    pub fn write() -> Self {
        Self {
            base: IBStream {
                vtbl: &STREAM_VTABLE,
            },
            data: Vec::new(),
            cursor: 0,
        }
    }

    /// Return a mutable pointer to the underlying COM object.
    pub fn as_ptr(&mut self) -> *mut IBStream {
        &mut self.base as *mut IBStream
    }
}

impl Default for FakeStream {
    fn default() -> Self {
        Self::write()
    }
}

/// Vtable backing the fake `IBStream` implementation.
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

/// `IBStream` query entrypoint for the fake VST3 stream.
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

/// Fake COM addref implementation.
unsafe extern "system" fn fake_add_ref(_this: *mut FUnknown) -> u32 {
    1
}

/// Fake COM release implementation.
unsafe extern "system" fn fake_release(_this: *mut FUnknown) -> u32 {
    1
}

/// Fake stream read implementation that copies bytes from the backing buffer.
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

/// Fake stream write implementation that appends bytes to the backing buffer.
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

/// Fake stream seek implementation that updates the cursor.
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

/// Fake stream tell implementation that reports the cursor position.
unsafe extern "system" fn fake_tell(this: *mut IBStream, result: *mut int64) -> tresult {
    let stream = unsafe { &mut *(this as *mut FakeStream) };
    if !result.is_null() {
        unsafe { *result = stream.cursor as int64 };
    }
    kResultOk
}
