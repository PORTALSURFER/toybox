
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

/// Toggle widget specification.
#[derive(Clone, Debug)]
pub struct ToggleSpec {
    /// Stable widget key.
    pub key: String,
    /// Label displayed above the toggle.
    pub label: String,
    /// Current value.
    pub value: bool,
    /// Optional explicit control size.
    pub control_size: Option<Size>,
    /// Layout constraints.
    pub layout: LayoutBox,
}

impl ToggleSpec {
    /// Create a toggle.
    pub fn new(key: impl Into<String>, label: impl Into<String>, value: bool) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            value,
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

/// Button widget specification.
#[derive(Clone, Debug)]
pub struct ButtonSpec {
    /// Stable widget key.
    pub key: String,
    /// Button label.
    pub label: String,
    /// Optional explicit control size.
    pub control_size: Option<Size>,
    /// Layout constraints.
    pub layout: LayoutBox,
}

impl ButtonSpec {
    /// Create a button.
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
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

/// Dropdown widget specification.
#[derive(Clone, Debug)]
pub struct DropdownSpec {
    /// Stable widget key.
    pub key: String,
    /// Label displayed above the dropdown.
    pub label: String,
    /// Options list.
    pub options: Vec<String>,
    /// Selected index.
    pub selected: usize,
    /// Optional explicit control size.
    pub control_size: Option<Size>,
    /// Layout constraints.
    pub layout: LayoutBox,
}

impl DropdownSpec {
    /// Create a dropdown.
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        options: Vec<String>,
        selected: usize,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            options,
            selected,
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

/// Interactive region specification.
#[derive(Clone, Debug)]
pub struct RegionSpec {
    /// Stable widget key.
    pub key: String,
    /// Region size.
    pub size: Size,
    /// Region-relative draw commands rendered before interaction handling.
    pub draw: Vec<DrawCommand>,
}

impl RegionSpec {
    /// Create an interactive region.
    pub fn new(key: impl Into<String>, size: Size) -> Self {
        Self {
            key: key.into(),
            size,
            draw: Vec::new(),
        }
    }

    /// Override region draw commands.
    pub fn draw_commands(mut self, draw: Vec<DrawCommand>) -> Self {
        self.draw = draw;
        self
    }
}

/// Indicator specification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IndicatorSpec {
    /// Indicator size.
    pub size: Size,
    /// Active state.
    pub active: bool,
}

impl IndicatorSpec {
    /// Create an indicator.
    pub const fn new(size: Size, active: bool) -> Self {
        Self { size, active }
    }
}

/// Core color token set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorTokens {
    /// Window background.
    pub background: Color,
    /// Surface fill.
    pub surface: Color,
    /// Border color.
    pub border: Color,
    /// Primary text.
    pub text: Color,
    /// Accent color.
    pub accent: Color,
}

impl Default for ColorTokens {
    fn default() -> Self {
        Self::main()
    }
}

impl ColorTokens {
    /// Build declarative color tokens from a semantic palette.
    pub const fn from_palette(palette: MainPalette) -> Self {
        Self {
            background: palette.background_primary,
            surface: palette.background_secondary,
            border: palette.ui_secondary,
            text: palette.text_primary,
            accent: palette.accent_focus,
        }
    }

    /// Return the canonical declarative color token set.
    pub const fn main() -> Self {
        Self::from_palette(MainPalette::main())
    }
}

/// Typography token set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypographyTokens {
    /// Bitmap text scale.
    pub text_scale: u32,
}

impl Default for TypographyTokens {
    fn default() -> Self {
        Self { text_scale: 2 }
    }
}

/// Spacing token set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpacingTokens {
    /// Tiny spacing.
    pub xs: i32,
    /// Small spacing.
    pub sm: i32,
    /// Medium spacing.
    pub md: i32,
    /// Large spacing.
    pub lg: i32,
}
