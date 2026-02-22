/// Request payload for rendering a slider inside a fixed rectangle.
#[derive(Clone, Copy)]
pub(crate) struct SliderRectRenderRequest<'a> {
    /// Stable widget id.
    id: WidgetId,
    /// Label displayed above the slider track.
    label: &'a str,
    /// Inclusive value range.
    range: (f32, f32),
    /// Value restored when the slider is double-clicked.
    default_value: f32,
    /// Bounds used for clipping and placement.
    rect: Rect,
    /// Explicit text scale override for the label.
    text_scale: u32,
    /// Optional color variants for role-driven styling.
    color_variants: Option<ControlColorVariants>,
    /// Disable pointer interaction and render disabled visuals.
    disabled: bool,
    /// Render focus affordances.
    focused: bool,
}

impl<'a> SliderRectRenderRequest<'a> {
    /// Build a slider-in-rect request with default text scale.
    pub(crate) fn new(
        id: WidgetId,
        label: &'a str,
        range: (f32, f32),
        rect: Rect,
    ) -> Self {
        Self {
            id,
            label,
            range,
            default_value: slider_default_value(range),
            rect,
            text_scale: 1,
            color_variants: None,
            disabled: false,
            focused: false,
        }
    }

    /// Override text scale for slider label rendering.
    pub(crate) fn with_text_scale(mut self, text_scale: u32) -> Self {
        self.text_scale = text_scale.max(1);
        self
    }

    /// Override the value restored by slider double-click interactions.
    pub(crate) fn with_default_value(mut self, default_value: f32) -> Self {
        self.default_value = default_value;
        self
    }

    /// Override state variants used for role-driven slider rendering.
    pub(crate) fn with_color_variants(mut self, variants: ControlColorVariants) -> Self {
        self.color_variants = Some(variants);
        self
    }

    /// Disable pointer interaction and render disabled styling.
    pub(crate) fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Render focus affordances for this slider.
    pub(crate) fn with_focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

/// Resolved layout geometry for one slider draw call.
#[derive(Clone, Copy)]
pub(crate) struct SliderLayoutResolved {
    /// Total vertical block consumed by slider + optional label.
    pub(crate) block_size: Size,
    /// Slider control rectangle.
    pub(crate) rect: Rect,
    /// Slider control height in pixels.
    pub(crate) height: i32,
}

/// Precomputed drawing geometry for slider visuals.
#[derive(Clone, Copy)]
pub(crate) struct SliderVisualState {
    /// Slider track rectangle.
    pub(crate) track_rect: Rect,
    /// Filled portion of the track.
    pub(crate) fill_rect: Rect,
    /// Handle center point.
    pub(crate) handle_center: Point,
    /// Handle radius in pixels.
    pub(crate) handle_radius: i32,
    /// Track fill color based on interaction state.
    pub(crate) track_fill: Color,
    /// Value fill color.
    pub(crate) value_fill: Color,
    /// Handle fill color.
    pub(crate) handle_fill: Color,
    /// Outline/focus ring color.
    pub(crate) outline: Color,
}

/// Public slider configuration shared across keyed and non-keyed slider APIs.
#[derive(Clone, Copy)]
pub struct SliderConfig {
    /// Inclusive slider value range.
    pub range: (f32, f32),
    /// Slider control footprint.
    pub size: Size,
}

/// Resolve the midpoint default value for a slider range.
fn slider_default_value(range: (f32, f32)) -> f32 {
    let (min, max) = if range.0 <= range.1 {
        (range.0, range.1)
    } else {
        (range.1, range.0)
    };
    min + (max - min) * 0.5
}
