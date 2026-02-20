/// Request payload for rendering a slider inside a fixed rectangle.
#[derive(Clone, Copy)]
pub(crate) struct SliderRectRenderRequest<'a> {
    /// Stable widget id.
    id: WidgetId,
    /// Label displayed above the slider track.
    label: &'a str,
    /// Inclusive value range.
    range: (f32, f32),
    /// Bounds used for clipping and placement.
    rect: Rect,
    /// Explicit text scale override for the label.
    text_scale: u32,
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
            rect,
            text_scale: 1,
        }
    }

    /// Override text scale for slider label rendering.
    pub(crate) fn with_text_scale(mut self, text_scale: u32) -> Self {
        self.text_scale = text_scale.max(1);
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
    pub(crate) fill: Color,
}

/// Public slider configuration shared across keyed and non-keyed slider APIs.
#[derive(Clone, Copy)]
pub struct SliderConfig {
    /// Inclusive slider value range.
    pub range: (f32, f32),
    /// Slider control footprint.
    pub size: Size,
}
