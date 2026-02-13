//! HostWindow behavior and Win32 parented-window lifecycle.

mod accessors;
mod parented;
mod spawn;

/// Parsed Win32 parent handles extracted from the host parent pointer.
#[derive(Clone, Copy, Debug)]
pub(super) struct ParentWindowHandles {
    /// Parent window handle value.
    pub(super) hwnd: isize,
    /// Parent module instance handle value.
    pub(super) hinstance: isize,
}
