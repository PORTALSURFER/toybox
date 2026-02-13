//! HostWindow behavior and Win32 parented-window lifecycle.

mod accessors;
mod parented;
mod spawn;

pub(super) use parented::ParentWindowHandles;
