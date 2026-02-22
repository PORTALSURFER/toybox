/// Declarative spec for the reusable EQ attractor editing surface.
///
/// This widget renders a log-frequency curve preview and attractor handles,
/// then emits typed interaction actions through [`UiAction`].
#[derive(Clone, Debug, PartialEq)]
pub struct EqAttractorSurfaceSpec {
    /// Stable widget key.
    pub key: String,
    /// Current immutable model payload.
    pub model: EqAttractorSurfaceModel,
    /// Visual and smoothing options.
    pub style: EqAttractorSurfaceStyle,
    /// Layout constraints.
    pub layout: LayoutBox,
}

impl EqAttractorSurfaceSpec {
    /// Build a surface spec from a stable key, model, and style.
    pub fn new(
        key: impl Into<String>,
        model: EqAttractorSurfaceModel,
        style: EqAttractorSurfaceStyle,
    ) -> Self {
        Self {
            key: key.into(),
            model,
            style,
            layout: LayoutBox::auto(),
        }
    }

    /// Override model payload.
    pub fn model(mut self, model: EqAttractorSurfaceModel) -> Self {
        self.model = model;
        self
    }

    /// Override style payload.
    pub fn style(mut self, style: EqAttractorSurfaceStyle) -> Self {
        self.style = style;
        self
    }

    /// Override layout constraints.
    pub fn layout(mut self, layout: LayoutBox) -> Self {
        self.layout = layout;
        self
    }
}
