impl KnobSpec {
    /// Create a knob.
    pub fn new(
        key: impl Into<String>,
        value: f32,
        range: (f32, f32),
    ) -> Self {
        Self {
            key: key.into(),
            value,
            range,
            default_value: default_value_for_range(range),
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

    /// Override the value restored by double-click reset interactions.
    pub fn default_value(mut self, default_value: f32) -> Self {
        self.default_value = default_value;
        self
    }
}

/// Slider widget specification.
#[derive(Clone, Debug)]
pub struct SliderSpec {
    /// Stable widget key.
    pub key: String,
    /// Current value.
    pub value: f32,
    /// Value range.
    pub range: (f32, f32),
    /// Value restored by core double-click reset interactions.
    pub default_value: f32,
    /// Optional explicit control size.
    pub control_size: Option<Size>,
    /// Layout constraints.
    pub layout: LayoutBox,
}

impl SliderSpec {
    /// Create a slider.
    pub fn new(
        key: impl Into<String>,
        value: f32,
        range: (f32, f32),
    ) -> Self {
        Self {
            key: key.into(),
            value,
            range,
            default_value: default_value_for_range(range),
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

    /// Override the value restored by double-click reset interactions.
    pub fn default_value(mut self, default_value: f32) -> Self {
        self.default_value = default_value;
        self
    }
}

/// Resolve midpoint default values for control ranges.
fn default_value_for_range(range: (f32, f32)) -> f32 {
    let (min, max) = if range.0 <= range.1 {
        (range.0, range.1)
    } else {
        (range.1, range.0)
    };
    min + (max - min) * 0.5
}
