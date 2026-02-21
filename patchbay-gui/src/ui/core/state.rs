
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
    /// Scroll offset for the currently open dropdown menu.
    open_dropdown_scroll_px: i32,
    /// Whether the currently open dropdown was rendered this frame.
    open_dropdown_seen_this_frame: bool,
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
    /// Per-textbox edit cursor/selection runtime keyed by textbox id.
    text_edit_runtime: HashMap<WidgetId, TextEditRuntimeState>,
    /// Per-curve-editor runtime keyed by curve-editor widget id.
    curve_editor_runtime: HashMap<WidgetId, CurveEditorRuntimeState>,
}

/// Cached container sizes for auto layout.
#[derive(Debug, Default)]
struct LayoutState {
    /// Last measured size for each keyed layout container.
    sizes: HashMap<WidgetId, Size>,
}

/// Runtime cursor/selection state for editable text boxes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextEditRuntimeState {
    /// Current cursor index in character units.
    pub(crate) cursor: usize,
    /// Selection anchor index in character units.
    pub(crate) anchor: usize,
    /// True while a pointer-initiated text selection drag is active.
    pub(crate) pointer_selecting: bool,
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
        self.open_dropdown_seen_this_frame = false;
    }

    /// Store the latest measured root frame size for host integrations.
    pub(crate) fn set_root_frame_size(&mut self, size: Size) {
        self.root_frame_used = true;
        self.root_frame_size = Some(size);
    }

    #[cfg_attr(not(any(target_os = "windows", test)), allow(dead_code))]
    /// Take and clear the most recent root frame size.
    pub(crate) fn take_root_frame_size(&mut self) -> Option<Size> {
        self.root_frame_size.take()
    }

    /// Clear the currently open dropdown state.
    pub(crate) fn clear_open_dropdown(&mut self) {
        self.open_dropdown = None;
        self.open_dropdown_scroll_px = 0;
        self.open_dropdown_seen_this_frame = false;
    }

    /// Mark the open dropdown as present in the current render traversal.
    pub(crate) fn mark_open_dropdown_seen(&mut self, id: WidgetId) {
        if self.open_dropdown == Some(id) {
            self.open_dropdown_seen_this_frame = true;
        }
    }

    /// Return whether the currently open dropdown was rendered this frame.
    pub(crate) fn open_dropdown_was_seen(&self) -> bool {
        self.open_dropdown_seen_this_frame
    }
}

/// Deferred dropdown overlay drawing data.
#[derive(Clone, Debug)]
struct DropdownOverlay {
    /// Rectangle of the closed dropdown control.
    base_rect: Rect,
    /// Root-clamped menu viewport rectangle.
    menu_rect: Rect,
    /// Option labels rendered in the overlay menu.
    options: Vec<String>,
    /// Option index currently hovered by the pointer.
    hovered: Option<usize>,
    /// Option index currently selected in the dropdown model.
    selected: usize,
    /// Whether overlay options render upward instead of downward.
    open_up: bool,
    /// Scroll offset in pixels for menu content.
    scroll_px: i32,
    /// Single option row height in pixels.
    row_height: i32,
    /// Optional scrollbar geometry when menu content overflows.
    scrollbar: Option<DropdownOverlayScrollbar>,
    /// Background fill color for menu options.
    fill_color: Color,
    /// Hover fill color for menu options.
    hover_fill_color: Color,
    /// Optional selected-option fill color for menu options.
    selected_fill_color: Option<Color>,
    /// Outline color for menu options.
    outline_color: Color,
    /// Text color for menu options.
    text_color: Color,
}

/// Precomputed scrollbar geometry for a dropdown overlay menu.
#[derive(Clone, Copy, Debug)]
struct DropdownOverlayScrollbar {
    /// Scrollbar track bounds.
    track_rect: Rect,
    /// Scrollbar thumb bounds.
    thumb_rect: Rect,
}
