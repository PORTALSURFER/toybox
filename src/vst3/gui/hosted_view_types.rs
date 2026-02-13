/// GUI contract for reusable host-parented VST3 views backed by Patchbay windows.
#[cfg(feature = "gui")]
pub trait Vst3HostedGui {
    /// Attach the host-provided raw parent window handle.
    fn set_parent_raw(&mut self, parent: RawWindowHandle);

    /// Open the GUI for the already configured host parent.
    fn open(&mut self) -> bool;

    /// Close the GUI if it is currently open.
    fn close(&mut self);

    /// Return the latest known GUI logical size.
    fn last_size(&self) -> Option<(u32, u32)>;

    /// Apply a host-provided GUI size to the hosted native view.
    ///
    /// Implementations should treat this as a local view update path and must
    /// avoid host-callback feedback loops.
    fn request_resize(&self, width: u32, height: u32);
}

/// Reusable VST3 `IPlugView` implementation for host-parented Patchbay GUIs.
#[cfg(feature = "gui")]
pub struct HostedVst3View<G: Vst3HostedGui> {
    /// Latest host-facing rectangle in plugin coordinates used for resize behavior.
    rect: Cell<ViewRect>,
    /// Tracks whether a native host parent has already been attached.
    attached: Cell<bool>,
    /// Minimum logical size exposed by `getSize` and size-constraint fallbacks.
    default_size: (i32, i32),
    /// GUI instance shared with FFI callbacks and synchronized under a mutex.
    gui: Mutex<G>,
}

#[cfg(feature = "gui")]
impl<G: Vst3HostedGui> HostedVst3View<G> {
    /// Create a new host-parented view with default logical dimensions.
    pub fn new(gui: G, default_width: u32, default_height: u32) -> Self {
        let width = default_width.max(1) as i32;
        let height = default_height.max(1) as i32;
        Self {
            rect: Cell::new(view_rect(width, height)),
            attached: Cell::new(false),
            default_size: (width, height),
            gui: Mutex::new(gui),
        }
    }

    /// Synchronize the cached rectangle from the hosted GUI's latest reported size.
    fn sync_rect_from_gui(&self) {
        let Ok(gui) = self.gui.lock() else {
            return;
        };
        if let Some((width, height)) = gui.last_size() {
            self.rect.set(view_rect(width as i32, height as i32));
        }
    }

    /// Return the minimum logical size used as the default resize floor.
    fn minimum_size(&self) -> (i32, i32) {
        self.default_size
    }

    /// Compute the width-to-height ratio derived from the default logical size.
    fn uniform_ratio(&self) -> f32 {
        self.default_size.0 as f32 / self.default_size.1.max(1) as f32
    }

    /// Constrain a requested resize while preserving aspect ratio and minimum size.
    fn constrain_uniform_size(&self, requested_width: i32, requested_height: i32) -> (i32, i32) {
        let (min_width, min_height) = self.minimum_size();
        let ratio = self.uniform_ratio();
        let clamped_width = requested_width.max(min_width).max(1);
        let clamped_height = requested_height.max(min_height).max(1);
        let current = self.rect.get();
        let current_width = (current.right - current.left).max(1);
        let current_height = (current.bottom - current.top).max(1);
        let width_delta = (clamped_width - current_width).abs();
        let height_delta = (clamped_height - current_height).abs();

        // Keep a single resize path by default (width-driven) to prevent
        // branch switching while dragging. Only use height-driven sizing when
        // the host is clearly performing a vertical-only resize gesture.
        if width_delta <= 1 && height_delta > 1 {
            let width = ((clamped_height as f32) * ratio).round() as i32;
            (
                width.max(min_width).max(1),
                clamped_height.max(min_height).max(1),
            )
        } else {
            let height = ((clamped_width as f32) / ratio).round() as i32;
            (
                clamped_width.max(min_width).max(1),
                height.max(min_height).max(1),
            )
        }
    }
}

#[cfg(feature = "gui")]
impl<G: Vst3HostedGui> Class for HostedVst3View<G> {
    type Interfaces = (IPlugView,);
}
