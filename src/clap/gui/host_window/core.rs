/// Wrapper around a Patchbay GUI window for a CLAP editor.
#[derive(Default)]
pub struct GuiHostWindow {
    /// Underlying host window adapter from `patchbay-gui`.
    inner: HostWindow,
    /// Policy controlling host-driven resize behavior.
    host_resize_policy: HostResizePolicy,
}

impl GuiHostWindow {
    /// Set the raw parent handle provided by the host.
    pub fn set_parent(&mut self, parent: RawWindowHandle) {
        log_line_safe("toybox/gui: set_parent");
        self.inner.set_parent(parent);
    }

    /// Return the most recently observed logical size, if any.
    pub fn last_size(&self) -> Option<(u32, u32)> {
        self.inner.last_size()
    }

    /// Return true if a native window has been created.
    pub fn is_open(&self) -> bool {
        self.inner.is_open()
    }

    /// Show the native window if it exists.
    pub fn show(&self) {
        log_line_safe("toybox/gui: show");
        self.inner.show();
    }

    /// Hide the native window if it exists.
    pub fn hide(&self) {
        log_line_safe("toybox/gui: hide");
        self.inner.hide();
    }

    /// Request a logical resize from the GUI thread.
    pub fn request_resize(&self, width: u32, height: u32) {
        log_line_safe(&format!(
            "toybox/gui: request_resize width={} height={}",
            width, height
        ));
        self.inner.request_resize(width, height);
    }

    /// Set an optional aspect ratio for window resizing.
    pub fn set_aspect_ratio(&mut self, ratio: Option<f32>) {
        log_line_safe(&format!("toybox/gui: set_aspect_ratio ratio={ratio:?}"));
        self.inner.set_aspect_ratio(ratio);
    }
}
