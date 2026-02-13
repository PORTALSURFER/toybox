impl GuiHostWindow {
    /// Set host-resize behavior for this window.
    pub fn set_host_resize_policy(&mut self, policy: HostResizePolicy) {
        log_line_safe(&format!(
            "toybox/gui: set_host_resize_policy policy={policy:?}"
        ));
        self.host_resize_policy = policy;
    }

    /// Disable host-driven resize handling for this window.
    pub fn disable_host_resize(&mut self) {
        self.set_host_resize_policy(HostResizePolicy::Disabled);
    }

    /// Return the canonical host-resize policy for Patchbay CLAP windows.
    ///
    /// Patchbay GUIs are designed to accept host-driven resize requests.
    pub const fn host_resize_enabled(&self) -> bool {
        matches!(self.host_resize_policy, HostResizePolicy::Enabled)
    }

    /// Normalize a host-provided GUI size to Patchbay's non-zero constraints.
    ///
    /// CLAP hosts may report zero during transient resize negotiation; Patchbay
    /// always clamps to at least `1x1`.
    pub fn normalize_host_size(&self, size: GuiSize) -> GuiSize {
        GuiSize {
            width: size.width.max(1),
            height: size.height.max(1),
        }
    }

    /// Resolve the host-adjusted size according to the current resize policy.
    ///
    /// Returns `None` when host-driven resizing is disabled.
    pub fn adjust_host_size(&self, size: GuiSize) -> Option<GuiSize> {
        self.host_resize_enabled()
            .then(|| self.normalize_host_size(size))
    }

    /// Apply a host-driven GUI resize request using Patchbay's policy.
    ///
    /// This keeps resize ownership in Toybox so plugin implementations do not
    /// need per-plugin resize forwarding logic.
    pub fn apply_host_size(&self, size: GuiSize) {
        if !self.host_resize_enabled() {
            log_line_safe("toybox/gui: apply_host_size ignored (resize disabled)");
            return;
        }
        let normalized = self.normalize_host_size(size);
        self.request_resize(normalized.width, normalized.height);
    }
}
