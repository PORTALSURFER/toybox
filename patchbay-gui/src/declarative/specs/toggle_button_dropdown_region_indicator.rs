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
