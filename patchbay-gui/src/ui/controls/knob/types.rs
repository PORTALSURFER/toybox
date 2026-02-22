/// Immutable label inputs for knob rendering.
#[derive(Clone, Copy)]
struct KnobLabels<'a> {
    /// Name label displayed above the knob.
    name: &'a str,
    /// Value label displayed below the knob.
    value: &'a str,
}

impl<'a> KnobLabels<'a> {
    /// Build a new knob label pair.
    fn new(name: &'a str, value: &'a str) -> Self {
        Self { name, value }
    }

    /// Return the normalized uppercase name label.
    fn normalized_name(self) -> String {
        normalize_knob_name_label(self.name)
    }

    /// Return the normalized value label.
    fn normalized_value(self) -> String {
        normalize_knob_value_label(self.value)
    }

    /// Return `true` when the name label has visible text.
    fn has_name(self) -> bool {
        !self.name.trim().is_empty()
    }

    /// Return `true` when the value label has visible text.
    fn has_value(self) -> bool {
        !self.value.trim().is_empty()
    }
}

/// Inclusive value range used by knob interactions.
#[derive(Clone, Copy)]
struct KnobRange {
    /// Minimum permitted value.
    min: f32,
    /// Maximum permitted value.
    max: f32,
}

impl KnobRange {
    /// Build a normalized range from tuple input.
    fn from_tuple(range: (f32, f32)) -> Self {
        let (a, b) = range;
        if a <= b {
            Self { min: a, max: b }
        } else {
            Self { min: b, max: a }
        }
    }

    /// Return the span width with a small non-zero floor.
    fn span(self) -> f32 {
        (self.max - self.min).max(1.0e-6)
    }

    /// Clamp `value` into this range.
    fn clamp(self, value: f32) -> f32 {
        value.clamp(self.min, self.max)
    }

    /// Convert `value` into a 0..1 normalized position.
    fn normalized(self, value: f32) -> f32 {
        ((value - self.min) / self.span()).clamp(0.0, 1.0)
    }
}

/// Resolved geometry for a single knob block.
#[derive(Clone, Copy)]
struct KnobGeometry {
    /// Outer block rectangle including labels and dial hit area.
    block_rect: Rect,
    /// Dial rectangle used for drawing the core knob.
    knob_rect: Rect,
    /// Expanded interactive hit rectangle around the dial.
    hit_rect: Rect,
    /// Dial center point in canvas coordinates.
    center: Point,
    /// Dial core radius.
    radius: i32,
    /// Dial side length in pixels.
    knob_size: i32,
    /// Vertical gap between dial and labels.
    label_gap: i32,
}

/// Inputs for a single knob render pass.
#[derive(Clone, Copy)]
struct KnobRenderSpec<'a> {
    /// Stable widget id.
    id: WidgetId,
    /// Label content to render.
    labels: KnobLabels<'a>,
    /// Interaction range limits.
    range: KnobRange,
    /// Value restored when the knob is double-clicked.
    default_value: f32,
    /// Optional color variants for role-driven styling.
    color_variants: Option<ControlColorVariants>,
    /// Disable pointer interaction and render disabled visuals.
    disabled: bool,
    /// Render focus affordances.
    focused: bool,
}

impl<'a> KnobRenderSpec<'a> {
    /// Build a render spec from primitive knob arguments and explicit default.
    fn from_args_with_default(
        id: WidgetId,
        name_label: &'a str,
        value_label: &'a str,
        range: (f32, f32),
        default_value: f32,
    ) -> Self {
        Self {
            id,
            labels: KnobLabels::new(name_label, value_label),
            range: KnobRange::from_tuple(range),
            default_value,
            color_variants: None,
            disabled: false,
            focused: false,
        }
    }
}

/// Rectangle-scoped rendering controls for knobs.
#[derive(Clone, Copy)]
struct KnobRectSpec {
    /// Bounding rectangle for clipped knob rendering.
    rect: Rect,
    /// Preferred knob diameter before clamping to available space.
    desired_diameter: u32,
}

impl KnobRectSpec {
    /// Build a rect-scoped knob spec.
    fn new(rect: Rect, desired_diameter: u32) -> Self {
        Self {
            rect,
            desired_diameter,
        }
    }
}

/// Request payload for rendering a knob inside a fixed rectangle.
#[derive(Clone, Copy)]
pub(crate) struct KnobRectRenderRequest<'a> {
    /// Stable widget id.
    id: WidgetId,
    /// Name/value label pair rendered above/below the knob.
    labels: KnobLabels<'a>,
    /// Inclusive interaction range.
    range: (f32, f32),
    /// Value restored when the knob is double-clicked.
    default_value: f32,
    /// Preferred dial diameter before clamping.
    desired_diameter: u32,
    /// Rectangular section bounds for clipped rendering.
    rect: Rect,
    /// Explicit text scale override for this knob.
    text_scale: u32,
    /// Optional color variants for role-driven styling.
    color_variants: Option<ControlColorVariants>,
    /// Disable pointer interaction and render disabled visuals.
    disabled: bool,
    /// Render focus affordances.
    focused: bool,
}

impl<'a> KnobRectRenderRequest<'a> {
    /// Build a knob-in-rect request with default text scale.
    pub(crate) fn new(
        id: WidgetId,
        name_label: &'a str,
        value_label: &'a str,
        range: (f32, f32),
        desired_diameter: u32,
        rect: Rect,
    ) -> Self {
        Self {
            id,
            labels: KnobLabels::new(name_label, value_label),
            range,
            default_value: knob_default_value(range),
            desired_diameter,
            rect,
            text_scale: 1,
            color_variants: None,
            disabled: false,
            focused: false,
        }
    }

    /// Override text scale for the rendered knob labels.
    pub(crate) fn with_text_scale(mut self, text_scale: u32) -> Self {
        self.text_scale = text_scale.max(1);
        self
    }

    /// Override the value restored by knob double-click interactions.
    pub(crate) fn with_default_value(mut self, default_value: f32) -> Self {
        self.default_value = default_value;
        self
    }

    /// Override state variants used for role-driven knob rendering.
    pub(crate) fn with_color_variants(mut self, variants: ControlColorVariants) -> Self {
        self.color_variants = Some(variants);
        self
    }

    /// Disable pointer interaction and render disabled styling.
    pub(crate) fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Render focus affordances for the knob.
    pub(crate) fn with_focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

/// Resolve the midpoint default value for a knob range.
fn knob_default_value(range: (f32, f32)) -> f32 {
    let normalized = KnobRange::from_tuple(range);
    normalized.min + (normalized.max - normalized.min) * 0.5
}
