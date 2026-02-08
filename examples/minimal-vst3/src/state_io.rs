//! State serialization helpers for the minimal VST3 example.

use toybox::vst3::prelude::Steinberg::{IBStream, kResultFalse, kResultOk, tresult};
use toybox::vst3::prelude::{StreamError, read_versioned_payload, write_versioned_payload};

use crate::constants::{STATE_MAGIC, STATE_VERSION};

/// Read normalized gain value from a serialized plugin state stream.
pub(crate) unsafe fn load_normalized_gain(state: *mut IBStream) -> Result<f64, StreamError> {
    // SAFETY: caller provides a host-owned VST3 state stream pointer.
    let payload = unsafe { read_versioned_payload(state, STATE_MAGIC, &[STATE_VERSION]) }?;
    if payload.payload.len() != 8 {
        return Err(StreamError::InvalidHeader);
    }

    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&payload.payload);
    Ok(f64::from_le_bytes(bytes).clamp(0.0, 1.0))
}

/// Write normalized gain value to a serialized plugin state stream.
pub(crate) unsafe fn store_normalized_gain(state: *mut IBStream, normalized_gain: f64) -> tresult {
    let payload = normalized_gain.to_le_bytes();
    // SAFETY: caller provides a host-owned VST3 state stream pointer.
    match unsafe { write_versioned_payload(state, STATE_MAGIC, STATE_VERSION, &payload) } {
        Ok(()) => kResultOk,
        Err(_) => kResultFalse,
    }
}
