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
    /// Fit the authored design size to the current window using root-canvas
    /// transform scaling while preserving aspect ratio. Content is centered in
    /// any remaining surface area.
    ///
    /// When host and design aspect ratios differ, remaining surface area is
    /// letterboxed.
    #[default]
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
    /// Runtime layout diagnostics detail mode.
    pub layout_diagnostics_mode: LayoutDiagnosticsMode,
    /// Root content slot.
    ///
    /// Root is treated as a special container and always stores exactly one
    /// slot child.
    pub(crate) content: Box<Node>,
}

impl RootFrameSpec {
    /// Create a root frame with a key and content node.
    ///
    /// The provided content is wrapped in a single slot so root obeys the
    /// container-slot grammar.
    pub fn new(key: impl Into<String>, content: Node) -> Self {
        Self {
            key: key.into(),
            title: None,
            padding: 12,
            layout: LayoutBox::auto(),
            tokens: None,
            design_size: None,
            scale_mode: RootScaleMode::UniformFit,
            zoom_override: None,
            layout_diagnostics_mode: LayoutDiagnosticsMode::EventsOnly,
            content: Box::new(Node::slot(content)),
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

    /// Set runtime layout diagnostics mode.
    pub fn layout_diagnostics_mode(mut self, mode: LayoutDiagnosticsMode) -> Self {
        self.layout_diagnostics_mode = mode;
        self
    }

    /// Borrow the root content slot node.
    pub fn content(&self) -> &Node {
        self.content.as_ref()
    }
}

/// Create a root frame using a fixed design resolution.
///
/// Layout and design space are both anchored to `design_size`. Runtime window
/// resize only changes render transform scale/offset.
pub fn root_frame_sized(
    key: impl Into<String>,
    content: Node,
    design_size: Size,
) -> RootFrameSpec {
    let design = Size {
        width: design_size.width.max(1),
        height: design_size.height.max(1),
    };
    RootFrameSpec::new(key, content)
        .layout(LayoutBox::fixed(design.width, design.height).max(design.width, design.height))
        .design_size(design)
        .scale_mode(RootScaleMode::UniformFit)
}
