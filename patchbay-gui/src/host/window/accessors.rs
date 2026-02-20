//! HostWindow accessors and state mutation helpers.

use std::sync::atomic::Ordering;

use raw_window_handle::RawWindowHandle;

use crate::canvas::Size;
use crate::host::types::HostWindow;
use crate::host::{pack_size, unpack_size};
use crate::win32::WindowHandle;

impl HostWindow {
    /// Assign the raw parent handle supplied by the CLAP host.
    pub fn set_parent(&mut self, parent: RawWindowHandle) {
        let new_parent_hwnd = match parent {
            RawWindowHandle::Win32(handle) => Some(handle.hwnd as isize),
            _ => None,
        };
        if self.parent_hwnd != new_parent_hwnd {
            if let Some(handle) = &self.handle {
                handle.destroy();
            }
            self.handle = None;
            self.parent_hwnd = new_parent_hwnd;
        }
        self.parent = Some(parent);
    }

    /// Return the most recent logical size reported by the window.
    pub fn last_size(&self) -> Option<(u32, u32)> {
        unpack_size(self.last_size.load(Ordering::Acquire))
    }

    /// Request a logical resize from the GUI thread.
    ///
    /// The requested size is also recorded immediately so callers observing
    /// `last_size` can react to host-size negotiation before the next render
    /// tick applies the resize request.
    pub fn request_resize(&self, width: u32, height: u32) {
        let size = Size {
            width: width.max(1),
            height: height.max(1),
        };
        self.record_last_size(size);
        self.resize_request
            .store(pack_size(size.width, size.height), Ordering::Release);
    }

    /// Return true if a native window has been created.
    pub fn is_open(&self) -> bool {
        self.handle.is_some()
    }

    /// Show the native window if it exists.
    pub fn show(&self) {
        if let Some(handle) = &self.handle {
            handle.set_visible(true);
        }
    }

    /// Hide the native window if it exists.
    pub fn hide(&self) {
        if let Some(handle) = &self.handle {
            handle.set_visible(false);
        }
    }

    /// Set a desired aspect ratio for host-driven resizing.
    pub fn set_aspect_ratio(&mut self, ratio: Option<f32>) {
        let bits = ratio
            .filter(|value| value.is_finite() && *value > 0.0)
            .unwrap_or(0.0);
        self.aspect_ratio.store(bits.to_bits(), Ordering::Release);
    }

    /// Access the OS-level window handle if one exists.
    pub fn handle(&self) -> Option<WindowHandle> {
        self.handle.clone()
    }

    /// Inject one text character into the hosted native window input queue.
    ///
    /// Returns `false` when no native window is open or when posting fails.
    pub fn post_text_char(&self, ch: char) -> bool {
        self.handle
            .as_ref()
            .map(|handle| handle.post_text_char(ch))
            .unwrap_or(false)
    }

    /// Clamp requested open size to at least one pixel per dimension.
    pub(super) fn normalize_open_size(size: Size) -> Size {
        Size {
            width: size.width.max(1),
            height: size.height.max(1),
        }
    }

    /// Persist the latest requested logical size for host polling.
    pub(super) fn record_last_size(&self, size: Size) {
        self.last_size
            .store(pack_size(size.width, size.height), Ordering::Release);
    }
}
