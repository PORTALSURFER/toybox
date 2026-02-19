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
#[cfg_attr(target_os = "windows", allow(dead_code))]
#[derive(Clone, Copy)]
pub(crate) struct DropdownMenuInteraction {
    /// Whether the dropdown remains open after processing input.
    pub(crate) open: bool,
    /// Whether a new option was selected.
    pub(crate) changed: bool,
    /// Hovered option index in the open menu, if any.
    pub(crate) hovered_index: Option<usize>,
    /// Whether the menu is rendered above the control.
    pub(crate) open_up: bool,
    /// Snapshot of mouse-pressed state for this pass.
    pub(crate) pressed: bool,
}

/// Geometry required to evaluate option hit-testing.
#[cfg_attr(target_os = "windows", allow(dead_code))]
#[derive(Clone, Copy)]
pub(crate) struct DropdownMenuGeometry {
    /// Dropdown control rectangle.
    pub(crate) rect: Rect,
    /// Single-option row height in pixels.
    pub(crate) control_height: i32,
    /// Whether menu rows are placed upward.
    pub(crate) open_up: bool,
}
