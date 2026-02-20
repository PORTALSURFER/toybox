/// Layout result for a dropdown control render pass.
#[derive(Clone, Copy)]
pub(crate) struct DropdownLayout {
    /// Computed block footprint (label + control).
    pub(crate) block_size: Size,
    /// Control rectangle used for interaction and drawing.
    pub(crate) rect: Rect,
    /// Control height in pixels.
    pub(crate) control_height: i32,
}

/// Persistent interaction result for an open dropdown menu pass.
#[derive(Clone, Copy)]
pub(crate) struct DropdownMenuInteraction {
    /// Whether the dropdown remains open after processing input.
    pub(crate) open: bool,
    /// Whether a new option was selected.
    pub(crate) changed: bool,
    /// Hovered option index in the open menu, if any.
    pub(crate) hovered_index: Option<usize>,
    /// Resolved menu viewport geometry.
    pub(crate) geometry: DropdownMenuGeometry,
    /// Snapshot of mouse-pressed state for this pass.
    pub(crate) pressed: bool,
}

/// Geometry required to evaluate option hit-testing.
#[derive(Clone, Copy)]
pub(crate) struct DropdownMenuGeometry {
    /// Dropdown control rectangle.
    pub(crate) rect: Rect,
    /// Menu viewport rectangle constrained to root bounds.
    pub(crate) menu_rect: Rect,
    /// Single-option row height in pixels.
    pub(crate) control_height: i32,
    /// Number of options in the menu.
    pub(crate) option_count: usize,
    /// Maximum scroll offset allowed for this menu.
    pub(crate) max_scroll_px: i32,
    /// Current scroll offset in pixels.
    pub(crate) scroll_px: i32,
    /// Whether menu rows are placed upward.
    pub(crate) open_up: bool,
}
