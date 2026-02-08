
/// Resolved root-frame rendering metadata for a frame.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RootRenderPlan {
    /// Root layout size in design-space pixels.
    pub layout_size: Size,
    /// Final resolved root scale factor.
    pub resolved_scale: f32,
    /// Transform used to map design-space drawing onto the surface.
    pub transform: RootTransform,
}

/// Root frame scaling behavior.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RootScaleMode {
    /// Disable root-level scaling and render at authored size.
    #[default]
    None,
    /// Fit the authored design size to the current window using root-canvas
    /// transform scaling.
    ///
    /// When host and design aspect ratios differ, scaling is resolved per axis
    /// so content fills the available window area.
    UniformFit,
}

/// Declarative UI tree describing a window.
#[derive(Clone, Debug)]
pub struct UiSpec {
    /// Root frame definition.
    pub root: RootFrameSpec,
}

impl UiSpec {
    /// Build a UI spec from a root frame.
    pub fn new(root: RootFrameSpec) -> Self {
        Self { root }
    }
}

/// Root frame definition for a declarative window.
#[derive(Clone, Debug)]
pub struct RootFrameSpec {
    /// Stable frame key.
    pub key: String,
    /// Optional title displayed in the header.
    pub title: Option<String>,
    /// Padding inside the frame.
    pub padding: i32,
    /// Root sizing constraints.
    pub layout: LayoutBox,
    /// Optional design-token overrides for this frame.
    pub tokens: Option<ThemeTokens>,
    /// Optional baseline design size used for uniform fit scaling.
    pub design_size: Option<Size>,
    /// Root-level scaling behavior.
    pub scale_mode: RootScaleMode,
    /// Optional zoom multiplier applied after scale mode resolution.
    pub zoom_override: Option<f32>,
    /// Root content node.
    pub content: Box<Node>,
}

impl RootFrameSpec {
    /// Create a root frame with a key and content node.
    pub fn new(key: impl Into<String>, content: Node) -> Self {
        Self {
            key: key.into(),
            title: None,
            padding: 12,
            layout: LayoutBox::auto(),
            tokens: None,
            design_size: None,
            scale_mode: RootScaleMode::None,
            zoom_override: None,
            content: Box::new(content),
        }
    }

    /// Set optional title text.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Override frame padding.
    pub fn padding(mut self, padding: i32) -> Self {
        self.padding = padding;
        self
    }

    /// Override root layout constraints.
    pub fn layout(mut self, layout: LayoutBox) -> Self {
        self.layout = layout;
        self
    }

    /// Override token set for this root.
    pub fn tokens(mut self, tokens: ThemeTokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    /// Set the baseline authored design size used by root scaling modes.
    pub fn design_size(mut self, size: Size) -> Self {
        self.design_size = Some(size);
        self
    }

    /// Set root scaling behavior.
    pub fn scale_mode(mut self, mode: RootScaleMode) -> Self {
        self.scale_mode = mode;
        self
    }

    /// Set an optional zoom multiplier applied after base scaling.
    pub fn zoom_override(mut self, zoom: f32) -> Self {
        self.zoom_override = Some(zoom);
        self
    }
}

/// Create a root frame sized from host window bounds with a minimum floor.
///
/// This helper standardizes the "host-sized layout" model: root layout uses the
/// current host window size while preserving a minimum authored baseline.
/// Root-level scale mode remains [`RootScaleMode::None`].
pub fn root_frame_sized(
    key: impl Into<String>,
    content: Node,
    min_size: Size,
    window_size: Size,
) -> RootFrameSpec {
    let resolved_width = window_size.width.max(min_size.width);
    let resolved_height = window_size.height.max(min_size.height);
    RootFrameSpec::new(key, content).layout(
        LayoutBox::fixed(resolved_width, resolved_height).max(resolved_width, resolved_height),
    )
}

/// Layout nodes for the declarative UI tree.
#[derive(Clone, Debug)]
pub enum Node {
    /// Panel container.
    Panel(PanelSpec),
    /// Horizontal flex container.
    Row(FlexSpec),
    /// Vertical flex container.
    Column(FlexSpec),
    /// Grid container.
    Grid(GridSpec),
    /// Absolute positioning container.
    Absolute(AbsoluteSpec),
    /// Label node.
    Label(LabelSpec),
    /// Spacer node.
    Spacer(SpacerSpec),
    /// Knob control.
    Knob(KnobSpec),
    /// Slider control.
    Slider(SliderSpec),
    /// Toggle control.
    Toggle(ToggleSpec),
    /// Button control.
    Button(ButtonSpec),
    /// Dropdown control.
    Dropdown(DropdownSpec),
    /// Interactive region.
    Region(RegionSpec),
    /// Indicator node.
    Indicator(IndicatorSpec),
}
