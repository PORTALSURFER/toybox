//! Declarative layout primitives for Patchbay GUI widgets.

use crate::canvas::{Color, Point, Rect, Size};
use crate::ui::{Theme, Ui};

/// Declarative UI tree describing a window.
pub struct UiSpec<'a, C> {
    /// Root frame definition.
    pub root: RootFrameSpec<'a, C>,
}

/// Root frame definition for a declarative window.
pub struct RootFrameSpec<'a, C> {
    /// Stable frame key.
    pub key: String,
    /// Optional title displayed in the header.
    pub title: Option<String>,
    /// Padding inside the frame.
    pub padding: i32,
    /// Root content.
    pub content: Box<Node<'a, C>>,
}

/// Layout nodes for the declarative UI tree.
pub enum Node<'a, C> {
    /// A titled panel container.
    Panel(PanelSpec<'a, C>),
    /// Horizontal layout container.
    Row(FlexSpec<'a, C>),
    /// Vertical layout container.
    Column(FlexSpec<'a, C>),
    /// Grid layout container.
    Grid(GridSpec<'a, C>),
    /// Text label.
    Label(LabelSpec),
    /// Fixed-size spacer.
    Spacer(SpacerSpec),
    /// Custom widget rendered by a closure.
    Widget(WidgetSpec<'a, C>),
}

/// Sizing behavior for declarative nodes.
#[derive(Clone, Copy, Debug)]
pub enum SizeSpec {
    /// Size to fit content.
    Auto,
    /// Fixed size in pixels.
    Fixed(Size),
    /// Fill available space.
    Fill,
}

/// Padding for container nodes.
#[derive(Clone, Copy, Debug, Default)]
pub struct Padding {
    /// Left padding in pixels.
    pub left: i32,
    /// Right padding in pixels.
    pub right: i32,
    /// Top padding in pixels.
    pub top: i32,
    /// Bottom padding in pixels.
    pub bottom: i32,
}

impl Padding {
    /// Create uniform padding on all sides.
    pub const fn uniform(value: i32) -> Self {
        Self {
            left: value,
            right: value,
            top: value,
            bottom: value,
        }
    }

    /// Create symmetric horizontal/vertical padding.
    pub const fn symmetric(horizontal: i32, vertical: i32) -> Self {
        Self {
            left: horizontal,
            right: horizontal,
            top: vertical,
            bottom: vertical,
        }
    }
}

/// Cross-axis alignment for flex containers.
#[derive(Clone, Copy, Debug)]
pub enum Align {
    /// Align to the start of the cross axis.
    Start,
    /// Center on the cross axis.
    Center,
    /// Align to the end of the cross axis.
    End,
}

impl Default for Align {
    fn default() -> Self {
        Self::Start
    }
}

/// Row/column layout specification.
pub struct FlexSpec<'a, C> {
    /// Optional explicit size for the container.
    pub size: SizeSpec,
    /// Gap between children in pixels.
    pub gap: i32,
    /// Padding applied to the container.
    pub padding: Padding,
    /// Cross-axis alignment.
    pub align: Align,
    /// Child nodes.
    pub children: Vec<Node<'a, C>>,
}

/// Grid layout specification.
pub struct GridSpec<'a, C> {
    /// Optional explicit size for the grid.
    pub size: SizeSpec,
    /// Number of columns in the grid.
    pub columns: i32,
    /// Size of each cell.
    pub cell_size: Size,
    /// Gap between cells in pixels.
    pub gap: i32,
    /// Padding applied to the grid.
    pub padding: Padding,
    /// Grid children in row-major order.
    pub children: Vec<Node<'a, C>>,
}

/// Panel layout specification.
pub struct PanelSpec<'a, C> {
    /// Stable panel key.
    pub key: String,
    /// Optional title displayed in the header.
    pub title: Option<String>,
    /// Padding inside the panel.
    pub padding: i32,
    /// Optional explicit size for the panel.
    pub size: SizeSpec,
    /// Panel content.
    pub content: Box<Node<'a, C>>,
}

/// Text label specification.
pub struct LabelSpec {
    /// Label text.
    pub text: String,
    /// Optional explicit size for the label.
    pub size: SizeSpec,
    /// Optional text color override.
    pub color: Option<Color>,
}

/// Spacer specification.
pub struct SpacerSpec {
    /// Fixed size for the spacer.
    pub size: Size,
}

/// Widget specification rendered by a callback.
pub struct WidgetSpec<'a, C> {
    /// Stable widget key.
    pub key: String,
    /// Optional explicit size for the widget.
    pub size: SizeSpec,
    /// Render callback invoked with the resolved rectangle.
    pub render: Box<dyn FnMut(&mut Ui<'_>, Rect, &mut C) + 'a>,
}

/// Measure the required size for a UI specification.
pub fn measure<C>(spec: &UiSpec<'_, C>, theme: &Theme) -> Size {
    measure_root_frame(&spec.root, theme)
}

/// Render a UI specification and return the measured size.
///
/// The root frame is always anchored at the window origin, so the `origin`
/// parameter is ignored.
pub fn render<C>(spec: &mut UiSpec<'_, C>, ui: &mut Ui<'_>, origin: Point, ctx: &mut C) -> Size {
    let theme = ui.theme().clone();
    let measured = measure_root_frame(&spec.root, &theme);
    let content = &mut *spec.root.content;
    let frame_padding = spec.root.padding;
    let title = spec.root.title.as_deref();
    let header_height = panel_header_height(title, &theme);
    let style = crate::ui::RootFrameStyle {
        title,
        padding: frame_padding,
        background: None,
        outline: None,
        header_height: Some(header_height),
    };
    let _ = origin;
    ui.root_frame_with_key(&spec.root.key, style, Some(measured), |ui, rect| {
        render_node(content, rect, ui, &theme, ctx);
    });
    measured
}

fn render_node<C>(node: &mut Node<'_, C>, rect: Rect, ui: &mut Ui<'_>, theme: &Theme, ctx: &mut C) {
    match node {
        Node::Panel(panel) => render_panel(panel, rect, ui, theme, ctx),
        Node::Row(flex) => render_flex(flex, rect, ui, theme, Axis::Horizontal, ctx),
        Node::Column(flex) => render_flex(flex, rect, ui, theme, Axis::Vertical, ctx),
        Node::Grid(grid) => render_grid(grid, rect, ui, theme, ctx),
        Node::Label(label) => render_label(label, rect, ui, theme),
        Node::Spacer(_) => {}
        Node::Widget(widget) => (widget.render)(ui, rect, ctx),
    }
}

fn measure_node<C>(node: &Node<'_, C>, theme: &Theme) -> Size {
    match node {
        Node::Panel(panel) => measure_panel(panel, theme),
        Node::Row(flex) => measure_flex(flex, theme, Axis::Horizontal),
        Node::Column(flex) => measure_flex(flex, theme, Axis::Vertical),
        Node::Grid(grid) => measure_grid(grid, theme),
        Node::Label(label) => measure_label(label, theme),
        Node::Spacer(spacer) => spacer.size,
        Node::Widget(widget) => match widget.size {
            SizeSpec::Fixed(size) => size,
            _ => Size { width: 0, height: 0 },
        },
    }
}

fn measure_root_frame<C>(frame: &RootFrameSpec<'_, C>, theme: &Theme) -> Size {
    let content = measure_node(&frame.content, theme);
    let header_height = panel_header_height(frame.title.as_deref(), theme);
    let width = content.width + (frame.padding.max(0) * 2) as u32;
    let height =
        content.height + (frame.padding.max(0) * 2 + header_height) as u32;
    Size { width, height }
}

fn measure_panel<C>(panel: &PanelSpec<'_, C>, theme: &Theme) -> Size {
    let content = measure_node(&panel.content, theme);
    let header_height = panel_header_height(panel.title.as_deref(), theme);
    let width = content.width + (panel.padding.max(0) * 2) as u32;
    let height =
        content.height + (panel.padding.max(0) * 2 + header_height) as u32;
    match panel.size {
        SizeSpec::Fixed(size) => size,
        _ => Size { width, height },
    }
}

fn measure_flex<C>(flex: &FlexSpec<'_, C>, theme: &Theme, axis: Axis) -> Size {
    if let SizeSpec::Fixed(size) = flex.size {
        return size;
    }
    let mut total_main = 0i32;
    let mut max_cross = 0i32;
    for child in &flex.children {
        let size = measure_node(child, theme);
        let (main, cross) = match axis {
            Axis::Horizontal => (size.width as i32, size.height as i32),
            Axis::Vertical => (size.height as i32, size.width as i32),
        };
        total_main += main;
        max_cross = max_cross.max(cross);
    }
    let gaps = flex.gap.max(0) * (flex.children.len().saturating_sub(1) as i32);
    total_main += gaps;
    let padded_main = total_main + flex.padding.left + flex.padding.right;
    let padded_cross =
        max_cross + flex.padding.top + flex.padding.bottom;
    match axis {
        Axis::Horizontal => Size {
            width: padded_main.max(0) as u32,
            height: padded_cross.max(0) as u32,
        },
        Axis::Vertical => Size {
            width: padded_cross.max(0) as u32,
            height: padded_main.max(0) as u32,
        },
    }
}

fn measure_grid<C>(grid: &GridSpec<'_, C>, _theme: &Theme) -> Size {
    if let SizeSpec::Fixed(size) = grid.size {
        return size;
    }
    let columns = grid.columns.max(1);
    let rows = if grid.children.is_empty() {
        0
    } else {
        (grid.children.len() as i32 + columns - 1) / columns
    };
    let width =
        columns * grid.cell_size.width as i32 + (columns - 1) * grid.gap.max(0);
    let height =
        rows * grid.cell_size.height as i32 + (rows - 1) * grid.gap.max(0);
    Size {
        width: (width + grid.padding.left + grid.padding.right).max(0) as u32,
        height: (height + grid.padding.top + grid.padding.bottom).max(0) as u32,
    }
}

fn measure_label(label: &LabelSpec, theme: &Theme) -> Size {
    match label.size {
        SizeSpec::Fixed(size) => size,
        _ => text_size(&label.text, theme.text_scale),
    }
}

fn render_panel<C>(
    panel: &mut PanelSpec<'_, C>,
    rect: Rect,
    ui: &mut Ui<'_>,
    theme: &Theme,
    ctx: &mut C,
) {
    let background = theme.knob_fill;
    let outline = theme.knob_outline;
    let canvas = ui.canvas();
    canvas.fill_rect(rect, background);
    canvas.stroke_rect(rect, 1, outline);

    let header_height = panel_header_height(panel.title.as_deref(), theme);
    if let Some(title) = panel.title.as_deref() {
        let pos = Point {
            x: rect.origin.x + panel.padding.max(0),
            y: rect.origin.y + panel.padding.max(0),
        };
        canvas.draw_text(pos, title, theme.text, theme.text_scale);
    }

    let content_origin = Point {
        x: rect.origin.x + panel.padding.max(0),
        y: rect.origin.y + panel.padding.max(0) + header_height,
    };
    let content_rect = Rect {
        origin: content_origin,
        size: Size {
            width: rect
                .size
                .width
                .saturating_sub((panel.padding.max(0) * 2) as u32),
            height: rect
                .size
                .height
                .saturating_sub((panel.padding.max(0) * 2 + header_height) as u32),
        },
    };
    render_node(&mut panel.content, content_rect, ui, theme, ctx);
}

fn render_flex<C>(
    flex: &mut FlexSpec<'_, C>,
    rect: Rect,
    ui: &mut Ui<'_>,
    theme: &Theme,
    axis: Axis,
    ctx: &mut C,
) {
    let mut main_sizes = Vec::with_capacity(flex.children.len());
    let mut cross_sizes = Vec::with_capacity(flex.children.len());
    let mut fixed_total = 0i32;
    let mut fill_count = 0i32;

    for child in &flex.children {
        let size = measure_node(child, theme);
        let (main, cross) = match axis {
            Axis::Horizontal => (size.width as i32, size.height as i32),
            Axis::Vertical => (size.height as i32, size.width as i32),
        };
        main_sizes.push(main);
        cross_sizes.push(cross);
        if matches!(child_size_spec(child), SizeSpec::Fill) {
            fill_count += 1;
        } else {
            fixed_total += main;
        }
    }

    let gap_total = flex.gap.max(0) * (flex.children.len().saturating_sub(1) as i32);
    let available_main = match axis {
        Axis::Horizontal => rect.size.width as i32,
        Axis::Vertical => rect.size.height as i32,
    };
    let available_main = available_main
        - gap_total
        - flex.padding.left
        - flex.padding.right;
    let remaining = (available_main - fixed_total).max(0);
    let fill_size = if fill_count > 0 {
        remaining / fill_count
    } else {
        0
    };

    let mut cursor_main = match axis {
        Axis::Horizontal => rect.origin.x + flex.padding.left,
        Axis::Vertical => rect.origin.y + flex.padding.top,
    };
    let child_count = flex.children.len();
    for (index, child) in flex.children.iter_mut().enumerate() {
        let size = measure_node(child, theme);
        let mut main = match axis {
            Axis::Horizontal => size.width as i32,
            Axis::Vertical => size.height as i32,
        };
        if matches!(child_size_spec(child), SizeSpec::Fill) {
            main = fill_size;
        }
        let cross = match axis {
            Axis::Horizontal => size.height as i32,
            Axis::Vertical => size.width as i32,
        };
        let cross_available = match axis {
            Axis::Horizontal => rect.size.height as i32 - flex.padding.top - flex.padding.bottom,
            Axis::Vertical => rect.size.width as i32 - flex.padding.left - flex.padding.right,
        };
        let cross_origin = match flex.align {
            Align::Start => match axis {
                Axis::Horizontal => rect.origin.y + flex.padding.top,
                Axis::Vertical => rect.origin.x + flex.padding.left,
            },
            Align::Center => match axis {
                Axis::Horizontal => {
                    rect.origin.y + flex.padding.top + (cross_available - cross) / 2
                }
                Axis::Vertical => {
                    rect.origin.x + flex.padding.left + (cross_available - cross) / 2
                }
            },
            Align::End => match axis {
                Axis::Horizontal => rect.origin.y + flex.padding.top + (cross_available - cross),
                Axis::Vertical => rect.origin.x + flex.padding.left + (cross_available - cross),
            },
        };
        let child_rect = match axis {
            Axis::Horizontal => Rect {
                origin: Point {
                    x: cursor_main,
                    y: cross_origin,
                },
                size: Size {
                    width: main.max(0) as u32,
                    height: cross.max(0) as u32,
                },
            },
            Axis::Vertical => Rect {
                origin: Point {
                    x: cross_origin,
                    y: cursor_main,
                },
                size: Size {
                    width: cross.max(0) as u32,
                    height: main.max(0) as u32,
                },
            },
        };
        render_node(child, child_rect, ui, theme, ctx);
        cursor_main += main + flex.gap.max(0);
        if index == child_count.saturating_sub(1) {
            break;
        }
    }
}

fn render_grid<C>(
    grid: &mut GridSpec<'_, C>,
    rect: Rect,
    ui: &mut Ui<'_>,
    theme: &Theme,
    ctx: &mut C,
) {
    let columns = grid.columns.max(1);
    let base = Point {
        x: rect.origin.x + grid.padding.left,
        y: rect.origin.y + grid.padding.top,
    };
    for (index, child) in grid.children.iter_mut().enumerate() {
        let idx = index as i32;
        let row = idx / columns;
        let col = idx % columns;
        let origin = Point {
            x: base.x + col * (grid.cell_size.width as i32 + grid.gap.max(0)),
            y: base.y + row * (grid.cell_size.height as i32 + grid.gap.max(0)),
        };
        let child_rect = Rect {
            origin,
            size: grid.cell_size,
        };
        render_node(child, child_rect, ui, theme, ctx);
    }
}

fn render_label(label: &LabelSpec, rect: Rect, ui: &mut Ui<'_>, theme: &Theme) {
    let color = label.color.unwrap_or(theme.text);
    let pos = rect.origin;
    ui.canvas().draw_text(pos, &label.text, color, theme.text_scale);
}

fn panel_header_height(title: Option<&str>, theme: &Theme) -> i32 {
    if title.is_some() {
        (8 * theme.text_scale as i32 + 4).max(0)
    } else {
        0
    }
}

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

fn child_size_spec<C>(node: &Node<'_, C>) -> SizeSpec {
    match node {
        Node::Panel(panel) => panel.size,
        Node::Row(flex) => flex.size,
        Node::Column(flex) => flex.size,
        Node::Grid(grid) => grid.size,
        Node::Label(label) => label.size,
        Node::Spacer(_) => SizeSpec::Fixed(Size { width: 0, height: 0 }),
        Node::Widget(widget) => widget.size,
    }
}

#[derive(Clone, Copy)]
enum Axis {
    Horizontal,
    Vertical,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::Canvas;
    use crate::host::InputState;
    use crate::ui::{Layout, UiState};

    #[test]
    fn measures_row_with_gap_and_padding() {
        let spec = UiSpec {
            root: RootFrameSpec {
                key: "root".to_string(),
                title: None,
                padding: 0,
                content: Box::new(Node::Row(FlexSpec {
                    size: SizeSpec::Auto,
                    gap: 4,
                    padding: Padding::uniform(2),
                    align: Align::Start,
                    children: vec![
                        Node::Spacer(SpacerSpec {
                            size: Size { width: 10, height: 8 },
                        }),
                        Node::Spacer(SpacerSpec {
                            size: Size { width: 6, height: 12 },
                        }),
                    ],
                })),
            },
        };
        let theme = Theme::default();
        let size = measure(&spec, &theme);
        assert_eq!(size.width, 10 + 6 + 4 + 2 * 2);
        assert_eq!(size.height, 12 + 2 * 2);
    }

    #[test]
    fn renders_widget_with_resolved_rect() {
        let rect_seen = std::cell::Cell::new(None);
        let mut spec = UiSpec {
            root: RootFrameSpec {
                key: "root".to_string(),
                title: None,
                padding: 0,
                content: Box::new(Node::Widget(WidgetSpec {
                    key: "widget".to_string(),
                    size: SizeSpec::Fixed(Size { width: 20, height: 10 }),
                    render: Box::new(|_ui, rect, _ctx: &mut ()| {
                        rect_seen.set(Some(rect));
                    }),
                })),
            },
        };
        let mut canvas = Canvas::new(100, 100);
        let mut layout = Layout::default();
        let theme = Theme::default();
        let mut ui_state = UiState::default();
        let input = InputState::default();
        let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);

        let measured = render(&mut spec, &mut ui, Point { x: 5, y: 7 }, &mut ());
        assert_eq!(measured.width, 20);
        assert_eq!(measured.height, 10);
        let rect = rect_seen.get().expect("widget rect not captured");
        assert_eq!(rect.origin.x, 0);
        assert_eq!(rect.origin.y, 0);
        assert_eq!(rect.size.width, 20);
        assert_eq!(rect.size.height, 10);
    }
}
