impl KnobSpec {
    /// Create a knob.
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        value: f32,
        range: (f32, f32),
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            value_label: None,
            value,
            range,
            layout: LayoutBox::auto(),
        }
    }

    /// Override value label.
    pub fn value_label(mut self, value_label: impl Into<String>) -> Self {
        self.value_label = Some(value_label.into());
        self
    }

    /// Override layout constraints.
    pub fn layout(mut self, layout: LayoutBox) -> Self {
        self.layout = layout;
        self
    }
}

/// Slider widget specification.
#[derive(Clone, Debug)]
pub struct SliderSpec {
    /// Stable widget key.
    pub key: String,
    /// Label displayed above the slider.
    pub label: String,
    /// Current value.
    pub value: f32,
    /// Value range.
    pub range: (f32, f32),
    /// Optional explicit control size.
    pub control_size: Option<Size>,
    /// Layout constraints.
    pub layout: LayoutBox,
}

impl SliderSpec {
    /// Create a slider.
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        value: f32,
        range: (f32, f32),
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            value,
            range,
            control_size: None,
            layout: LayoutBox::auto(),
        }
    }

    /// Override control size.
    pub fn control_size(mut self, size: Size) -> Self {
        self.control_size = Some(size);
        self
    }

    /// Override layout constraints.
    pub fn layout(mut self, layout: LayoutBox) -> Self {
        self.layout = layout;
        self
    }
}
