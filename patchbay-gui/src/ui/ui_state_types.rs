
/// Input and stateful data for widget interaction.
#[derive(Debug, Default)]
pub struct UiState {
    /// Currently active widget id (pressed/dragging).
    active: Option<WidgetId>,
    /// Widget id currently under pointer hover.
    hot: Option<WidgetId>,
    /// Pointer position at the start of a drag gesture.
    drag_start: Option<Point>,
    /// Value captured when dragging begins.
    drag_value: f32,
    /// Dropdown currently opened this frame, if any.
    open_dropdown: Option<WidgetId>,
    /// Cached layout measurements keyed by container id.
    layout: LayoutState,
    /// Deferred dropdown overlays to render after widgets.
    overlays: Vec<DropdownOverlay>,
    /// Tracks whether this frame already consumed mouse-press input.
    consume_mouse_pressed: bool,
    /// Most recently measured root frame size.
    root_frame_size: Option<Size>,
    /// Whether root frame sizing was updated this frame.
    root_frame_used: bool,
}

/// Cached container sizes for auto layout.
#[derive(Debug, Default)]
struct LayoutState {
    /// Last measured size for each keyed layout container.
    sizes: HashMap<WidgetId, Size>,
}

impl LayoutState {
    /// Return the cached size for a container id.
    fn get(&self, id: WidgetId) -> Option<Size> {
        self.sizes.get(&id).copied()
    }

    /// Update the cached size for a container id.
    fn set(&mut self, id: WidgetId, size: Size) {
        self.sizes.insert(id, size);
    }
}

impl UiState {
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    /// Start a new frame and reset one-frame root-size tracking.
    pub(crate) fn begin_frame(&mut self) {
        self.root_frame_used = false;
        self.root_frame_size = None;
    }

    /// Store the latest measured root frame size for host integrations.
    pub(crate) fn set_root_frame_size(&mut self, size: Size) {
        self.root_frame_used = true;
        self.root_frame_size = Some(size);
    }

    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    /// Take and clear the most recent root frame size.
    pub(crate) fn take_root_frame_size(&mut self) -> Option<Size> {
        self.root_frame_size.take()
    }
}

/// Deferred dropdown overlay drawing data.
#[derive(Clone, Debug)]
struct DropdownOverlay {
    /// Rectangle of the closed dropdown control.
    base_rect: Rect,
    /// Option labels rendered in the overlay menu.
    options: Vec<String>,
    /// Option index currently hovered by the pointer.
    hovered: Option<usize>,
    /// Whether overlay options render upward instead of downward.
    open_up: bool,
    /// Clip rectangle inherited from the owning layout section.
    clip_rect: Rect,
}
