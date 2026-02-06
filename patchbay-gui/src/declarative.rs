//! Declarative layout primitives for Patchbay GUI widgets.

use crate::canvas::{Color, Point, Rect, Size};
use crate::logging::log_line_safe;
use crate::ui::{
    ButtonResponse, DropdownResponse, KnobResponse, RegionResponse, SliderResponse, Theme,
    ToggleResponse, Ui, WidgetId,
};

/// Validation errors produced by declarative UI helpers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum DeclarativeError {
    /// A widget-like node was placed outside of a panel container.
    #[error("declarative node `{node_kind}` must be nested inside a panel")]
    WidgetOutsidePanel {
        /// The concrete node type that violated the panel-only rule.
        node_kind: &'static str,
    },
}

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
    /// Absolute-positioned container for pixel-perfect layouts.
    Absolute(AbsoluteSpec<'a, C>),
    /// Text label.
    Label(LabelSpec),
    /// Fixed-size spacer.
    Spacer(SpacerSpec),
    /// Knob control widget.
    Knob(KnobSpec<'a, C>),
    /// Slider control widget.
    Slider(SliderSpec<'a, C>),
    /// Toggle control widget.
    Toggle(ToggleSpec<'a, C>),
    /// Button control widget.
    Button(ButtonSpec<'a, C>),
    /// Dropdown selector widget.
    Dropdown(DropdownSpec<'a, C>),
    /// Interactive region widget for custom drawing.
    Region(RegionSpec<'a, C>),
    /// Non-interactive indicator widget.
    Indicator(IndicatorSpec),
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
    /// Optional background fill for the panel.
    pub background: Option<Color>,
    /// Optional outline color for the panel.
    pub outline: Option<Color>,
    /// Optional header height override.
    pub header_height: Option<i32>,
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

/// Absolute positioning container specification.
pub struct AbsoluteSpec<'a, C> {
    /// Optional explicit size for the container.
    pub size: SizeSpec,
    /// Children with explicit origins.
    pub children: Vec<AbsoluteChild<'a, C>>,
}

/// Child node with an explicit origin.
pub struct AbsoluteChild<'a, C> {
    /// The child origin relative to the container.
    pub origin: Point,
    /// The child node.
    pub node: Node<'a, C>,
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

/// Knob widget specification.
pub struct KnobSpec<'a, C> {
    /// Stable widget key.
    pub key: String,
    /// Label displayed above the knob.
    pub label: String,
    /// Optional explicit value label below the knob.
    pub value_label: Option<String>,
    /// Current value for the knob.
    pub value: f32,
    /// Range of valid values.
    pub range: (f32, f32),
    /// Optional explicit size for the widget.
    pub size: SizeSpec,
    /// Optional callback invoked after interaction.
    pub on_interaction: Option<Box<dyn FnMut(&mut C, KnobEvent) + 'a>>,
}

/// Slider widget specification.
pub struct SliderSpec<'a, C> {
    /// Stable widget key.
    pub key: String,
    /// Optional label displayed above the slider.
    pub label: String,
    /// Current value for the slider.
    pub value: f32,
    /// Range of valid values.
    pub range: (f32, f32),
    /// Size of the slider control itself.
    pub control_size: Size,
    /// Optional explicit size for the widget.
    pub size: SizeSpec,
    /// Optional callback invoked after interaction.
    pub on_interaction: Option<Box<dyn FnMut(&mut C, SliderEvent) + 'a>>,
}

/// Toggle widget specification.
pub struct ToggleSpec<'a, C> {
    /// Stable widget key.
    pub key: String,
    /// Optional label displayed above the toggle.
    pub label: String,
    /// Current value for the toggle.
    pub value: bool,
    /// Size of the toggle control itself.
    pub control_size: Size,
    /// Optional explicit size for the widget.
    pub size: SizeSpec,
    /// Optional callback invoked after interaction.
    pub on_interaction: Option<Box<dyn FnMut(&mut C, ToggleEvent) + 'a>>,
}

/// Button widget specification.
pub struct ButtonSpec<'a, C> {
    /// Stable widget key.
    pub key: String,
    /// Label displayed inside the button.
    pub label: String,
    /// Size of the button control itself.
    pub control_size: Size,
    /// Optional explicit size for the widget.
    pub size: SizeSpec,
    /// Optional callback invoked after interaction.
    pub on_interaction: Option<Box<dyn FnMut(&mut C, ButtonEvent) + 'a>>,
}

/// Dropdown widget specification.
pub struct DropdownSpec<'a, C> {
    /// Stable widget key.
    pub key: String,
    /// Optional label displayed above the dropdown.
    pub label: String,
    /// Options displayed in the dropdown.
    pub options: Vec<String>,
    /// Current selected index.
    pub selected: usize,
    /// Size of the dropdown control itself.
    pub control_size: Size,
    /// Optional explicit size for the widget.
    pub size: SizeSpec,
    /// Optional callback invoked after interaction.
    pub on_interaction: Option<Box<dyn FnMut(&mut C, DropdownEvent) + 'a>>,
}

/// Interactive region widget specification.
pub struct RegionSpec<'a, C> {
    /// Stable widget key.
    pub key: String,
    /// Explicit size for the region.
    pub size: Size,
    /// Optional callback invoked after interaction.
    pub on_interaction: Option<Box<dyn FnMut(&mut C, RegionEvent) + 'a>>,
    /// Optional custom drawing callback.
    pub draw: Option<Box<dyn FnMut(&mut crate::canvas::Canvas, Rect, &mut C, RegionResponse) + 'a>>,
}

/// Indicator widget specification.
pub struct IndicatorSpec {
    /// Explicit size for the indicator.
    pub size: Size,
    /// Whether the indicator is active.
    pub active: bool,
}

/// Event data for knob widgets.
pub struct KnobEvent {
    /// The updated value.
    pub value: f32,
    /// The widget response metadata.
    pub response: KnobResponse,
}

/// Event data for slider widgets.
pub struct SliderEvent {
    /// The updated value.
    pub value: f32,
    /// The widget response metadata.
    pub response: SliderResponse,
}

/// Event data for toggle widgets.
pub struct ToggleEvent {
    /// The updated value.
    pub value: bool,
    /// The widget response metadata.
    pub response: ToggleResponse,
}

/// Event data for button widgets.
pub struct ButtonEvent {
    /// The widget response metadata.
    pub response: ButtonResponse,
}

/// Event data for dropdown widgets.
pub struct DropdownEvent {
    /// The updated selection index.
    pub selected: usize,
    /// The widget response metadata.
    pub response: DropdownResponse,
}

/// Event data for region widgets.
pub struct RegionEvent {
    /// The widget response metadata.
    pub response: RegionResponse,
}

/// Measure the required size for a UI specification.
///
/// This helper preserves backward compatibility by logging validation failures
/// and still returning a best-effort measurement. Use [`measure_checked`] when
/// callers need explicit error handling.
pub fn measure<C>(spec: &UiSpec<'_, C>, theme: &Theme) -> Size {
    if let Err(err) = validate_panel_only(&spec.root.content, false) {
        log_line_safe(&format!(
            "declarative: measure fallback after validation error: {err}"
        ));
    }
    measure_root_frame(&spec.root, theme)
}

/// Measure the required size for a UI specification, validating panel-only rules.
///
/// # Errors
/// Returns [`DeclarativeError::WidgetOutsidePanel`] when a widget-like node is
/// not nested inside a panel.
pub fn measure_checked<C>(spec: &UiSpec<'_, C>, theme: &Theme) -> Result<Size, DeclarativeError> {
    validate_panel_only(&spec.root.content, false)?;
    Ok(measure_root_frame(&spec.root, theme))
}

/// Render a UI specification and return the measured size.
///
/// The root frame is always anchored at the window origin, so the `origin`
/// parameter is ignored.
pub fn render<C>(spec: &mut UiSpec<'_, C>, ui: &mut Ui<'_>, origin: Point, ctx: &mut C) -> Size {
    let theme = ui.theme().clone();
    if let Err(err) = validate_panel_only(&spec.root.content, false) {
        log_line_safe(&format!(
            "declarative: render fallback after validation error: {err}"
        ));
    }
    render_impl(spec, ui, origin, ctx, &theme)
}

/// Render a UI specification and return the measured size.
///
/// This variant validates the declarative tree and returns a
/// [`DeclarativeError`] instead of panicking.
///
/// # Errors
/// Returns [`DeclarativeError::WidgetOutsidePanel`] when a widget-like node is
/// not nested inside a panel.
pub fn render_checked<C>(
    spec: &mut UiSpec<'_, C>,
    ui: &mut Ui<'_>,
    origin: Point,
    ctx: &mut C,
) -> Result<Size, DeclarativeError> {
    let theme = ui.theme().clone();
    validate_panel_only(&spec.root.content, false)?;
    Ok(render_impl(spec, ui, origin, ctx, &theme))
}

fn render_impl<C>(
    spec: &mut UiSpec<'_, C>,
    ui: &mut Ui<'_>,
    origin: Point,
    ctx: &mut C,
    theme: &Theme,
) -> Size {
    let measured = measure_root_frame(&spec.root, theme);
    let content = &mut *spec.root.content;
    let frame_padding = spec.root.padding;
    let title = spec.root.title.as_deref();
    let header_height = panel_header_height(title, theme);
    let style = crate::ui::RootFrameStyle {
        title,
        padding: frame_padding,
        background: None,
        outline: None,
        header_height: Some(header_height),
    };
    let _ = origin;
    ui.root_frame_with_key(&spec.root.key, style, Some(measured), |ui, rect| {
        render_node(content, rect, ui, theme, ctx);
    });
    measured
}

fn validate_panel_only<C>(node: &Node<'_, C>, in_panel: bool) -> Result<(), DeclarativeError> {
    match node {
        Node::Panel(panel) => validate_panel_only(&panel.content, true)?,
        Node::Row(flex) => {
            for child in &flex.children {
                validate_panel_only(child, in_panel)?;
            }
        }
        Node::Column(flex) => {
            for child in &flex.children {
                validate_panel_only(child, in_panel)?;
            }
        }
        Node::Grid(grid) => {
            for child in &grid.children {
                validate_panel_only(child, in_panel)?;
            }
        }
        Node::Absolute(absolute) => {
            for child in &absolute.children {
                validate_panel_only(&child.node, in_panel)?;
            }
        }
        Node::Label(_)
        | Node::Spacer(_)
        | Node::Knob(_)
        | Node::Slider(_)
        | Node::Toggle(_)
        | Node::Button(_)
        | Node::Dropdown(_)
        | Node::Region(_)
        | Node::Indicator(_)
        | Node::Widget(_) => {
            if !in_panel {
                return Err(DeclarativeError::WidgetOutsidePanel {
                    node_kind: node_kind(node),
                });
            }
        }
    }
    Ok(())
}

fn node_kind<C>(node: &Node<'_, C>) -> &'static str {
    match node {
        Node::Panel(_) => "Panel",
        Node::Row(_) => "Row",
        Node::Column(_) => "Column",
        Node::Grid(_) => "Grid",
        Node::Absolute(_) => "Absolute",
        Node::Label(_) => "Label",
        Node::Spacer(_) => "Spacer",
        Node::Knob(_) => "Knob",
        Node::Slider(_) => "Slider",
        Node::Toggle(_) => "Toggle",
        Node::Button(_) => "Button",
        Node::Dropdown(_) => "Dropdown",
        Node::Region(_) => "Region",
        Node::Indicator(_) => "Indicator",
        Node::Widget(_) => "Widget",
    }
}

fn render_node<C>(node: &mut Node<'_, C>, rect: Rect, ui: &mut Ui<'_>, theme: &Theme, ctx: &mut C) {
    match node {
        Node::Panel(panel) => render_panel(panel, rect, ui, theme, ctx),
        Node::Row(flex) => render_flex(flex, rect, ui, theme, Axis::Horizontal, ctx),
        Node::Column(flex) => render_flex(flex, rect, ui, theme, Axis::Vertical, ctx),
        Node::Grid(grid) => render_grid(grid, rect, ui, theme, ctx),
        Node::Absolute(absolute) => render_absolute(absolute, rect, ui, theme, ctx),
        Node::Label(label) => render_label(label, rect, ui, theme),
        Node::Spacer(_) => {}
        Node::Knob(knob) => render_knob(knob, rect, ui, theme, ctx),
        Node::Slider(slider) => render_slider(slider, rect, ui, theme, ctx),
        Node::Toggle(toggle) => render_toggle(toggle, rect, ui, theme, ctx),
        Node::Button(button) => render_button(button, rect, ui, theme, ctx),
        Node::Dropdown(dropdown) => render_dropdown(dropdown, rect, ui, theme, ctx),
        Node::Region(region) => render_region(region, rect, ui, theme, ctx),
        Node::Indicator(indicator) => render_indicator(indicator, rect, ui, theme),
        Node::Widget(widget) => (widget.render)(ui, rect, ctx),
    }
}

fn measure_node<C>(node: &Node<'_, C>, theme: &Theme) -> Size {
    match node {
        Node::Panel(panel) => measure_panel(panel, theme),
        Node::Row(flex) => measure_flex(flex, theme, Axis::Horizontal),
        Node::Column(flex) => measure_flex(flex, theme, Axis::Vertical),
        Node::Grid(grid) => measure_grid(grid, theme),
        Node::Absolute(absolute) => measure_absolute(absolute, theme),
        Node::Label(label) => measure_label(label, theme),
        Node::Spacer(spacer) => spacer.size,
        Node::Knob(knob) => measure_knob(knob, theme),
        Node::Slider(slider) => measure_slider(slider, theme),
        Node::Toggle(toggle) => measure_toggle(toggle, theme),
        Node::Button(button) => measure_button(button, theme),
        Node::Dropdown(dropdown) => measure_dropdown(dropdown, theme),
        Node::Region(region) => region.size,
        Node::Indicator(indicator) => indicator.size,
        Node::Widget(widget) => match widget.size {
            SizeSpec::Fixed(size) => size,
            _ => Size {
                width: 0,
                height: 0,
            },
        },
    }
}

fn measure_root_frame<C>(frame: &RootFrameSpec<'_, C>, theme: &Theme) -> Size {
    let content = measure_node(&frame.content, theme);
    let header_height = panel_header_height(frame.title.as_deref(), theme);
    let width = content.width + (frame.padding.max(0) * 2) as u32;
    let height = content.height + (frame.padding.max(0) * 2 + header_height) as u32;
    Size { width, height }
}

fn measure_panel<C>(panel: &PanelSpec<'_, C>, theme: &Theme) -> Size {
    let content = measure_node(&panel.content, theme);
    let header_height = panel
        .header_height
        .unwrap_or_else(|| panel_header_height(panel.title.as_deref(), theme));
    let width = content.width + (panel.padding.max(0) * 2) as u32;
    let height = content.height + (panel.padding.max(0) * 2 + header_height) as u32;
    clamp_size_spec(panel.size, Size { width, height })
}

fn measure_flex<C>(flex: &FlexSpec<'_, C>, theme: &Theme, axis: Axis) -> Size {
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
    let (main_padding, cross_padding) = match axis {
        Axis::Horizontal => (
            flex.padding.left + flex.padding.right,
            flex.padding.top + flex.padding.bottom,
        ),
        Axis::Vertical => (
            flex.padding.top + flex.padding.bottom,
            flex.padding.left + flex.padding.right,
        ),
    };
    let padded_main = total_main + main_padding;
    let padded_cross = max_cross + cross_padding;
    let measured = match axis {
        Axis::Horizontal => Size {
            width: padded_main.max(0) as u32,
            height: padded_cross.max(0) as u32,
        },
        Axis::Vertical => Size {
            width: padded_cross.max(0) as u32,
            height: padded_main.max(0) as u32,
        },
    };
    clamp_size_spec(flex.size, measured)
}

fn measure_grid<C>(grid: &GridSpec<'_, C>, theme: &Theme) -> Size {
    let (cell_width, cell_height) = grid_cell_minimums(grid, theme);
    let columns = grid.columns.max(1);
    let rows = if grid.children.is_empty() {
        0
    } else {
        (grid.children.len() as i32 + columns - 1) / columns
    };
    let width = columns * cell_width as i32 + (columns - 1) * grid.gap.max(0);
    let height = rows * cell_height as i32 + (rows - 1) * grid.gap.max(0);
    let measured = Size {
        width: (width + grid.padding.left + grid.padding.right).max(0) as u32,
        height: (height + grid.padding.top + grid.padding.bottom).max(0) as u32,
    };
    clamp_size_spec(grid.size, measured)
}

fn measure_label(label: &LabelSpec, theme: &Theme) -> Size {
    let measured = text_size(&label.text, theme.text_scale);
    clamp_size_spec(label.size, measured)
}

fn measure_absolute<C>(absolute: &AbsoluteSpec<'_, C>, theme: &Theme) -> Size {
    let mut max_x = 0i32;
    let mut max_y = 0i32;
    for child in &absolute.children {
        let size = measure_node(&child.node, theme);
        let right = child.origin.x + size.width as i32;
        let bottom = child.origin.y + size.height as i32;
        max_x = max_x.max(right);
        max_y = max_y.max(bottom);
    }
    let measured = Size {
        width: max_x.max(0) as u32,
        height: max_y.max(0) as u32,
    };
    clamp_size_spec(absolute.size, measured)
}

fn measure_knob<C>(knob: &KnobSpec<'_, C>, theme: &Theme) -> Size {
    let label_height = 8 * theme.text_scale as i32;
    let label_gap = 4 * theme.text_scale as i32;
    let label_size = text_size(&knob.label, theme.text_scale);
    let value_label = knob
        .value_label
        .clone()
        .unwrap_or_else(|| format_knob_value(knob.value));
    let value_size = text_size(&value_label, theme.text_scale);
    let knob_diameter = crate::ui::DEFAULT_KNOB_DIAMETER.max(1) as u32;
    let width = label_size.width.max(value_size.width).max(knob_diameter);
    let height = knob_diameter + (label_height * 2 + label_gap * 2).max(0) as u32;
    clamp_size_spec(knob.size, Size { width, height })
}

fn measure_slider<C>(slider: &SliderSpec<'_, C>, theme: &Theme) -> Size {
    let label_height = if slider.label.is_empty() {
        0
    } else {
        8 * theme.text_scale as i32
    };
    let label_size = text_size(&slider.label, theme.text_scale);
    let width = slider.control_size.width.max(label_size.width);
    let height = slider.control_size.height + label_height.max(0) as u32;
    clamp_size_spec(slider.size, Size { width, height })
}

fn measure_toggle<C>(toggle: &ToggleSpec<'_, C>, theme: &Theme) -> Size {
    let label_height = if toggle.label.is_empty() {
        0
    } else {
        8 * theme.text_scale as i32
    };
    let label_size = text_size(&toggle.label, theme.text_scale);
    let width = toggle.control_size.width.max(label_size.width);
    let height = toggle.control_size.height + label_height.max(0) as u32;
    clamp_size_spec(toggle.size, Size { width, height })
}

fn measure_button<C>(button: &ButtonSpec<'_, C>, theme: &Theme) -> Size {
    let label_size = text_size(&button.label, theme.text_scale);
    let width = button.control_size.width.max(label_size.width + 8);
    let height = button.control_size.height.max(label_size.height + 4);
    clamp_size_spec(button.size, Size { width, height })
}

fn measure_dropdown<C>(dropdown: &DropdownSpec<'_, C>, theme: &Theme) -> Size {
    let label_height = if dropdown.label.is_empty() {
        0
    } else {
        8 * theme.text_scale as i32
    };
    let label_size = text_size(&dropdown.label, theme.text_scale);
    let width = dropdown.control_size.width.max(label_size.width);
    let height = dropdown.control_size.height + label_height.max(0) as u32;
    clamp_size_spec(dropdown.size, Size { width, height })
}

fn clamp_size_spec(spec: SizeSpec, measured: Size) -> Size {
    match spec {
        SizeSpec::Fixed(size) => Size {
            width: size.width.max(measured.width),
            height: size.height.max(measured.height),
        },
        _ => measured,
    }
}

fn grid_cell_minimums<C>(grid: &GridSpec<'_, C>, theme: &Theme) -> (u32, u32) {
    let mut max_child = Size {
        width: 0,
        height: 0,
    };
    for child in &grid.children {
        let size = measure_node(child, theme);
        max_child.width = max_child.width.max(size.width);
        max_child.height = max_child.height.max(size.height);
    }
    (
        grid.cell_size.width.max(max_child.width),
        grid.cell_size.height.max(max_child.height),
    )
}

fn render_panel<C>(
    panel: &mut PanelSpec<'_, C>,
    rect: Rect,
    ui: &mut Ui<'_>,
    theme: &Theme,
    ctx: &mut C,
) {
    let background = panel.background.unwrap_or(theme.knob_fill);
    let outline = panel.outline.unwrap_or(theme.knob_outline);
    let canvas = ui.canvas();
    canvas.fill_rect(rect, background);
    canvas.stroke_rect(rect, 1, outline);

    let header_height = panel
        .header_height
        .unwrap_or_else(|| panel_header_height(panel.title.as_deref(), theme));
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
    let mut min_main_sizes = Vec::with_capacity(flex.children.len());
    let mut cross_sizes = Vec::with_capacity(flex.children.len());
    let mut fill_count = 0i32;
    let mut min_total = 0i32;

    for child in &flex.children {
        let size = measure_node(child, theme);
        let (main, cross) = match axis {
            Axis::Horizontal => (size.width as i32, size.height as i32),
            Axis::Vertical => (size.height as i32, size.width as i32),
        };
        min_main_sizes.push(main);
        cross_sizes.push(cross);
        if matches!(child_size_spec(child), SizeSpec::Fill) {
            fill_count += 1;
        }
        min_total += main;
    }

    let gap_total = flex.gap.max(0) * (flex.children.len().saturating_sub(1) as i32);
    let available_main = match axis {
        Axis::Horizontal => rect.size.width as i32,
        Axis::Vertical => rect.size.height as i32,
    };
    let main_padding = match axis {
        Axis::Horizontal => flex.padding.left + flex.padding.right,
        Axis::Vertical => flex.padding.top + flex.padding.bottom,
    };
    let available_main = available_main - gap_total - main_padding;
    let remaining = (available_main - min_total).max(0);
    let fill_extra = if fill_count > 0 {
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
            main = main + fill_extra;
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
                Axis::Vertical => rect.origin.x + flex.padding.left + (cross_available - cross) / 2,
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
    let (cell_width, cell_height) = grid_cell_minimums(grid, theme);
    let base = Point {
        x: rect.origin.x + grid.padding.left,
        y: rect.origin.y + grid.padding.top,
    };
    for (index, child) in grid.children.iter_mut().enumerate() {
        let idx = index as i32;
        let row = idx / columns;
        let col = idx % columns;
        let origin = Point {
            x: base.x + col * (cell_width as i32 + grid.gap.max(0)),
            y: base.y + row * (cell_height as i32 + grid.gap.max(0)),
        };
        let child_rect = Rect {
            origin,
            size: Size {
                width: cell_width,
                height: cell_height,
            },
        };
        render_node(child, child_rect, ui, theme, ctx);
    }
}

fn render_label(label: &LabelSpec, rect: Rect, ui: &mut Ui<'_>, theme: &Theme) {
    let color = label.color.unwrap_or(theme.text);
    let pos = rect.origin;
    ui.canvas()
        .draw_text(pos, &label.text, color, theme.text_scale);
}

fn render_absolute<C>(
    absolute: &mut AbsoluteSpec<'_, C>,
    rect: Rect,
    ui: &mut Ui<'_>,
    theme: &Theme,
    ctx: &mut C,
) {
    for child in absolute.children.iter_mut() {
        let size = measure_node(&child.node, theme);
        let child_rect = Rect {
            origin: Point {
                x: rect.origin.x + child.origin.x,
                y: rect.origin.y + child.origin.y,
            },
            size,
        };
        render_node(&mut child.node, child_rect, ui, theme, ctx);
    }
}

fn render_knob<C>(
    knob: &mut KnobSpec<'_, C>,
    rect: Rect,
    ui: &mut Ui<'_>,
    _theme: &Theme,
    ctx: &mut C,
) {
    let id = WidgetId::from_label(&knob.key);
    let mut value = knob.value;
    let value_label = knob
        .value_label
        .clone()
        .unwrap_or_else(|| format_knob_value(value));
    let response =
        ui.knob_with_labels_in_rect(id, &knob.label, &value_label, &mut value, knob.range, rect);
    if let Some(callback) = knob.on_interaction.as_mut() {
        callback(ctx, KnobEvent { value, response });
    }
}

fn render_slider<C>(
    slider: &mut SliderSpec<'_, C>,
    rect: Rect,
    ui: &mut Ui<'_>,
    _theme: &Theme,
    ctx: &mut C,
) {
    let id = WidgetId::from_label(&slider.key);
    let mut value = slider.value;
    let response = ui.slider_in_rect(
        id,
        &slider.label,
        &mut value,
        slider.range,
        slider.control_size,
        rect,
    );
    if let Some(callback) = slider.on_interaction.as_mut() {
        callback(ctx, SliderEvent { value, response });
    }
}

fn render_toggle<C>(
    toggle: &mut ToggleSpec<'_, C>,
    rect: Rect,
    ui: &mut Ui<'_>,
    _theme: &Theme,
    ctx: &mut C,
) {
    let id = WidgetId::from_label(&toggle.key);
    let mut value = toggle.value;
    let response = ui.toggle_in_rect(id, &toggle.label, &mut value, toggle.control_size, rect);
    if let Some(callback) = toggle.on_interaction.as_mut() {
        callback(ctx, ToggleEvent { value, response });
    }
}

fn render_button<C>(
    button: &mut ButtonSpec<'_, C>,
    rect: Rect,
    ui: &mut Ui<'_>,
    _theme: &Theme,
    ctx: &mut C,
) {
    let id = WidgetId::from_label(&button.key);
    let response = ui.button_in_rect(id, &button.label, button.control_size, rect);
    if let Some(callback) = button.on_interaction.as_mut() {
        callback(ctx, ButtonEvent { response });
    }
}

fn render_dropdown<C>(
    dropdown: &mut DropdownSpec<'_, C>,
    rect: Rect,
    ui: &mut Ui<'_>,
    _theme: &Theme,
    ctx: &mut C,
) {
    let id = WidgetId::from_label(&dropdown.key);
    let mut selected = dropdown.selected;
    let option_refs: Vec<&str> = dropdown.options.iter().map(|s| s.as_str()).collect();
    let response = ui.dropdown_in_rect(
        id,
        &dropdown.label,
        &option_refs,
        &mut selected,
        dropdown.control_size,
        rect,
    );
    if let Some(callback) = dropdown.on_interaction.as_mut() {
        callback(ctx, DropdownEvent { selected, response });
    }
}

fn render_region<C>(
    region: &mut RegionSpec<'_, C>,
    rect: Rect,
    ui: &mut Ui<'_>,
    _theme: &Theme,
    ctx: &mut C,
) {
    let response = ui.region_with_key(&region.key, rect);
    if let Some(draw) = region.draw.as_mut() {
        draw(ui.canvas(), rect, ctx, response);
    }
    if let Some(callback) = region.on_interaction.as_mut() {
        callback(ctx, RegionEvent { response });
    }
}

fn render_indicator(indicator: &IndicatorSpec, rect: Rect, ui: &mut Ui<'_>, _theme: &Theme) {
    ui.indicator(rect, indicator.active);
}

fn panel_header_height(title: Option<&str>, theme: &Theme) -> i32 {
    if title.is_some() {
        (8 * theme.text_scale as i32 + 4).max(0)
    } else {
        0
    }
}

fn format_knob_value(value: f32) -> String {
    let mut text = if value.abs() >= 1.0 {
        format!("{value:.2}")
    } else if value.abs() >= 0.1 {
        format!("{value:.3}")
    } else {
        format!("{value:.4}")
    };
    if let Some(dot) = text.find('.') {
        while text.ends_with('0') && text.len() > dot + 2 {
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
        Node::Absolute(absolute) => absolute.size,
        Node::Label(label) => label.size,
        Node::Spacer(_) => SizeSpec::Fixed(Size {
            width: 0,
            height: 0,
        }),
        Node::Knob(knob) => knob.size,
        Node::Slider(slider) => slider.size,
        Node::Toggle(toggle) => toggle.size,
        Node::Button(button) => button.size,
        Node::Dropdown(dropdown) => dropdown.size,
        Node::Region(region) => SizeSpec::Fixed(region.size),
        Node::Indicator(indicator) => SizeSpec::Fixed(indicator.size),
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
                content: Box::new(Node::Panel(PanelSpec {
                    key: "panel".to_string(),
                    title: None,
                    padding: 0,
                    background: None,
                    outline: None,
                    header_height: None,
                    size: SizeSpec::Auto,
                    content: Box::new(Node::Row(FlexSpec {
                        size: SizeSpec::Auto,
                        gap: 4,
                        padding: Padding::uniform(2),
                        align: Align::Start,
                        children: vec![
                            Node::Spacer(SpacerSpec {
                                size: Size {
                                    width: 10,
                                    height: 8,
                                },
                            }),
                            Node::Spacer(SpacerSpec {
                                size: Size {
                                    width: 6,
                                    height: 12,
                                },
                            }),
                        ],
                    })),
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
                content: Box::new(Node::Panel(PanelSpec {
                    key: "panel".to_string(),
                    title: None,
                    padding: 0,
                    background: None,
                    outline: None,
                    header_height: None,
                    size: SizeSpec::Auto,
                    content: Box::new(Node::Widget(WidgetSpec {
                        key: "widget".to_string(),
                        size: SizeSpec::Fixed(Size {
                            width: 20,
                            height: 10,
                        }),
                        render: Box::new(|_ui, rect, _ctx: &mut ()| {
                            rect_seen.set(Some(rect));
                        }),
                    })),
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

    #[test]
    fn knob_measurement_matches_default_knob_diameter_contract() {
        let label = "PITCH COUPLING";
        let value_label = "100%";
        let spec = UiSpec {
            root: RootFrameSpec {
                key: "root".to_string(),
                title: None,
                padding: 0,
                content: Box::new(Node::Panel(PanelSpec {
                    key: "panel".to_string(),
                    title: None,
                    padding: 0,
                    background: None,
                    outline: None,
                    header_height: None,
                    size: SizeSpec::Auto,
                    content: Box::new(Node::Knob(KnobSpec {
                        key: "knob".to_string(),
                        label: label.to_string(),
                        value_label: Some(value_label.to_string()),
                        value: 0.5,
                        range: (0.0, 1.0),
                        size: SizeSpec::Auto,
                        on_interaction: None,
                    })),
                })),
            },
        };
        let theme = Theme::default();
        let size = measure(&spec, &theme);
        let expected_width = text_size(label, theme.text_scale)
            .width
            .max(text_size(value_label, theme.text_scale).width)
            .max(crate::ui::DEFAULT_KNOB_DIAMETER as u32);
        let expected_height = crate::ui::DEFAULT_KNOB_DIAMETER as u32
            + (8 * theme.text_scale as i32 * 2 + 4 * theme.text_scale as i32 * 2).max(0) as u32;
        assert_eq!(size.width, expected_width);
        assert_eq!(size.height, expected_height);
    }

    #[test]
    fn widget_outside_panel_returns_error() {
        let spec = UiSpec {
            root: RootFrameSpec {
                key: "root".to_string(),
                title: None,
                padding: 0,
                content: Box::new(Node::Label(LabelSpec {
                    text: "Oops".to_string(),
                    size: SizeSpec::Auto,
                    color: None,
                })),
            },
        };
        let theme = Theme::default();
        let error = measure_checked(&spec, &theme).expect_err("expected validation error");
        assert_eq!(
            error,
            DeclarativeError::WidgetOutsidePanel { node_kind: "Label" }
        );
    }

    #[test]
    fn legacy_measure_is_best_effort_on_invalid_tree() {
        let spec = UiSpec {
            root: RootFrameSpec {
                key: "root".to_string(),
                title: None,
                padding: 0,
                content: Box::new(Node::Label(LabelSpec {
                    text: "Fallback".to_string(),
                    size: SizeSpec::Auto,
                    color: None,
                })),
            },
        };
        let theme = Theme::default();
        let size = measure(&spec, &theme);
        assert!(size.width > 0);
        assert!(size.height > 0);
    }
}
