//! Non-Windows host window stubs for Patchbay GUI.
//!
//! This module keeps the crate buildable on platforms where the native window
//! backend is unavailable. GUI opening APIs return [`GuiError::UnsupportedHandle`].

mod errors;
mod requests;
mod types;
mod window;

pub use errors::GuiError;
pub use requests::{OpenParentedCallbacks, OpenParentedMode, OpenParentedRequest};
pub use types::{HostWindow, InputState, ShortcutBinding, ShortcutModifiers, WindowHandle};

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
        fn init_state(_: &mut ()) {}

        fn build_root_frame(_: &crate::InputState, _: &()) -> UiSpec {
            UiSpec::new(crate::declarative::RootFrameSpec::new(
                "root",
                crate::declarative::Node::Spacer(crate::declarative::SpacerSpec::new(
                    crate::canvas::Size {
                        width: 1,
                        height: 1,
                    },
                )),
            ))
        }

        fn reduce_action(_: &mut (), _: crate::declarative::UiAction) {}

        let mut host = HostWindow::default();
        host.set_parent(raw_window_handle::RawWindowHandle::AppKit(
            raw_window_handle::AppKitWindowHandle::empty(),
        ));
        let result = host.open_parented_with(OpenParentedRequest::with_callbacks(
            "Stub".into(),
            crate::canvas::Size {
                width: 320,
                height: 200,
            },
            OpenParentedCallbacks::new((), init_state, build_root_frame, reduce_action),
        ));
        assert!(matches!(result, Err(GuiError::UnsupportedHandle)));
    }
}
