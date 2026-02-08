
/// Layout state for sequential widgets.
#[derive(Debug, Clone, Copy)]
pub struct Layout {
    /// Current cursor position in pixels.
    pub cursor: Point,
    /// Width of the layout column.
    pub column_width: i32,
    /// Vertical spacing between widgets.
    pub spacing: i32,
    /// Default knob size in pixels.
    pub knob_size: i32,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            cursor: Point { x: 16, y: 16 },
            column_width: 180,
            spacing: 18,
            knob_size: DEFAULT_KNOB_DIAMETER,
        }
    }
}

/// Styling configuration for panel containers.
#[derive(Clone, Copy, Debug)]
pub struct PanelStyle<'a> {
    /// Optional title rendered in the panel header.
    pub title: Option<&'a str>,
    /// Padding applied to all sides of the panel content.
    pub padding: i32,
    /// Optional background fill color for the panel.
    pub background: Option<Color>,
    /// Optional outline color for the panel.
    pub outline: Option<Color>,
    /// Explicit header height override (in pixels).
    pub header_height: Option<i32>,
}

impl Default for PanelStyle<'_> {
    fn default() -> Self {
        Self {
            title: None,
            padding: 12,
            background: None,
            outline: None,
            header_height: None,
        }
    }
}

/// Styling configuration for root frame containers.
#[derive(Clone, Copy, Debug)]
pub struct RootFrameStyle<'a> {
    /// Optional title rendered in the frame header.
    pub title: Option<&'a str>,
    /// Padding applied to all sides of the frame content.
    pub padding: i32,
    /// Optional background fill color for the frame.
    pub background: Option<Color>,
    /// Optional outline color for the frame.
    pub outline: Option<Color>,
    /// Explicit header height override (in pixels).
    pub header_height: Option<i32>,
}

impl Default for RootFrameStyle<'_> {
    fn default() -> Self {
        Self {
            title: None,
            padding: 12,
            background: None,
            outline: None,
            header_height: None,
        }
    }
}

/// Response metadata from root frame containers.
#[derive(Clone, Copy, Debug)]
pub struct RootFrameResponse {
    /// The outer bounds of the frame.
    pub outer_rect: Rect,
    /// The content rectangle available to children.
    pub content_rect: Rect,
    /// The measured size captured for window sizing.
    pub measured_size: Size,
}

/// Response metadata from panel containers.
#[derive(Clone, Copy, Debug)]
pub struct PanelResponse {
    /// The outer bounds of the panel.
    pub outer_rect: Rect,
    /// The content rectangle available to children.
    pub content_rect: Rect,
    /// The measured size captured for auto layout.
    pub measured_size: Size,
}

/// Specification for grid layouts.
#[derive(Clone, Copy, Debug)]
pub struct GridSpec {
    /// Number of columns in the grid.
    pub columns: i32,
    /// Size of each grid cell.
    pub cell_size: Size,
    /// Gap between grid cells.
    pub gap: i32,
    /// Optional explicit row count.
    pub rows: Option<i32>,
}

/// Response metadata from grid containers.
#[derive(Clone, Copy, Debug)]
pub struct GridResponse {
    /// The bounding rectangle covering all rows and columns used.
    pub bounds_rect: Rect,
    /// Total rows used by the grid.
    pub rows: i32,
    /// Total columns in the grid.
    pub columns: i32,
}

/// Helper context for addressing grid cells.
pub struct GridContext {
    /// Top-left origin of the grid in window coordinates.
    origin: Point,
    /// Grid spacing and cell size specification.
    spec: GridSpec,
    /// Maximum referenced cell index for bounds reporting.
    max_index: i32,
}

impl GridContext {
    /// Create a new grid context at a given origin.
    fn new(origin: Point, spec: GridSpec) -> Self {
        Self {
            origin,
            spec,
            max_index: -1,
        }
    }

    /// Return the rect for a cell at the given linear index.
    pub fn cell_rect(&mut self, index: i32) -> Rect {
        let idx = index.max(0);
        self.max_index = self.max_index.max(idx);
        let col = idx % self.spec.columns.max(1);
        let row = idx / self.spec.columns.max(1);
        self.cell_rect_rc(row, col)
    }

    /// Return the rect for a cell at the given row/column.
    pub fn cell_rect_rc(&mut self, row: i32, col: i32) -> Rect {
        let row = row.max(0);
        let col = col.max(0);
        let x = self.origin.x + col * (self.spec.cell_size.width as i32 + self.spec.gap);
        let y = self.origin.y + row * (self.spec.cell_size.height as i32 + self.spec.gap);
        Rect {
            origin: Point { x, y },
            size: self.spec.cell_size,
        }
    }

    /// Set the UI cursor to the specified cell origin and return its rect.
    pub fn set_cursor_to_cell(&mut self, ui: &mut Ui<'_>, index: i32) -> Rect {
        let rect = self.cell_rect(index);
        ui.set_cursor(rect.origin);
        rect
    }
}
