//! Non-Windows host window stubs for Patchbay GUI.
//!
//! This module keeps the crate buildable on platforms where the native window
//! backend is unavailable. GUI opening APIs return [`GuiError::UnsupportedHandle`].

mod errors;
mod requests;
mod types;
mod window;

pub use errors::GuiError;
pub use requests::{OpenParentedMode, OpenParentedRequest};
pub use types::{HostWindow, InputState, WindowHandle};

/// Pack a size into a compact atomic payload.
pub(super) fn pack_size(width: u32, height: u32) -> u64 {
    ((width as u64) << 32) | (height as u64)
}

/// Decode an atomic size payload.
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
    use crate::declarative::UiSpec;

    #[test]
    fn pack_unpack_roundtrip() {
        let packed = pack_size(640, 480);
        assert_eq!(unpack_size(packed), Some((640, 480)));
    }

    #[test]
    fn open_parented_reports_unsupported_after_parent() {
        let mut host = HostWindow::default();
        host.set_parent(raw_window_handle::RawWindowHandle::AppKit(
            raw_window_handle::AppKitWindowHandle::empty(),
        ));
        let result = host.open_parented(
            "Stub".into(),
            (320, 200),
            (),
            |_state| {},
            |_input, _state| {
                UiSpec::new(crate::declarative::RootFrameSpec::new(
                    "root",
                    crate::declarative::Node::Spacer(crate::declarative::SpacerSpec::new(
                        crate::canvas::Size {
                            width: 1,
                            height: 1,
                        },
                    )),
                ))
            },
            |_state, _action| {},
        );
        assert!(matches!(result, Err(GuiError::UnsupportedHandle)));
    }
}
