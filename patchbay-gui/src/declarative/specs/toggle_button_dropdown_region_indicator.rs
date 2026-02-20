/// Toggle widget specification.
#[derive(Clone, Debug)]
pub struct ToggleSpec {
    /// Stable widget key.
    pub key: String,
    /// Current value.
    pub value: bool,
    /// Optional explicit control size.
    pub control_size: Option<Size>,
    /// Layout constraints.
    pub layout: LayoutBox,
}

impl ToggleSpec {
    /// Create a toggle.
    pub fn new(key: impl Into<String>, value: bool) -> Self {
        Self {
            key: key.into(),
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
    /// Optional explicit control size.
    pub control_size: Option<Size>,
    /// Layout constraints.
    pub layout: LayoutBox,
}

impl ButtonSpec {
    /// Create a button.
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
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
    /// Number of available options.
    pub option_count: usize,
    /// Optional per-option labels shown in the dropdown menu and closed state.
    ///
    /// When present, labels are mapped by index. Any missing trailing labels
    /// fall back to 1-based numeric labels.
    pub option_labels: Option<Vec<String>>,
    /// Selected index.
    pub selected: usize,
    /// Optional explicit control size.
    pub control_size: Option<Size>,
    /// Optional override for the dropdown control background fill.
    pub background_override: Option<Color>,
    /// Optional override for the dropdown control outline color.
    pub outline_override: Option<Color>,
    /// Optional override for dropdown label text color.
    pub text_color_override: Option<Color>,
    /// Layout constraints.
    pub layout: LayoutBox,
}

impl DropdownSpec {
    /// Create a dropdown.
    pub fn new(
        key: impl Into<String>,
        option_count: usize,
        selected: usize,
    ) -> Self {
        Self {
            key: key.into(),
            option_count,
            option_labels: None,
            selected,
            control_size: None,
            background_override: None,
            outline_override: None,
            text_color_override: None,
            layout: LayoutBox::auto(),
        }
    }

    /// Override dropdown option labels.
    pub fn option_labels(mut self, labels: Vec<String>) -> Self {
        self.option_labels = Some(labels);
        self
    }

    /// Override control size.
    pub fn control_size(mut self, size: Size) -> Self {
        self.control_size = Some(size);
        self
    }

    /// Override dropdown control background fill color.
    pub fn background_color(mut self, color: Color) -> Self {
        self.background_override = Some(color);
        self
    }

    /// Override dropdown control outline color.
    pub fn outline_color(mut self, color: Color) -> Self {
        self.outline_override = Some(color);
        self
    }

    /// Override dropdown label text color.
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color_override = Some(color);
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
