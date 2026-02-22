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
            color_role: None,
            disabled: false,
            focused: false,
            layout: LayoutBox::auto(),
        }
    }

    /// Override control size.
    pub fn control_size(mut self, size: Size) -> Self {
        self.control_size = Some(size);
        self
    }

    /// Resolve this knob's colors from a widget color role.
    pub fn color_role(mut self, role: WidgetColorRole) -> Self {
        self.color_role = Some(role);
        self
    }

    /// Disable this knob's interactions and render disabled styling.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Toggle focused styling for this knob.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
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
    /// Optional role used to resolve interaction-state colors.
    pub color_role: Option<WidgetColorRole>,
    /// Disable pointer interaction and render disabled visuals.
    pub disabled: bool,
    /// Render focus affordances for keyboard/selection focus.
    pub focused: bool,
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
            color_role: None,
            disabled: false,
            focused: false,
            layout: LayoutBox::auto(),
        }
    }

    /// Override control size.
    pub fn control_size(mut self, size: Size) -> Self {
        self.control_size = Some(size);
        self
    }

    /// Resolve this slider's colors from a widget color role.
    pub fn color_role(mut self, role: WidgetColorRole) -> Self {
        self.color_role = Some(role);
        self
    }

    /// Disable this slider's interactions and render disabled styling.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Toggle focused styling for this slider.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
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
