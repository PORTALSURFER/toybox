//! Widget rendering and interaction state for the Patchbay GUI.

use std::collections::HashMap;

use crate::canvas::{Canvas, Color, Point, Rect, Size};
use crate::host::InputState;

/// Default rendered knob diameter in pixels for declarative and immediate UIs.
pub(crate) const DEFAULT_KNOB_DIAMETER: i32 = 64;
/// Horizontal padding reserved around knob circles and arc rings in block layouts.
const KNOB_BLOCK_SIDE_PADDING: i32 = 8;

/// Unique identifier for widgets across frames.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WidgetId(u64);

impl WidgetId {
    /// Create a widget id from a stable numeric seed.
    pub const fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// Hash a label into a widget id.
    ///
    /// The label must remain stable across frames for correct interaction
    /// tracking. If the label can change (for example, including formatted
    /// values), prefer using a stable key and hashing that instead.
    pub fn from_label(label: &str) -> Self {
        let mut hash = 0xcbf29ce484222325u64;
        for byte in label.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        Self(hash)
    }
}

/// Theme colors for the GUI widgets.
#[derive(Clone, Debug)]
pub struct Theme {
    /// Canvas background color.
    pub background: Color,
    /// Primary text color.
    pub text: Color,
    /// Text scale factor for the bitmap font.
    pub text_scale: u32,
    /// Knob fill color.
    pub knob_fill: Color,
    /// Knob outline color.
    pub knob_outline: Color,
    /// Knob active color.
    pub knob_active: Color,
    /// Knob hover color.
    pub knob_hover: Color,
    /// Knob indicator color.
    pub knob_indicator: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: Color::rgb(18, 19, 22),
            text: Color::rgb(238, 239, 242),
            text_scale: 2,
            knob_fill: Color::rgb(52, 57, 66),
            knob_outline: Color::rgb(88, 94, 104),
            knob_active: Color::rgb(90, 140, 220),
            knob_hover: Color::rgb(110, 130, 170),
            knob_indicator: Color::rgb(240, 240, 240),
        }
    }
}

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

    /// Store the latest measured root frame size for host resize requests.
    pub(crate) fn set_root_frame_size(&mut self, size: Size) {
        self.root_frame_used = true;
        self.root_frame_size = Some(size);
    }

    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    /// Return whether a root frame size was recorded this frame.
    pub(crate) fn root_frame_used(&self) -> bool {
        self.root_frame_used
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
}

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

/// Return the axis-aligned union of two rectangles.
fn rect_union(a: Rect, b: Rect) -> Rect {
    let min_x = a.origin.x.min(b.origin.x);
    let min_y = a.origin.y.min(b.origin.y);
    let max_x = (a.origin.x + a.size.width as i32).max(b.origin.x + b.size.width as i32);
    let max_y = (a.origin.y + a.size.height as i32).max(b.origin.y + b.size.height as i32);
    Rect {
        origin: Point { x: min_x, y: min_y },
        size: Size {
            width: (max_x - min_x).max(0) as u32,
            height: (max_y - min_y).max(0) as u32,
        },
    }
}

/// Measure monospaced bitmap text bounds at the given scale.
fn text_size(text: &str, scale: u32) -> Size {
    let scale = scale.max(1) as i32;
    let mut max_cols = 0i32;
    let mut lines = 1i32;
    let mut current = 0i32;
    for ch in text.chars() {
        if ch == '\n' {
            max_cols = max_cols.max(current);
            current = 0;
            lines += 1;
        } else {
            current += 1;
        }
    }
    max_cols = max_cols.max(current);
    Size {
        width: (max_cols * 6 * scale).max(0) as u32,
        height: (lines * 8 * scale).max(0) as u32,
    }
}

/// Fit a string to a single line, appending ellipsis when truncated.
fn fit_text_single_line_ellipsis(text: &str, max_width: u32, scale: u32) -> String {
    if max_width == 0 {
        return String::new();
    }

    let single_line: String = text
        .chars()
        .map(|ch| if ch == '\n' || ch == '\r' { ' ' } else { ch })
        .collect();
    if text_size(&single_line, scale).width <= max_width {
        return single_line;
    }

    let char_width = 6 * scale.max(1);
    if char_width == 0 {
        return String::new();
    }
    let max_chars = (max_width / char_width) as usize;
    if max_chars == 0 {
        return String::new();
    }
    if max_chars <= 3 {
        return ".".repeat(max_chars);
    }

    let mut fitted: String = single_line.chars().take(max_chars - 3).collect();
    fitted.push_str("...");
    fitted
}

/// Response metadata from knob widgets.
#[derive(Clone, Copy, Debug, Default)]
pub struct KnobResponse {
    /// The knob value changed this frame.
    pub changed: bool,
    /// The pointer is hovering the knob.
    pub hovered: bool,
    /// The knob is actively being dragged.
    pub active: bool,
}

/// Response metadata from slider widgets.
#[derive(Clone, Copy, Debug, Default)]
pub struct SliderResponse {
    /// The slider value changed this frame.
    pub changed: bool,
    /// The pointer is hovering the slider.
    pub hovered: bool,
    /// The slider is actively being dragged.
    pub active: bool,
}

/// Response metadata from toggle widgets.
#[derive(Clone, Copy, Debug, Default)]
pub struct ToggleResponse {
    /// The toggle value changed this frame.
    pub changed: bool,
    /// The pointer is hovering the toggle.
    pub hovered: bool,
}

/// Response metadata from button widgets.
#[derive(Clone, Copy, Debug, Default)]
pub struct ButtonResponse {
    /// The button was clicked this frame.
    pub clicked: bool,
    /// The pointer is hovering the button.
    pub hovered: bool,
}

/// Response metadata from custom region widgets.
#[derive(Clone, Copy, Debug, Default)]
pub struct RegionResponse {
    /// The pointer is hovering the region.
    pub hovered: bool,
    /// Pointer position relative to region bounds.
    pub local_pointer: Point,
    /// Pointer position relative to region origin without bounds clamping.
    pub raw_local_pointer: Point,
    /// Whether Alt was held during this frame.
    pub alt_down: bool,
    /// The region is actively being dragged.
    pub active: bool,
    /// The primary button was pressed on the region.
    pub pressed: bool,
    /// The primary button was released on the region.
    pub released: bool,
    /// The pointer is being dragged while active.
    pub dragged: bool,
    /// The secondary button was clicked on the region.
    pub secondary_clicked: bool,
    /// The primary button was double-clicked on the region.
    pub double_clicked: bool,
}

/// Response metadata from dropdown widgets.
#[derive(Clone, Copy, Debug, Default)]
pub struct DropdownResponse {
    /// The selection changed this frame.
    pub changed: bool,
    /// The dropdown is open this frame.
    pub open: bool,
    /// The pointer is hovering the dropdown control.
    pub hovered: bool,
}

/// UI frame context used to draw widgets and handle input.
pub struct Ui<'a> {
    /// Destination canvas for all widget drawing.
    canvas: &'a mut Canvas,
    /// Immutable input snapshot for the current frame.
    input: &'a InputState,
    /// Mutable per-window UI interaction state.
    state: &'a mut UiState,
    /// Sequential layout cursor and sizing configuration.
    layout: &'a mut Layout,
    /// Theme colors and typography values.
    theme: &'a Theme,
    /// Saved layout scopes used by `with_layout`.
    layout_stack: Vec<Layout>,
    /// Nested bounds tracking stack for auto-size containers.
    bounds_stack: Vec<Option<Rect>>,
}

impl<'a> Ui<'a> {
    /// Create a UI frame tied to the given canvas and input snapshot.
    pub fn new(
        canvas: &'a mut Canvas,
        input: &'a InputState,
        state: &'a mut UiState,
        layout: &'a mut Layout,
        theme: &'a Theme,
    ) -> Self {
        Self {
            canvas,
            input,
            state,
            layout,
            theme,
            layout_stack: Vec::new(),
            bounds_stack: Vec::new(),
        }
    }

    /// Access the current layout cursor.
    pub fn cursor(&self) -> Point {
        self.layout.cursor
    }

    /// Set the layout cursor position.
    pub fn set_cursor(&mut self, cursor: Point) {
        self.layout.cursor = cursor;
    }

    /// Advance the cursor vertically.
    pub fn advance_y(&mut self, amount: i32) {
        self.layout.cursor.y += amount;
    }

    /// Draw a label at the given position.
    pub fn text(&mut self, position: Point, text: &str) {
        self.canvas
            .draw_text(position, text, self.theme.text, self.theme.text_scale);
        let size = text_size(text, self.theme.text_scale);
        self.track_rect_internal(Rect {
            origin: position,
            size,
        });
    }

    /// Draw a label at the given position with a custom color.
    pub fn text_with_color(&mut self, position: Point, text: &str, color: Color) {
        self.canvas
            .draw_text(position, text, color, self.theme.text_scale);
        let size = text_size(text, self.theme.text_scale);
        self.track_rect_internal(Rect {
            origin: position,
            size,
        });
    }

    /// Access the input snapshot for this frame.
    pub fn input(&self) -> &InputState {
        self.input
    }

    /// Return the key pressed this frame, if any.
    pub fn key_pressed(&self) -> Option<char> {
        self.input.key_pressed
    }

    /// Access the canvas for custom drawing.
    pub fn canvas(&mut self) -> &mut Canvas {
        self.canvas
    }

    /// Access the current theme settings.
    pub fn theme(&self) -> &Theme {
        self.theme
    }

    /// Access the layout for custom sizing.
    pub fn layout_mut(&mut self) -> &mut Layout {
        self.layout
    }

    /// Return the rendered footprint for a knob block at the current theme scale.
    ///
    /// The name and value labels are single-line and clamped to the knob block
    /// width, and the width includes ring/indicator padding so adjacent knobs in
    /// grid layouts do not overlap visually or interactively.
    pub fn knob_block_size(&self, _name_label: &str, _value_label: &str) -> Size {
        let knob_diameter = self.layout.knob_size.max(1) as u32;
        let label_height = 8 * self.theme.text_scale.max(1);
        let label_gap = 4 * self.theme.text_scale.max(1);
        let padded_width = knob_diameter + (KNOB_BLOCK_SIDE_PADDING.max(0) * 2) as u32;
        Size {
            width: padded_width,
            height: knob_diameter + label_height * 2 + label_gap * 2,
        }
    }

    /// Return the rendered footprint for a slider block.
    ///
    /// The label is rendered above the control and clamped to control width.
    pub fn slider_block_size(&self, label: &str, control_size: Size) -> Size {
        let control = Size {
            width: control_size.width.max(1),
            height: control_size.height.max(1),
        };
        let label_height = if label.is_empty() {
            0
        } else {
            8 * self.theme.text_scale.max(1)
        };
        Size {
            width: control.width,
            height: control.height + label_height,
        }
    }

    /// Return the rendered footprint for a toggle block.
    ///
    /// The label is rendered above the control and clamped to control width.
    pub fn toggle_block_size(&self, label: &str, control_size: Size) -> Size {
        let control = Size {
            width: control_size.width.max(1),
            height: control_size.height.max(1),
        };
        let label_height = if label.is_empty() {
            0
        } else {
            8 * self.theme.text_scale.max(1)
        };
        Size {
            width: control.width,
            height: control.height + label_height,
        }
    }

    /// Return the rendered footprint for a dropdown block.
    ///
    /// The label is rendered above the control and clamped to control width.
    pub fn dropdown_block_size(&self, label: &str, control_size: Size) -> Size {
        let control = Size {
            width: control_size.width.max(1),
            height: control_size.height.max(1),
        };
        let label_height = if label.is_empty() {
            0
        } else {
            8 * self.theme.text_scale.max(1)
        };
        Size {
            width: control.width,
            height: control.height + label_height,
        }
    }

    /// Return the rendered footprint for a button block.
    ///
    /// Button labels are rendered inside the provided control size.
    pub fn button_block_size(&self, _label: &str, control_size: Size) -> Size {
        Size {
            width: control_size.width.max(1),
            height: control_size.height.max(1),
        }
    }

    /// Run a closure with a temporary layout origin.
    pub fn with_layout<F>(&mut self, origin: Point, mut f: F)
    where
        F: FnMut(&mut Ui<'_>),
    {
        let previous = *self.layout;
        self.layout_stack.push(previous);
        self.layout.cursor = origin;
        f(self);
        if let Some(restored) = self.layout_stack.pop() {
            *self.layout = restored;
        }
    }

    /// Queue a dropdown overlay for deferred draw order.
    fn push_dropdown_overlay(
        &mut self,
        base_rect: Rect,
        options: &[&str],
        hovered: Option<usize>,
        open_up: bool,
    ) {
        self.state.overlays.push(DropdownOverlay {
            base_rect,
            options: options.iter().map(|option| (*option).to_string()).collect(),
            hovered,
            open_up,
        });
    }

    /// Draw any deferred overlays (dropdown menus).
    pub fn draw_overlays(&mut self) {
        for overlay in self.state.overlays.iter() {
            let rect = overlay.base_rect;
            let height = rect.size.height as i32;
            for (index, option) in overlay.options.iter().enumerate() {
                let option_rect = Rect {
                    origin: Point {
                        x: rect.origin.x,
                        y: if overlay.open_up {
                            rect.origin.y - height * (index as i32 + 1)
                        } else {
                            rect.origin.y + height * (index as i32 + 1)
                        },
                    },
                    size: rect.size,
                };
                let option_fill = if overlay.hovered == Some(index) {
                    self.theme.knob_hover
                } else {
                    self.theme.knob_fill
                };
                self.canvas.fill_rect(option_rect, option_fill);
                self.canvas
                    .stroke_rect(option_rect, 1, self.theme.knob_outline);
                let option_text = Point {
                    x: option_rect.origin.x + 4,
                    y: option_rect.origin.y + (height - (7 * self.theme.text_scale as i32)) / 2,
                };
                let fitted = fit_text_single_line_ellipsis(
                    option,
                    option_rect.size.width.saturating_sub(8),
                    self.theme.text_scale,
                );
                self.canvas
                    .draw_text(option_text, &fitted, self.theme.text, self.theme.text_scale);
            }
        }
    }

    /// Clear any deferred overlay drawings for the next frame.
    pub fn clear_overlays(&mut self) {
        self.state.overlays.clear();
    }

    /// Reset per-frame input consumption flags.
    pub fn reset_input_consumption(&mut self) {
        self.state.consume_mouse_pressed = false;
    }

    /// Return true when a primary-button press is still available this frame.
    fn mouse_pressed(&self) -> bool {
        self.input.mouse_pressed && !self.state.consume_mouse_pressed
    }

    /// Consume the frame's primary-button press.
    fn consume_mouse_pressed(&mut self) {
        self.state.consume_mouse_pressed = true;
    }

    /// Draw a single-line label with ellipsis truncation.
    fn draw_text_single_line_clamped(
        &mut self,
        origin: Point,
        text: &str,
        max_width: u32,
        color: Color,
        track_bounds: bool,
    ) -> Size {
        let fitted = fit_text_single_line_ellipsis(text, max_width, self.theme.text_scale);
        if fitted.is_empty() {
            return Size {
                width: 0,
                height: 0,
            };
        }
        self.canvas
            .draw_text(origin, &fitted, color, self.theme.text_scale);
        let size = text_size(&fitted, self.theme.text_scale);
        if track_bounds {
            self.track_rect_internal(Rect { origin, size });
        }
        size
    }

    /// Push a new empty bounds union for nested layout tracking.
    fn push_bounds(&mut self) {
        self.bounds_stack.push(None);
    }

    /// Pop the current bounds union.
    fn pop_bounds(&mut self) -> Option<Rect> {
        self.bounds_stack.pop().flatten()
    }

    /// Merge a rendered rectangle into the current bounds union.
    fn track_rect_internal(&mut self, rect: Rect) {
        if let Some(entry) = self.bounds_stack.last_mut() {
            *entry = Some(match *entry {
                Some(existing) => rect_union(existing, rect),
                None => rect,
            });
        }
    }

    /// Track a rectangle so container sizing can include custom drawing.
    pub fn track_rect(&mut self, rect: Rect) {
        self.track_rect_internal(rect);
    }

    /// Draw a root frame container sized to its contents.
    ///
    /// Root frames are the top-level container for a window. The measured size
    /// is stored for auto-resizing the native window each frame. When `size`
    /// is provided, it is treated as the pre-measured content size.
    pub fn root_frame_with_key<F>(
        &mut self,
        key: &str,
        style: RootFrameStyle<'_>,
        size: Option<Size>,
        mut f: F,
    ) -> RootFrameResponse
    where
        F: FnMut(&mut Ui<'_>, Rect),
    {
        self.root_frame_with_key_at(key, style, size, Point { x: 0, y: 0 }, |ui, rect| {
            f(ui, rect);
        })
    }

    /// Draw a root frame at an explicit origin.
    ///
    /// The measured root size is still reported for host auto-resize.
    pub fn root_frame_with_key_at<F>(
        &mut self,
        key: &str,
        style: RootFrameStyle<'_>,
        size: Option<Size>,
        origin: Point,
        mut f: F,
    ) -> RootFrameResponse
    where
        F: FnMut(&mut Ui<'_>, Rect),
    {
        let id = WidgetId::from_label(key);
        let header_height = style.header_height.unwrap_or_else(|| {
            if style.title.is_some() {
                (8 * self.theme.text_scale as i32 + 4).max(0)
            } else {
                0
            }
        });
        let padding = style.padding.max(0);
        let fallback = Size {
            width: (padding * 2 + 160).max(0) as u32,
            height: (padding * 2 + header_height + 80).max(0) as u32,
        };
        let requested_size = size;
        let cached = self.state.layout.get(id);
        let size = requested_size.or(cached).unwrap_or(fallback);
        let outer_rect = Rect { origin, size };
        let background = style.background.unwrap_or(self.theme.knob_fill);
        let outline = style.outline.unwrap_or(self.theme.knob_outline);

        self.canvas.fill_rect(outer_rect, background);
        self.canvas.stroke_rect(outer_rect, 1, outline);

        if let Some(title) = style.title {
            let title_pos = Point {
                x: origin.x + padding,
                y: origin.y + padding,
            };
            self.canvas
                .draw_text(title_pos, title, self.theme.text, self.theme.text_scale);
            let title_size = text_size(title, self.theme.text_scale);
            self.track_rect_internal(Rect {
                origin: title_pos,
                size: title_size,
            });
        }

        let content_origin = Point {
            x: origin.x + padding,
            y: origin.y + padding + header_height,
        };
        let content_rect = Rect {
            origin: content_origin,
            size: Size {
                width: size.width.saturating_sub((padding * 2) as u32),
                height: size
                    .height
                    .saturating_sub((padding * 2 + header_height) as u32),
            },
        };

        self.push_bounds();
        self.with_layout(content_origin, |ui| f(ui, content_rect));
        let measured_bounds = self.pop_bounds();

        let measured_size = if let Some(bounds) = measured_bounds {
            let max_x = bounds.origin.x + bounds.size.width as i32;
            let max_y = bounds.origin.y + bounds.size.height as i32;
            let content_width = (max_x - content_origin.x).max(0) as u32;
            let content_height = (max_y - content_origin.y).max(0) as u32;
            Size {
                width: content_width + (padding * 2) as u32,
                height: content_height + (padding * 2 + header_height) as u32,
            }
        } else {
            Size {
                width: (padding * 2) as u32,
                height: (padding * 2 + header_height) as u32,
            }
        };
        let measured_size = match requested_size {
            Some(explicit) => Size {
                width: explicit.width.max(measured_size.width),
                height: explicit.height.max(measured_size.height),
            },
            None => measured_size,
        };

        self.state.layout.set(id, measured_size);
        self.track_rect_internal(outer_rect);
        self.state.set_root_frame_size(measured_size);

        RootFrameResponse {
            outer_rect,
            content_rect,
            measured_size,
        }
    }

    /// Draw a root frame with a stable default key.
    pub fn root_frame<F>(&mut self, style: RootFrameStyle<'_>, f: F) -> RootFrameResponse
    where
        F: FnMut(&mut Ui<'_>, Rect),
    {
        self.root_frame_with_key("__root_frame__", style, None, f)
    }

    /// Draw a panel container with an optional title and padding.
    ///
    /// The panel can auto-size to fit its contents. When `size` is `None`, the
    /// panel uses the last measured size for the key and updates it after the
    /// closure runs. Auto-sized panels advance the layout cursor using the
    /// newly measured height so following widgets line up with the rendered
    /// content.
    pub fn panel_with_key<F>(
        &mut self,
        key: &str,
        style: PanelStyle<'_>,
        size: Option<Size>,
        mut f: F,
    ) -> PanelResponse
    where
        F: FnMut(&mut Ui<'_>, Rect),
    {
        let id = WidgetId::from_label(key);
        let header_height = style.header_height.unwrap_or_else(|| {
            if style.title.is_some() {
                (8 * self.theme.text_scale as i32 + 4).max(0)
            } else {
                0
            }
        });
        let padding = style.padding.max(0);
        let fallback = Size {
            width: (padding * 2 + 160).max(0) as u32,
            height: (padding * 2 + header_height + 80).max(0) as u32,
        };
        let requested_size = size;
        let cached = self.state.layout.get(id);
        let size = requested_size.or(cached).unwrap_or(fallback);
        let origin = self.layout.cursor;
        let outer_rect = Rect { origin, size };
        let background = style.background.unwrap_or(self.theme.knob_fill);
        let outline = style.outline.unwrap_or(self.theme.knob_outline);

        self.canvas.fill_rect(outer_rect, background);
        self.canvas.stroke_rect(outer_rect, 1, outline);

        if let Some(title) = style.title {
            let title_pos = Point {
                x: origin.x + padding,
                y: origin.y + padding,
            };
            self.canvas
                .draw_text(title_pos, title, self.theme.text, self.theme.text_scale);
            let title_size = text_size(title, self.theme.text_scale);
            self.track_rect_internal(Rect {
                origin: title_pos,
                size: title_size,
            });
        }

        let content_origin = Point {
            x: origin.x + padding,
            y: origin.y + padding + header_height,
        };
        let content_rect = Rect {
            origin: content_origin,
            size: Size {
                width: size.width.saturating_sub((padding * 2) as u32),
                height: size
                    .height
                    .saturating_sub((padding * 2 + header_height) as u32),
            },
        };

        self.push_bounds();
        self.with_layout(content_origin, |ui| f(ui, content_rect));
        let measured_bounds = self.pop_bounds();

        let measured_size = if let Some(bounds) = measured_bounds {
            let max_x = bounds.origin.x + bounds.size.width as i32;
            let max_y = bounds.origin.y + bounds.size.height as i32;
            let content_width = (max_x - content_origin.x).max(0) as u32;
            let content_height = (max_y - content_origin.y).max(0) as u32;
            Size {
                width: content_width + (padding * 2) as u32,
                height: content_height + (padding * 2 + header_height) as u32,
            }
        } else {
            Size {
                width: (padding * 2) as u32,
                height: (padding * 2 + header_height) as u32,
            }
        };

        let measured_size = match requested_size {
            Some(explicit) => Size {
                width: explicit.width.max(measured_size.width),
                height: explicit.height.max(measured_size.height),
            },
            None => measured_size,
        };

        self.state.layout.set(id, measured_size);
        self.track_rect_internal(outer_rect);
        let advance_height = measured_size.height;
        self.layout.cursor.y = origin.y + advance_height as i32 + self.layout.spacing;

        PanelResponse {
            outer_rect,
            content_rect,
            measured_size,
        }
    }

    /// Draw a grid container and provide a helper for addressing cells.
    pub fn grid_with_key<F>(
        &mut self,
        _key: &str,
        spec: GridSpec,
        origin: Point,
        mut f: F,
    ) -> GridResponse
    where
        F: FnMut(&mut Ui<'_>, &mut GridContext),
    {
        let mut ctx = GridContext::new(origin, spec);
        f(self, &mut ctx);

        let rows = spec.rows.unwrap_or_else(|| {
            if ctx.max_index < 0 {
                0
            } else {
                (ctx.max_index / spec.columns.max(1)) + 1
            }
        });
        let columns = spec.columns.max(1);
        let width = if rows == 0 || columns == 0 {
            0
        } else {
            columns * spec.cell_size.width as i32 + (columns - 1) * spec.gap
        };
        let height = if rows == 0 || columns == 0 {
            0
        } else {
            rows * spec.cell_size.height as i32 + (rows - 1) * spec.gap
        };
        let bounds_rect = Rect {
            origin,
            size: Size {
                width: width.max(0) as u32,
                height: height.max(0) as u32,
            },
        };
        self.track_rect_internal(bounds_rect);

        GridResponse {
            bounds_rect,
            rows,
            columns,
        }
    }

    /// Draw a label at the current cursor and advance the cursor.
    pub fn label(&mut self, text: &str) {
        let pos = self.layout.cursor;
        let line_height = 8 * self.theme.text_scale as i32;
        self.canvas
            .draw_text(pos, text, self.theme.text, self.theme.text_scale);
        let size = text_size(text, self.theme.text_scale);
        self.track_rect_internal(Rect { origin: pos, size });
        self.layout.cursor.y += line_height + self.layout.spacing;
    }

    /// Create an interactive region for custom drawing.
    ///
    /// Use this to capture hover/drag interactions over arbitrary rectangles.
    /// The `key` must be stable across frames.
    pub fn region_with_key(&mut self, key: &str, rect: Rect) -> RegionResponse {
        let id = WidgetId::from_label(key);
        let hovered = rect.contains(self.input.pointer_pos);
        let local_pointer = local_pointer_in_rect(self.input.pointer_pos, rect);
        let raw_local_pointer = raw_local_pointer_in_rect(self.input.pointer_pos, rect);
        if hovered {
            self.state.hot = Some(id);
        }

        let mut response = RegionResponse {
            hovered,
            local_pointer,
            raw_local_pointer,
            alt_down: self.input.alt_down,
            active: self.state.active == Some(id),
            pressed: false,
            released: false,
            dragged: false,
            secondary_clicked: false,
            double_clicked: false,
        };

        if hovered && self.mouse_pressed() {
            self.state.active = Some(id);
            self.state.drag_start = Some(self.input.pointer_pos);
            response.active = true;
            response.pressed = true;
        }

        if self.state.active == Some(id) && self.input.mouse_released {
            self.state.active = None;
            self.state.drag_start = None;
            response.active = false;
            response.released = true;
        }

        if self.state.active == Some(id) && self.input.mouse_down {
            response.dragged = true;
        }

        if hovered && self.input.mouse_secondary_pressed {
            response.secondary_clicked = true;
        }

        if hovered && self.input.mouse_double_clicked {
            response.double_clicked = true;
        }

        response
    }

    /// Format a knob value using compact decimal precision.
    fn format_knob_value(value: f32) -> String {
        let mut text = format!("{value:.2}");
        if text.contains('.') {
            while text.ends_with('0') {
                text.pop();
            }
            if text.ends_with('.') {
                text.pop();
            }
        }
        if text == "-0" {
            text = "0".to_string();
        }
        text
    }

    /// Draw a knob with the given name label and value.
    ///
    /// The provided `id` must be stable across frames. If the label changes
    /// while dragging, use [`Ui::knob_with_key`] to provide a separate stable
    /// identifier.
    pub fn knob(
        &mut self,
        id: WidgetId,
        label: &str,
        value: &mut f32,
        range: (f32, f32),
    ) -> KnobResponse {
        let value_label = Self::format_knob_value(*value);
        self.knob_with_labels(id, label, &value_label, value, range)
    }

    /// Draw a knob with a name label above and a value label below.
    ///
    /// Labels are clamped to the knob width with single-line ellipsis to keep
    /// dense manual layouts from overlapping neighboring controls.
    pub fn knob_with_labels(
        &mut self,
        id: WidgetId,
        name_label: &str,
        value_label: &str,
        value: &mut f32,
        range: (f32, f32),
    ) -> KnobResponse {
        let knob_size = self.layout.knob_size.max(1);
        let block_size = self.knob_block_size(name_label, value_label);
        let label_height = 8 * self.theme.text_scale as i32;
        let label_gap = 4 * self.theme.text_scale as i32;
        let knob_x_offset = ((block_size.width as i32 - knob_size) / 2).max(0);
        let knob_origin = Point {
            x: self.layout.cursor.x + knob_x_offset,
            y: self.layout.cursor.y + label_height + label_gap,
        };
        let knob_rect = Rect {
            origin: knob_origin,
            size: Size {
                width: knob_size as u32,
                height: knob_size as u32,
            },
        };
        let hit_rect = Rect {
            origin: Point {
                x: knob_rect.origin.x - KNOB_BLOCK_SIDE_PADDING,
                y: knob_rect.origin.y - KNOB_BLOCK_SIDE_PADDING,
            },
            size: Size {
                width: (knob_size + KNOB_BLOCK_SIDE_PADDING * 2).max(1) as u32,
                height: (knob_size + KNOB_BLOCK_SIDE_PADDING * 2).max(1) as u32,
            },
        };
        self.track_rect_internal(hit_rect);
        let center = Point {
            x: knob_rect.origin.x + knob_size / 2,
            y: knob_rect.origin.y + knob_size / 2,
        };
        let radius = (knob_size / 2 - 4).max(1);
        let hovered = hit_rect.contains(self.input.pointer_pos);
        if hovered {
            self.state.hot = Some(id);
        }

        let mut response = KnobResponse {
            hovered,
            active: self.state.active == Some(id),
            changed: false,
        };

        if hovered && self.mouse_pressed() {
            self.state.active = Some(id);
            self.state.drag_start = Some(self.input.pointer_pos);
            self.state.drag_value = *value;
            response.active = true;
        }

        if self.state.active == Some(id) && self.input.mouse_released {
            self.state.active = None;
            self.state.drag_start = None;
            response.active = false;
        }

        if self.state.active == Some(id)
            && self.input.mouse_down
            && let Some(start) = self.state.drag_start
        {
            let dy = (self.input.pointer_pos.y - start.y) as f32;
            let delta = dy * 0.005 * (range.1 - range.0);
            let new_value = (self.state.drag_value + delta).clamp(range.0, range.1);
            if (*value - new_value).abs() > f32::EPSILON {
                *value = new_value;
                response.changed = true;
            }
        }

        if hovered && self.input.wheel_delta != 0.0 {
            let step = 0.02 * (range.1 - range.0);
            let new_value =
                (*value - step * self.input.wheel_delta.signum()).clamp(range.0, range.1);
            if (*value - new_value).abs() > f32::EPSILON {
                *value = new_value;
                response.changed = true;
            }
        }

        let t = (*value - range.0) / (range.1 - range.0).max(1.0e-6);
        let arc_start = 7.0 * std::f32::consts::PI / 4.0;
        let arc_end = 5.0 * std::f32::consts::PI / 4.0;
        let arc_span = if arc_end < arc_start {
            arc_end + std::f32::consts::TAU - arc_start
        } else {
            arc_end - arc_start
        };
        let angle = arc_start + t * arc_span;
        let indicator = knob_indicator_point(center, radius, angle);

        let fill = if response.active {
            self.theme.knob_active
        } else if hovered {
            self.theme.knob_hover
        } else {
            self.theme.knob_fill
        };

        self.canvas.fill_circle(center, radius, fill);
        self.canvas
            .stroke_circle(center, radius, 2, self.theme.knob_outline);
        let arc_radius = radius + 6;
        let arc_thickness = 3;
        self.canvas.stroke_arc(
            center,
            arc_radius,
            arc_thickness,
            arc_start,
            arc_end,
            self.theme.knob_outline,
        );
        self.canvas.stroke_arc(
            center,
            arc_radius,
            arc_thickness,
            arc_start,
            angle,
            self.theme.knob_indicator,
        );
        self.canvas
            .draw_line(center, indicator, self.theme.knob_indicator);

        let name_pos = Point {
            x: knob_rect.origin.x,
            y: self.layout.cursor.y,
        };
        if !name_label.is_empty() {
            let _ = self.draw_text_single_line_clamped(
                name_pos,
                name_label,
                block_size.width,
                self.theme.text,
                true,
            );
        }

        let value_pos = Point {
            x: knob_rect.origin.x,
            y: knob_rect.origin.y + knob_size + label_gap,
        };
        if !value_label.is_empty() {
            let _ = self.draw_text_single_line_clamped(
                value_pos,
                value_label,
                block_size.width,
                self.theme.text,
                true,
            );
        }

        let block_height = block_size.height as i32;
        self.layout.cursor.y += block_height + self.layout.spacing;
        response
    }

    /// Draw a knob with a stable key and a potentially dynamic name label.
    ///
    /// The `key` should be a stable identifier across frames (for example,
    /// `"attack"`). The `label` can change to include formatted values without
    /// breaking drag tracking.
    ///
    /// # Example
    /// ```ignore
    /// # use patchbay_gui::{Canvas, InputState, Layout, Theme, Ui, UiState};
    /// let mut canvas = Canvas::new(200, 200);
    /// let mut layout = Layout::default();
    /// let theme = Theme::default();
    /// let mut ui_state = UiState::default();
    /// let input = InputState::default();
    /// let mut value = 0.5;
    /// let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
    /// ui.knob_with_key("attack", "Attack 0.50s", &mut value, (0.0, 1.0));
    /// ```
    pub fn knob_with_key(
        &mut self,
        key: &str,
        label: &str,
        value: &mut f32,
        range: (f32, f32),
    ) -> KnobResponse {
        let id = WidgetId::from_label(key);
        self.knob(id, label, value, range)
    }

    /// Draw a knob with explicit name and value labels and a stable key.
    pub fn knob_with_key_labels(
        &mut self,
        key: &str,
        name_label: &str,
        value_label: &str,
        value: &mut f32,
        range: (f32, f32),
    ) -> KnobResponse {
        let id = WidgetId::from_label(key);
        self.knob_with_labels(id, name_label, value_label, value, range)
    }

    /// Draw a horizontal slider with the given label and value.
    ///
    /// The provided `id` must be stable across frames. If the label changes
    /// while dragging, use [`Ui::slider_with_key`] to provide a separate stable
    /// identifier.
    pub fn slider(
        &mut self,
        id: WidgetId,
        label: &str,
        value: &mut f32,
        range: (f32, f32),
        width: i32,
        height: i32,
    ) -> SliderResponse {
        let width = width.max(1);
        let height = height.max(1);
        let control_size = Size {
            width: width as u32,
            height: height as u32,
        };
        let block_size = self.slider_block_size(label, control_size);
        let label_height = 8 * self.theme.text_scale as i32;
        let base = self.layout.cursor;
        let mut rect_origin = base;
        if !label.is_empty() {
            let _ = self.draw_text_single_line_clamped(
                base,
                label,
                control_size.width,
                self.theme.text,
                true,
            );
            rect_origin.y += label_height;
        }

        let rect = Rect {
            origin: rect_origin,
            size: control_size,
        };
        self.track_rect_internal(rect);
        let hovered = rect.contains(self.input.pointer_pos);
        if hovered {
            self.state.hot = Some(id);
        }

        let mut response = SliderResponse {
            hovered,
            active: self.state.active == Some(id),
            changed: false,
        };

        if hovered && self.mouse_pressed() {
            self.state.active = Some(id);
            response.active = true;
        }

        if self.state.active == Some(id) && self.input.mouse_released {
            self.state.active = None;
            response.active = false;
        }

        if self.state.active == Some(id) && self.input.mouse_down {
            let span = (range.1 - range.0).max(1.0e-6);
            let x = (self.input.pointer_pos.x - rect.origin.x) as f32;
            let t = (x / rect.size.width.max(1) as f32).clamp(0.0, 1.0);
            let new_value = range.0 + t * span;
            if (*value - new_value).abs() > f32::EPSILON {
                *value = new_value;
                response.changed = true;
            }
        }

        if hovered && self.input.wheel_delta != 0.0 {
            let span = (range.1 - range.0).max(1.0e-6);
            let step = 0.02 * span;
            let new_value =
                (*value + step * self.input.wheel_delta.signum()).clamp(range.0, range.1);
            if (*value - new_value).abs() > f32::EPSILON {
                *value = new_value;
                response.changed = true;
            }
        }

        let span = (range.1 - range.0).max(1.0e-6);
        let t = ((*value - range.0) / span).clamp(0.0, 1.0);
        let track_height = (height / 4).max(4);
        let track_y = rect.origin.y + (height - track_height) / 2;
        let track_rect = Rect {
            origin: Point {
                x: rect.origin.x,
                y: track_y,
            },
            size: Size {
                width: rect.size.width,
                height: track_height as u32,
            },
        };
        let fill_width = ((rect.size.width as f32) * t).round() as u32;
        let fill_rect = Rect {
            origin: track_rect.origin,
            size: Size {
                width: fill_width,
                height: track_rect.size.height,
            },
        };
        let fill = if response.active {
            self.theme.knob_active
        } else if hovered {
            self.theme.knob_hover
        } else {
            self.theme.knob_fill
        };
        self.canvas.fill_rect(track_rect, fill);
        self.canvas
            .stroke_rect(track_rect, 1, self.theme.knob_outline);
        self.canvas.fill_rect(fill_rect, self.theme.knob_indicator);

        let handle_x = rect.origin.x + (rect.size.width as f32 * t) as i32;
        let handle_center = Point {
            x: handle_x,
            y: rect.origin.y + height / 2,
        };
        let handle_radius = (height / 2).max(3);
        self.canvas
            .fill_circle(handle_center, handle_radius, self.theme.knob_indicator);

        self.layout.cursor.y = rect.origin.y + block_size.height as i32 + self.layout.spacing;
        response
    }

    /// Draw a horizontal slider with a stable key and a dynamic label.
    pub fn slider_with_key(
        &mut self,
        key: &str,
        label: &str,
        value: &mut f32,
        range: (f32, f32),
        width: i32,
        height: i32,
    ) -> SliderResponse {
        let id = WidgetId::from_label(key);
        self.slider(id, label, value, range, width, height)
    }

    /// Draw a toggle switch with the given label and value.
    pub fn toggle(
        &mut self,
        id: WidgetId,
        label: &str,
        value: &mut bool,
        width: i32,
        height: i32,
    ) -> ToggleResponse {
        let width = width.max(1);
        let height = height.max(1);
        let control_size = Size {
            width: width as u32,
            height: height as u32,
        };
        let block_size = self.toggle_block_size(label, control_size);
        let label_height = 8 * self.theme.text_scale as i32;
        let base = self.layout.cursor;
        let mut rect_origin = base;
        if !label.is_empty() {
            let _ = self.draw_text_single_line_clamped(
                base,
                label,
                control_size.width,
                self.theme.text,
                true,
            );
            rect_origin.y += label_height;
        }
        let rect = Rect {
            origin: rect_origin,
            size: control_size,
        };
        self.track_rect_internal(rect);
        let hovered = rect.contains(self.input.pointer_pos);
        if hovered {
            self.state.hot = Some(id);
        }
        let mut response = ToggleResponse {
            hovered,
            changed: false,
        };
        if hovered && self.mouse_pressed() {
            *value = !*value;
            response.changed = true;
        }
        let fill = if *value {
            self.theme.knob_indicator
        } else if hovered {
            self.theme.knob_hover
        } else {
            self.theme.knob_fill
        };
        self.canvas.fill_rect(rect, fill);
        self.canvas.stroke_rect(rect, 1, self.theme.knob_outline);

        let thumb_radius = (height / 2).max(3);
        let thumb_x = if *value {
            rect.origin.x + width - thumb_radius
        } else {
            rect.origin.x + thumb_radius
        };
        let thumb_center = Point {
            x: thumb_x,
            y: rect.origin.y + height / 2,
        };
        self.canvas
            .fill_circle(thumb_center, thumb_radius, self.theme.knob_outline);

        self.layout.cursor.y = rect.origin.y + block_size.height as i32 + self.layout.spacing;
        response
    }

    /// Draw a toggle switch with a stable key and a dynamic label.
    pub fn toggle_with_key(
        &mut self,
        key: &str,
        label: &str,
        value: &mut bool,
        width: i32,
        height: i32,
    ) -> ToggleResponse {
        let id = WidgetId::from_label(key);
        self.toggle(id, label, value, width, height)
    }

    /// Draw a button with the given label.
    pub fn button(&mut self, id: WidgetId, label: &str, width: i32, height: i32) -> ButtonResponse {
        let width = width.max(1);
        let height = height.max(1);
        let control_size = Size {
            width: width as u32,
            height: height as u32,
        };
        let rect = Rect {
            origin: self.layout.cursor,
            size: control_size,
        };
        self.track_rect_internal(rect);
        let hovered = rect.contains(self.input.pointer_pos);
        if hovered {
            self.state.hot = Some(id);
        }
        let mut response = ButtonResponse {
            hovered,
            clicked: false,
        };
        if hovered && self.mouse_pressed() {
            response.clicked = true;
        }
        let fill = if hovered {
            self.theme.knob_hover
        } else {
            self.theme.knob_fill
        };
        self.canvas.fill_rect(rect, fill);
        self.canvas.stroke_rect(rect, 1, self.theme.knob_outline);
        let text_pos = Point {
            x: rect.origin.x + 4,
            y: rect.origin.y + (height - (7 * self.theme.text_scale as i32)) / 2,
        };
        let _ = self.draw_text_single_line_clamped(
            text_pos,
            label,
            rect.size.width.saturating_sub(8),
            self.theme.text,
            false,
        );

        self.layout.cursor.y = rect.origin.y + height + self.layout.spacing;
        response
    }

    /// Draw a button with a stable key and a dynamic label.
    pub fn button_with_key(
        &mut self,
        key: &str,
        label: &str,
        width: i32,
        height: i32,
    ) -> ButtonResponse {
        let id = WidgetId::from_label(key);
        self.button(id, label, width, height)
    }

    /// Draw a dropdown selector with the given label and options.
    pub fn dropdown(
        &mut self,
        id: WidgetId,
        label: &str,
        options: &[&str],
        selected: &mut usize,
        width: i32,
        height: i32,
    ) -> DropdownResponse {
        let width = width.max(1);
        let height = height.max(1);
        let control_size = Size {
            width: width as u32,
            height: height as u32,
        };
        let block_size = self.dropdown_block_size(label, control_size);
        let label_height = 8 * self.theme.text_scale as i32;
        let base = self.layout.cursor;
        let mut rect_origin = base;
        if !label.is_empty() {
            let _ = self.draw_text_single_line_clamped(
                base,
                label,
                control_size.width,
                self.theme.text,
                true,
            );
            rect_origin.y += label_height;
        }

        let rect = Rect {
            origin: rect_origin,
            size: control_size,
        };
        self.track_rect_internal(rect);
        let hovered = rect.contains(self.input.pointer_pos);
        if hovered {
            self.state.hot = Some(id);
        }
        let mut response = DropdownResponse {
            hovered,
            open: self.state.open_dropdown == Some(id),
            changed: false,
        };

        if hovered && self.mouse_pressed() {
            if response.open {
                self.state.open_dropdown = None;
                response.open = false;
            } else {
                self.state.open_dropdown = Some(id);
                response.open = true;
            }
        }

        let fill = if response.open {
            self.theme.knob_active
        } else if hovered {
            self.theme.knob_hover
        } else {
            self.theme.knob_fill
        };
        self.canvas.fill_rect(rect, fill);
        self.canvas.stroke_rect(rect, 1, self.theme.knob_outline);
        let current = options.get(*selected).copied().unwrap_or("-");
        let text_pos = Point {
            x: rect.origin.x + 4,
            y: rect.origin.y + (height - (7 * self.theme.text_scale as i32)) / 2,
        };
        let _ = self.draw_text_single_line_clamped(
            text_pos,
            current,
            rect.size.width.saturating_sub(8),
            self.theme.text,
            false,
        );

        if response.open {
            let pressed = self.mouse_pressed();
            let mut any_hovered = false;
            let mut hovered_index = None;
            let menu_height = height * options.len() as i32;
            let canvas_height = self.canvas.size().height as i32;
            let open_up = rect.origin.y + height + menu_height > canvas_height
                && rect.origin.y >= menu_height;
            for (index, _option) in options.iter().enumerate() {
                let option_rect = Rect {
                    origin: Point {
                        x: rect.origin.x,
                        y: if open_up {
                            rect.origin.y - height * (index as i32 + 1)
                        } else {
                            rect.origin.y + height * (index as i32 + 1)
                        },
                    },
                    size: rect.size,
                };
                let option_hovered = option_rect.contains(self.input.pointer_pos);
                if option_hovered {
                    any_hovered = true;
                    hovered_index = Some(index);
                }
                if option_hovered && pressed {
                    *selected = index;
                    response.changed = true;
                    self.state.open_dropdown = None;
                    response.open = false;
                }
            }

            if pressed && !hovered && !any_hovered {
                self.state.open_dropdown = None;
                response.open = false;
            }

            if response.open {
                self.push_dropdown_overlay(rect, options, hovered_index, open_up);
            }

            if pressed {
                self.consume_mouse_pressed();
            }
        }

        self.layout.cursor.y = rect.origin.y + block_size.height as i32 + self.layout.spacing;
        response
    }

    /// Draw a dropdown selector with a stable key and a dynamic label.
    pub fn dropdown_with_key(
        &mut self,
        key: &str,
        label: &str,
        options: &[&str],
        selected: &mut usize,
        width: i32,
        height: i32,
    ) -> DropdownResponse {
        let id = WidgetId::from_label(key);
        self.dropdown(id, label, options, selected, width, height)
    }

    /// Render a knob in a fixed rectangle without affecting surrounding layout.
    pub(crate) fn knob_with_labels_in_rect(
        &mut self,
        id: WidgetId,
        name_label: &str,
        value_label: &str,
        value: &mut f32,
        range: (f32, f32),
        rect: Rect,
    ) -> KnobResponse {
        let previous = *self.layout;
        let label_height = 8 * self.theme.text_scale as i32;
        let label_gap = 4 * self.theme.text_scale as i32;
        // Keep knob rendering bounded to the default diameter so measured and
        // rendered footprints remain consistent in dense declarative layouts.
        let available_height = (rect.size.height as i32 - label_height * 2 - label_gap * 2).max(1);
        let knob_size = DEFAULT_KNOB_DIAMETER
            .min(rect.size.width as i32)
            .min(available_height)
            .max(1);
        self.layout.cursor = rect.origin;
        self.layout.knob_size = knob_size;
        let response = self.knob_with_labels(id, name_label, value_label, value, range);
        *self.layout = previous;
        response
    }

    /// Render a slider in a fixed rectangle without affecting surrounding layout.
    pub(crate) fn slider_in_rect(
        &mut self,
        id: WidgetId,
        label: &str,
        value: &mut f32,
        range: (f32, f32),
        control_size: Size,
        rect: Rect,
    ) -> SliderResponse {
        let previous = *self.layout;
        self.layout.cursor = rect.origin;
        let height = control_size.height.max(1) as i32;
        let response = self.slider(
            id,
            label,
            value,
            range,
            rect.size.width.max(1) as i32,
            height,
        );
        *self.layout = previous;
        response
    }

    /// Render a toggle in a fixed rectangle without affecting surrounding layout.
    pub(crate) fn toggle_in_rect(
        &mut self,
        id: WidgetId,
        label: &str,
        value: &mut bool,
        control_size: Size,
        rect: Rect,
    ) -> ToggleResponse {
        let previous = *self.layout;
        self.layout.cursor = rect.origin;
        let height = control_size.height.max(1) as i32;
        let response = self.toggle(id, label, value, rect.size.width.max(1) as i32, height);
        *self.layout = previous;
        response
    }

    /// Render a button in a fixed rectangle without affecting surrounding layout.
    pub(crate) fn button_in_rect(
        &mut self,
        id: WidgetId,
        label: &str,
        _control_size: Size,
        rect: Rect,
    ) -> ButtonResponse {
        let previous = *self.layout;
        self.layout.cursor = rect.origin;
        let response = self.button(
            id,
            label,
            rect.size.width.max(1) as i32,
            rect.size.height.max(1) as i32,
        );
        *self.layout = previous;
        response
    }

    /// Render a dropdown in a fixed rectangle without affecting surrounding layout.
    pub(crate) fn dropdown_in_rect(
        &mut self,
        id: WidgetId,
        label: &str,
        options: &[&str],
        selected: &mut usize,
        control_size: Size,
        rect: Rect,
    ) -> DropdownResponse {
        let previous = *self.layout;
        self.layout.cursor = rect.origin;
        let height = control_size.height.max(1) as i32;
        let response = self.dropdown(
            id,
            label,
            options,
            selected,
            rect.size.width.max(1) as i32,
            height,
        );
        *self.layout = previous;
        response
    }

    /// Clear the background with the theme color.
    pub fn clear(&mut self) {
        self.canvas.clear(self.theme.background);
    }

    /// Draw a non-interactive indicator cell.
    ///
    /// This is useful for sequencer step lights or other simple state displays.
    pub fn indicator(&mut self, rect: Rect, active: bool) {
        let fill = if active {
            self.theme.knob_indicator
        } else {
            self.theme.knob_fill
        };
        self.canvas.fill_rect(rect, fill);
        self.canvas.stroke_rect(rect, 1, self.theme.knob_outline);
        self.track_rect_internal(rect);
    }
}

/// Convert a knob angle in mathematical coordinates into a screen-space point.
///
/// Angles are interpreted in the same convention as [`Canvas::stroke_arc`]:
/// `0` points right and positive rotation is counter-clockwise. Screen-space
/// Y grows downward, so the sine term is inverted.
fn knob_indicator_point(center: Point, radius: i32, angle: f32) -> Point {
    Point {
        x: center.x + (angle.cos() * (radius as f32 * 0.7)) as i32,
        y: center.y - (angle.sin() * (radius as f32 * 0.7)) as i32,
    }
}

fn local_pointer_in_rect(pointer: Point, rect: Rect) -> Point {
    let max_x = rect.size.width.saturating_sub(1) as i32;
    let max_y = rect.size.height.saturating_sub(1) as i32;
    Point {
        x: (pointer.x - rect.origin.x).clamp(0, max_x.max(0)),
        y: (pointer.y - rect.origin.y).clamp(0, max_y.max(0)),
    }
}

fn raw_local_pointer_in_rect(pointer: Point, rect: Rect) -> Point {
    Point {
        x: pointer.x - rect.origin.x,
        y: pointer.y - rect.origin.y,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::Canvas;
    use crate::host::InputState;

    #[test]
    fn knob_updates_value_on_drag() {
        let mut canvas = Canvas::new(200, 200);
        let mut layout = Layout::default();
        let layout_origin = layout.cursor;
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let mut value = 0.5;
        let mut input = InputState {
            pointer_pos: Point { x: 40, y: 60 },
            mouse_pressed: true,
            mouse_down: true,
            ..InputState::default()
        };

        {
            let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
            ui.knob(WidgetId::new(1), "GAIN", &mut value, (0.0, 1.0));
        }

        input.mouse_pressed = false;
        input.pointer_pos = Point { x: 40, y: 20 };
        layout.cursor = layout_origin;

        {
            let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
            let response = ui.knob(WidgetId::new(1), "GAIN", &mut value, (0.0, 1.0));
            assert!(response.changed);
            assert!(value < 0.5, "dragging up should now reduce value");
        }
    }

    #[test]
    fn knob_with_key_allows_dynamic_labels() {
        let mut canvas = Canvas::new(200, 200);
        let mut layout = Layout::default();
        let layout_origin = layout.cursor;
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let mut value = 0.5;
        let mut input = InputState {
            pointer_pos: Point { x: 40, y: 60 },
            mouse_pressed: true,
            mouse_down: true,
            ..InputState::default()
        };

        {
            let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
            ui.knob_with_key("attack", "Attack 0.50s", &mut value, (0.0, 1.0));
        }

        input.mouse_pressed = false;
        input.pointer_pos = Point { x: 40, y: 20 };
        layout.cursor = layout_origin;

        {
            let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
            let response = ui.knob_with_key("attack", "Attack 0.60s", &mut value, (0.0, 1.0));
            assert!(response.changed);
        }
    }

    #[test]
    fn knob_indicator_point_uses_arc_coordinate_convention() {
        let center = Point { x: 100, y: 100 };
        let radius = 20;

        let start = 7.0 * std::f32::consts::PI / 4.0;
        let top = std::f32::consts::PI / 2.0;

        let start_point = knob_indicator_point(center, radius, start);
        let top_point = knob_indicator_point(center, radius, top);

        assert!(start_point.x > center.x);
        assert!(start_point.y > center.y);
        assert!(top_point.y < center.y);
    }

    #[test]
    fn knob_in_rect_does_not_expand_beyond_default_diameter() {
        let mut canvas = Canvas::new(260, 260);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let mut value = 0.5;
        let input = InputState {
            pointer_pos: Point { x: 24, y: 150 },
            ..InputState::default()
        };

        let rect = Rect {
            origin: Point { x: 0, y: 0 },
            size: Size {
                width: 200,
                height: 220,
            },
        };
        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let response = ui.knob_with_labels_in_rect(
            WidgetId::new(77),
            "GAIN",
            "50%",
            &mut value,
            (0.0, 1.0),
            rect,
        );

        assert!(
            !response.hovered,
            "pointer should be below a default-sized knob, even in a tall rect"
        );
    }

    #[test]
    fn knob_labels_are_clamped_to_knob_width() {
        let mut canvas = Canvas::new(320, 240);
        let mut layout = Layout::default();
        let expected_width = (layout.knob_size.max(1) + KNOB_BLOCK_SIDE_PADDING * 2).max(1) as u32;
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState::default();
        let mut value = 0.5;

        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let response = ui.panel_with_key(
            "knob-clamp",
            PanelStyle {
                padding: 0,
                ..PanelStyle::default()
            },
            None,
            |ui, _| {
                let _ = ui.knob_with_key_labels(
                    "knob",
                    "PITCH DEPTHPITCH CURVE",
                    "100.000000000 HZ",
                    &mut value,
                    (0.0, 1.0),
                );
            },
        );

        assert_eq!(response.measured_size.width, expected_width);
    }

    #[test]
    fn slider_labels_are_clamped_to_control_width() {
        let mut canvas = Canvas::new(320, 240);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState::default();
        let mut value = 0.5;
        let width = 90;
        let height = 18;

        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let response = ui.panel_with_key(
            "slider-clamp",
            PanelStyle {
                padding: 0,
                ..PanelStyle::default()
            },
            None,
            |ui, _| {
                let _ = ui.slider(
                    WidgetId::new(201),
                    "VERY LONG SLIDER LABEL FOR DENSE LAYOUTS",
                    &mut value,
                    (0.0, 1.0),
                    width,
                    height,
                );
            },
        );

        assert_eq!(response.measured_size.width, width as u32);
    }

    #[test]
    fn toggle_labels_are_clamped_to_control_width() {
        let mut canvas = Canvas::new(320, 240);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState::default();
        let mut value = false;
        let width = 96;
        let height = 18;

        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let response = ui.panel_with_key(
            "toggle-clamp",
            PanelStyle {
                padding: 0,
                ..PanelStyle::default()
            },
            None,
            |ui, _| {
                let _ = ui.toggle(
                    WidgetId::new(202),
                    "VERY LONG TOGGLE LABEL FOR DENSE LAYOUTS",
                    &mut value,
                    width,
                    height,
                );
            },
        );

        assert_eq!(response.measured_size.width, width as u32);
    }

    #[test]
    fn dropdown_labels_are_clamped_to_control_width() {
        let mut canvas = Canvas::new(320, 240);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState::default();
        let options = ["One", "Two", "Three"];
        let mut selected = 0usize;
        let width = 92;
        let height = 18;

        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let response = ui.panel_with_key(
            "dropdown-clamp",
            PanelStyle {
                padding: 0,
                ..PanelStyle::default()
            },
            None,
            |ui, _| {
                let _ = ui.dropdown(
                    WidgetId::new(203),
                    "VERY LONG DROPDOWN LABEL FOR DENSE LAYOUTS",
                    &options,
                    &mut selected,
                    width,
                    height,
                );
            },
        );

        assert_eq!(response.measured_size.width, width as u32);
    }

    #[test]
    fn block_size_helpers_match_rendered_width_contracts() {
        let mut canvas = Canvas::new(200, 200);
        let mut layout = Layout::default();
        let expected_knob_width =
            (layout.knob_size.max(1) + KNOB_BLOCK_SIDE_PADDING * 2).max(1) as u32;
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState::default();
        let ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);

        let knob = ui.knob_block_size("Pitch Depth", "100%");
        assert_eq!(knob.width, expected_knob_width);

        let slider = ui.slider_block_size(
            "Drive",
            Size {
                width: 84,
                height: 16,
            },
        );
        assert_eq!(slider.width, 84);

        let toggle = ui.toggle_block_size(
            "Enable",
            Size {
                width: 70,
                height: 18,
            },
        );
        assert_eq!(toggle.width, 70);

        let dropdown = ui.dropdown_block_size(
            "Mode",
            Size {
                width: 112,
                height: 18,
            },
        );
        assert_eq!(dropdown.width, 112);

        let button = ui.button_block_size(
            "Apply",
            Size {
                width: 88,
                height: 22,
            },
        );
        assert_eq!(button.width, 88);
        assert_eq!(button.height, 22);
    }

    #[test]
    fn slider_updates_value_on_drag() {
        let mut canvas = Canvas::new(200, 200);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let mut value = 0.0;
        let mut input = InputState {
            pointer_pos: Point { x: 20, y: 40 },
            mouse_pressed: true,
            mouse_down: true,
            ..InputState::default()
        };

        {
            let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
            ui.slider(WidgetId::new(2), "GAIN", &mut value, (0.0, 1.0), 100, 16);
        }

        input.mouse_pressed = false;
        input.pointer_pos = Point { x: 80, y: 40 };

        {
            let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
            let response = ui.slider(WidgetId::new(2), "GAIN", &mut value, (0.0, 1.0), 100, 16);
            assert!(response.changed);
        }
    }

    #[test]
    fn root_frame_measures_text_content() {
        let mut canvas = Canvas::new(200, 200);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState::default();

        {
            let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
            ui.root_frame_with_key(
                "root",
                RootFrameStyle {
                    padding: 0,
                    ..RootFrameStyle::default()
                },
                None,
                |ui, _rect| {
                    ui.text(Point { x: 0, y: 0 }, "Root");
                },
            );
        }

        let measured = ui_state
            .take_root_frame_size()
            .expect("root frame size missing");
        let expected = text_size("Root", theme.text_scale);
        assert_eq!(measured.width, expected.width);
        assert_eq!(measured.height, expected.height);
    }

    #[test]
    fn root_frame_respects_explicit_size() {
        let mut canvas = Canvas::new(200, 200);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState::default();
        let explicit = Size {
            width: 123,
            height: 77,
        };

        {
            let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
            ui.root_frame_with_key(
                "root",
                RootFrameStyle::default(),
                Some(explicit),
                |_ui, _| {},
            );
        }

        let measured = ui_state
            .take_root_frame_size()
            .expect("root frame size missing");
        assert_eq!(measured, explicit);
    }

    #[test]
    fn root_frame_clamps_explicit_size_to_content() {
        let mut canvas = Canvas::new(200, 200);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState::default();
        let explicit = Size {
            width: 1,
            height: 1,
        };

        {
            let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
            ui.root_frame_with_key(
                "root",
                RootFrameStyle {
                    padding: 0,
                    ..RootFrameStyle::default()
                },
                Some(explicit),
                |ui, _| {
                    ui.text(Point { x: 0, y: 0 }, "Root");
                },
            );
        }

        let measured = ui_state
            .take_root_frame_size()
            .expect("root frame size missing");
        let expected = text_size("Root", theme.text_scale);
        assert!(measured.width >= expected.width);
        assert!(measured.height >= expected.height);
    }

    #[test]
    fn toggle_flips_on_click() {
        let mut canvas = Canvas::new(200, 200);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let mut value = false;
        let input = InputState {
            pointer_pos: Point { x: 20, y: 40 },
            mouse_pressed: true,
            ..InputState::default()
        };

        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let response = ui.toggle(WidgetId::new(3), "Toggle", &mut value, 40, 16);
        assert!(response.changed);
        assert!(value);
    }

    #[test]
    fn button_reports_click() {
        let mut canvas = Canvas::new(200, 200);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState {
            pointer_pos: Point { x: 20, y: 20 },
            mouse_pressed: true,
            ..InputState::default()
        };

        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let response = ui.button(WidgetId::new(4), "OK", 40, 16);
        assert!(response.clicked);
    }

    #[test]
    fn dropdown_selects_option() {
        let mut canvas = Canvas::new(200, 200);
        let mut layout = Layout::default();
        let layout_origin = layout.cursor;
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let mut input = InputState {
            pointer_pos: Point { x: 20, y: 40 },
            mouse_pressed: true,
            ..InputState::default()
        };
        let options = ["Off", "Mono", "Poly"];
        let mut selected = 0;
        {
            let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
            ui.dropdown(WidgetId::new(5), "Mode", &options, &mut selected, 80, 16);
        }

        input.mouse_pressed = true;
        input.pointer_pos = Point { x: 20, y: 70 };
        layout.cursor = layout_origin;
        {
            let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
            ui.reset_input_consumption();
            let response = ui.dropdown(WidgetId::new(5), "Mode", &options, &mut selected, 80, 16);
            assert!(response.changed);
            assert_eq!(selected, 1);
        }
    }

    #[test]
    fn dropdown_respects_consumed_mouse_press() {
        let mut canvas = Canvas::new(200, 200);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let options = ["One", "Two"];
        let mut selected = 0;

        ui_state.consume_mouse_pressed = true;
        let input = InputState {
            pointer_pos: Point { x: 20, y: 40 },
            mouse_pressed: true,
            ..InputState::default()
        };

        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let response = ui.dropdown(WidgetId::new(9), "Mode", &options, &mut selected, 80, 16);
        assert!(!response.open);
        assert!(ui.state.open_dropdown.is_none());
    }

    #[test]
    fn panel_auto_sizes_after_draw() {
        let mut canvas = Canvas::new(400, 200);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState::default();

        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let response = ui.panel_with_key(
            "panel",
            PanelStyle {
                title: Some("Panel"),
                ..PanelStyle::default()
            },
            None,
            |ui, _rect| {
                let mut value = 0.5;
                ui.knob_with_key("gain", "GAIN", &mut value, (0.0, 1.0));
            },
        );

        assert!(response.measured_size.width > 0);
        assert!(response.measured_size.height > 0);
    }

    #[test]
    fn panel_auto_size_advances_layout_by_measured_height() {
        let mut canvas = Canvas::new(400, 200);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState::default();

        let start = layout.cursor;
        let spacing = layout.spacing;

        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let response = ui.panel_with_key(
            "panel-advance",
            PanelStyle {
                title: Some("Panel"),
                ..PanelStyle::default()
            },
            None,
            |ui, _rect| {
                let mut value = 0.5;
                ui.knob_with_key("gain", "GAIN", &mut value, (0.0, 1.0));
            },
        );

        let expected_y = start.y + response.measured_size.height as i32 + spacing;
        assert_eq!(ui.layout.cursor.y, expected_y);
    }

    #[test]
    fn panel_clamps_explicit_size_to_content() {
        let mut canvas = Canvas::new(400, 200);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState::default();
        let explicit = Size {
            width: 1,
            height: 1,
        };

        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let response = ui.panel_with_key(
            "panel-explicit",
            PanelStyle {
                title: Some("Panel"),
                ..PanelStyle::default()
            },
            Some(explicit),
            |ui, _rect| {
                let mut value = 0.5;
                ui.knob_with_key("gain", "GAIN", &mut value, (0.0, 1.0));
            },
        );

        assert!(response.measured_size.width > explicit.width);
        assert!(response.measured_size.height > explicit.height);
    }

    #[test]
    fn grid_cell_positions_are_consistent() {
        let mut canvas = Canvas::new(200, 200);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState::default();

        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
        let origin = Point { x: 10, y: 20 };
        let spec = GridSpec {
            columns: 4,
            cell_size: Size {
                width: 10,
                height: 12,
            },
            gap: 2,
            rows: None,
        };
        let response = ui.grid_with_key("grid", spec, origin, |_ui, grid| {
            let rect = grid.cell_rect(5);
            assert_eq!(rect.origin.x, origin.x + (10 + 2));
            assert_eq!(rect.origin.y, origin.y + (12 + 2));
        });

        assert_eq!(response.rows, 2);
        assert_eq!(response.columns, 4);
    }
}
