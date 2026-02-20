//! Host window management and input plumbing for Patchbay GUI.

mod errors;
mod requests;
mod types;
mod window;

pub use errors::GuiError;
pub use requests::{OpenParentedCallbacks, OpenParentedMode, OpenParentedRequest};
pub use types::{HostWindow, InputState, ShortcutBinding, ShortcutModifiers};

/// Pack a width/height pair into an atomic payload.
pub(super) fn pack_size(width: u32, height: u32) -> u64 {
    ((width as u64) << 32) | (height as u64)
}

/// Decode an atomic size payload into `(width, height)`.
pub(super) fn unpack_size(value: u64) -> Option<(u32, u32)> {
    if value == 0 {
        return None;
    }
    let width = (value >> 32) as u32;
    let height = (value & 0xFFFF_FFFF) as u32;
    Some((width, height))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_unpack_roundtrip() {
        let packed = pack_size(640, 480);
        assert_eq!(unpack_size(packed), Some((640, 480)));
    }
}
