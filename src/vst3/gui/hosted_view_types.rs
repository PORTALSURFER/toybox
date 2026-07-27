/// GUI contract for reusable host-parented VST3 views backed by Patchbay windows.
#[cfg(any(feature = "gui", feature = "radiant-vst3"))]
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

    /// Forward one VST3 key-down event to the hosted GUI.
    ///
    /// Return `true` when the event was consumed by the plugin UI and should
    /// not be handled by the host.
    fn on_key_down(&self, _key: char16, _key_code: int16, _modifiers: int16) -> bool {
        false
    }

    /// Forward one VST3 key-up event to the hosted GUI.
    ///
    /// Return `true` when the event was consumed by the plugin UI and should
    /// not be handled by the host.
    fn on_key_up(&self, _key: char16, _key_code: int16, _modifiers: int16) -> bool {
        false
    }
}

#[cfg(any(feature = "gui", feature = "radiant-vst3"))]
#[derive(Clone, Copy)]
/// Ordered logical bounds used by embedded-host resize negotiation.
struct SizeBounds {
    /// Inclusive minimum width and height.
    minimum: (i32, i32),
    /// Inclusive maximum width and height.
    maximum: (i32, i32),
}

#[cfg(any(feature = "gui", feature = "radiant-vst3"))]
impl SizeBounds {
    /// Build ordered, non-zero bounds that always contain the default size.
    fn new(default: (i32, i32), minimum: (u32, u32), maximum: (u32, u32)) -> Self {
        let minimum = (
            logical_dimension(minimum.0).min(default.0),
            logical_dimension(minimum.1).min(default.1),
        );
        let maximum = (
            logical_dimension(maximum.0).max(default.0).max(minimum.0),
            logical_dimension(maximum.1).max(default.1).max(minimum.1),
        );
        Self { minimum, maximum }
    }
}

/// Reusable VST3 `IPlugView` implementation for host-parented Patchbay GUIs.
#[cfg(any(feature = "gui", feature = "radiant-vst3"))]
pub struct HostedVst3View<G: Vst3HostedGui> {
    /// Latest host-facing rectangle in plugin coordinates used for resize behavior.
    rect: Cell<ViewRect>,
    /// Tracks whether a native host parent has already been attached.
    attached: Cell<bool>,
    /// Default logical size exposed by `getSize` before host resize negotiation.
    default_size: (i32, i32),
    /// Optional explicit logical size bounds for embedded-host resize negotiation.
    size_bounds: Option<SizeBounds>,
    /// Whether resize operations should preserve a uniform aspect ratio.
    preserve_aspect_ratio: bool,
    /// Whether requested sizes are clamped to the declared minimum when resizing.
    enforce_minimum_size: bool,
    /// GUI instance shared with FFI callbacks and synchronized under a mutex.
    gui: Mutex<G>,
}

#[cfg(any(feature = "gui", feature = "radiant-vst3"))]
impl<G: Vst3HostedGui> HostedVst3View<G> {
    /// Create a new host-parented view with default logical dimensions.
    pub fn new(gui: G, default_width: u32, default_height: u32) -> Self {
        let width = logical_dimension(default_width);
        let height = logical_dimension(default_height);
        Self {
            rect: Cell::new(view_rect(width, height)),
            attached: Cell::new(false),
            default_size: (width, height),
            size_bounds: None,
            preserve_aspect_ratio: true,
            enforce_minimum_size: false,
            gui: Mutex::new(gui),
        }
    }

    /// Declare the supported logical size range for an embedded view.
    ///
    /// Values are normalized to a non-zero ordered range. The default size is
    /// always retained inside the range, so an invalid host contract cannot
    /// make the initial `getSize` result unsatisfiable.
    pub fn with_size_bounds(
        mut self,
        minimum_width: u32,
        minimum_height: u32,
        maximum_width: u32,
        maximum_height: u32,
    ) -> Self {
        self.size_bounds = Some(SizeBounds::new(
            self.default_size,
            (minimum_width, minimum_height),
            (maximum_width, maximum_height),
        ));
        self
    }

    /// Control whether host resize requests preserve a uniform aspect ratio.
    ///
    /// When `false`, VST3 size requests are applied as received.
    /// When `true`, width/height are adjusted to keep the default aspect ratio.
    pub fn preserve_aspect_ratio(mut self, keep_ratio: bool) -> Self {
        self.preserve_aspect_ratio = keep_ratio;
        self
    }

    /// Control whether host resize requests are clamped to the default minimum
    /// size when aspect-ratio preservation is disabled.
    ///
    /// The 1px floor always remains; this flag only controls the default-size
    /// floor for hosts that keep `preserve_aspect_ratio(false)`.
    pub fn enforce_minimum_size(mut self, enforce: bool) -> Self {
        self.enforce_minimum_size = enforce;
        self
    }

    /// Synchronize the cached rectangle from the hosted GUI's latest reported size.
    fn sync_rect_from_gui(&self) {
        let Ok(gui) = self.gui.lock() else {
            return;
        };
        if let Some((width, height)) = gui.last_size() {
            let requested_width = logical_dimension(width);
            let requested_height = logical_dimension(height);
            let (width, height) = self.constrain_uniform_size(requested_width, requested_height);
            self.rect.set(view_rect(width, height));
        }
    }

    /// Return the logical bounds used for resize negotiation.
    fn resize_bounds(&self) -> ((i32, i32), (i32, i32)) {
        if let Some(bounds) = self.size_bounds {
            return (bounds.minimum, bounds.maximum);
        }

        let floor = if self.preserve_aspect_ratio || self.enforce_minimum_size {
            self.default_size
        } else {
            (1, 1)
        };
        (floor, (i32::MAX, i32::MAX))
    }

    /// Compute the width-to-height ratio derived from the default logical size.
    fn uniform_ratio(&self) -> f32 {
        self.default_size.0 as f32 / self.default_size.1.max(1) as f32
    }

    /// Fit a ratio-preserving size to the declared bounds.
    fn fit_uniform_size(&self, preferred: (i32, i32)) -> (i32, i32) {
        let (minimum, maximum) = self.resize_bounds();
        let ratio = self.uniform_ratio();
        let mut width = preferred.0.max(1);
        let mut height = preferred.1.max(1);

        // Adjust the opposite axis after each bound correction. A valid
        // explicit range always contains the default ratio, so this converges
        // to a bounded pair; the fallback protects against rounding edge
        // cases at the integer boundaries.
        for _ in 0..4 {
            if width < minimum.0 {
                width = minimum.0;
                height = ((width as f32) / ratio).round() as i32;
                continue;
            }
            if width > maximum.0 {
                width = maximum.0;
                height = ((width as f32) / ratio).round() as i32;
                continue;
            }
            if height < minimum.1 {
                height = minimum.1;
                width = ((height as f32) * ratio).round() as i32;
                continue;
            }
            if height > maximum.1 {
                height = maximum.1;
                width = ((height as f32) * ratio).round() as i32;
                continue;
            }
            return (width.max(1), height.max(1));
        }

        (
            self.default_size.0.clamp(minimum.0, maximum.0),
            self.default_size.1.clamp(minimum.1, maximum.1),
        )
    }

    /// Constrain a requested resize while preserving aspect ratio and minimum size.
    fn constrain_uniform_size(&self, requested_width: i32, requested_height: i32) -> (i32, i32) {
        let ((min_width, min_height), (max_width, max_height)) = self.resize_bounds();
        let ratio = self.uniform_ratio();
        let clamped_width = requested_width.clamp(min_width, max_width);
        let clamped_height = requested_height.clamp(min_height, max_height);
        if !self.preserve_aspect_ratio {
            return (clamped_width, clamped_height);
        }
        let current = self.rect.get();
        let current_width = (current.right - current.left).max(1);
        let current_height = (current.bottom - current.top).max(1);
        let width_delta = (clamped_width - current_width).abs();
        let height_delta = (clamped_height - current_height).abs();

        // Keep a single resize path by default (width-driven) to prevent
        // branch switching while dragging. Use height-driven sizing when the
        // host is resizing mostly vertically (or at least not changing width).
        let preferred = if width_delta <= 1 && height_delta > 0 {
            let width = ((clamped_height as f32) * ratio).round() as i32;
            (width, clamped_height)
        } else {
            let height = ((clamped_width as f32) / ratio).round() as i32;
            (clamped_width, height)
        };
        self.fit_uniform_size(preferred)
    }
}

#[cfg(any(feature = "gui", feature = "radiant-vst3"))]
/// Convert a host-provided logical dimension to a positive VST3 coordinate.
fn logical_dimension(value: u32) -> i32 {
    value.clamp(1, i32::MAX as u32) as i32
}

#[cfg(any(feature = "gui", feature = "radiant-vst3"))]
impl<G: Vst3HostedGui> Class for HostedVst3View<G> {
    type Interfaces = (IPlugView,);
}
