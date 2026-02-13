use super::{check_payload_length, StreamError, VersionedPayload};
use toybox_vst3_ffi::ComRef;
use toybox_vst3_ffi::Steinberg::{IBStream, IBStreamTrait, int32, kResultOk};

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
        let chunk = remaining.len().min(int32::MAX as usize);
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
        let chunk = remaining.len().min(int32::MAX as usize);
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
    let bytes = super::try_encode_versioned_payload(magic, version, payload)?;
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

