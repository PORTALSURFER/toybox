//! Native child-window adapters for the embedded GPUI platform.

#[cfg(target_os = "macos")]
#[path = "native/macos.rs"]
mod macos;
#[cfg(target_os = "macos")]
pub(crate) use macos::{
    NativeChild, current_event_identity, parent_scale_factor, read_clipboard, write_clipboard,
};

#[cfg(target_os = "windows")]
#[path = "native/windows.rs"]
mod windows;
#[cfg(target_os = "windows")]
pub(crate) use windows::{NativeChild, parent_scale_factor, read_clipboard, write_clipboard};
