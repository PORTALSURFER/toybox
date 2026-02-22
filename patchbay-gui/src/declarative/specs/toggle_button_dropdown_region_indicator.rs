/// Toggle widget specification.
#[derive(Clone, Debug)]
pub struct ToggleSpec {
    /// Stable widget key.
    pub key: String,
    /// Current value.
    pub value: bool,
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

impl ToggleSpec {
    /// Create a toggle.
    pub fn new(key: impl Into<String>, value: bool) -> Self {
        Self {
            key: key.into(),
            value,
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

    /// Resolve this toggle's colors from a widget color role.
    pub fn color_role(mut self, role: WidgetColorRole) -> Self {
        self.color_role = Some(role);
        self
    }

    /// Disable this toggle's interactions and render disabled styling.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Toggle focused styling for this toggle.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
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
    /// Optional label rendered by the button widget itself.
    ///
    /// When unset, the button renders without text and caller-composed content
    /// can still be layered above it.
    pub label: Option<String>,
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

impl ButtonSpec {
    /// Create a button.
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: None,
            control_size: None,
            color_role: None,
            disabled: false,
            focused: false,
            layout: LayoutBox::auto(),
        }
    }

    /// Override button label text.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Override control size.
    pub fn control_size(mut self, size: Size) -> Self {
        self.control_size = Some(size);
        self
    }

    /// Resolve this button's colors from a widget color role.
    pub fn color_role(mut self, role: WidgetColorRole) -> Self {
        self.color_role = Some(role);
        self
    }

    /// Disable this button's interactions and render disabled styling.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Toggle focused styling for this button.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
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
    /// Optional override for dropdown hover fill.
    pub hover_background_override: Option<Color>,
    /// Optional override for dropdown open-state control fill.
    pub active_background_override: Option<Color>,
    /// Optional override for the dropdown control outline color.
    pub outline_override: Option<Color>,
    /// Optional override for dropdown label text color.
    pub text_color_override: Option<Color>,
    /// Optional override for the selected option row fill in the open menu.
    pub selected_option_background_override: Option<Color>,
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
            hover_background_override: None,
            active_background_override: None,
            outline_override: None,
            text_color_override: None,
            selected_option_background_override: None,
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

    /// Override dropdown hover fill color.
    pub fn hover_background_color(mut self, color: Color) -> Self {
        self.hover_background_override = Some(color);
        self
    }

    /// Override dropdown open-state control fill color.
    pub fn active_background_color(mut self, color: Color) -> Self {
        self.active_background_override = Some(color);
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

    /// Override selected option row fill color in the open menu.
    pub fn selected_option_background_color(mut self, color: Color) -> Self {
        self.selected_option_background_override = Some(color);
        self
    }

    /// Override layout constraints.
    pub fn layout(mut self, layout: LayoutBox) -> Self {
        self.layout = layout;
        self
    }
}

/// Tab-bar widget specification.
#[derive(Clone, Debug)]
pub struct TabBarSpec {
    /// Stable widget key.
    pub key: String,
    /// Number of available tabs.
    pub tab_count: usize,
    /// Optional per-tab labels shown in segment order.
    ///
    /// When present, labels are mapped by index. Any missing trailing labels
    /// fall back to 1-based numeric labels.
    pub tab_labels: Option<Vec<String>>,
    /// Selected tab index.
    pub selected: usize,
    /// Optional explicit control size.
    pub control_size: Option<Size>,
    /// Optional role used to resolve interaction-state colors.
    pub color_role: Option<WidgetColorRole>,
    /// Disable pointer/keyboard interaction and render disabled visuals.
    pub disabled: bool,
    /// Render focus affordances for keyboard/selection focus.
    pub focused: bool,
    /// Layout constraints.
    pub layout: LayoutBox,
}

impl TabBarSpec {
    /// Create a tab bar.
    pub fn new(
        key: impl Into<String>,
        tab_count: usize,
        selected: usize,
    ) -> Self {
        Self {
            key: key.into(),
            tab_count,
            tab_labels: None,
            selected,
            control_size: None,
            color_role: None,
            disabled: false,
            focused: false,
            layout: LayoutBox::auto(),
        }
    }

    /// Override tab labels.
    pub fn tab_labels(mut self, labels: Vec<String>) -> Self {
        self.tab_labels = Some(labels);
        self
    }

    /// Override control size.
    pub fn control_size(mut self, size: Size) -> Self {
        self.control_size = Some(size);
        self
    }

    /// Resolve this tab-bar's colors from a widget color role.
    pub fn color_role(mut self, role: WidgetColorRole) -> Self {
        self.color_role = Some(role);
        self
    }

    /// Disable this tab-bar's interactions and render disabled styling.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Toggle focused styling for this tab-bar.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
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
    /// Layout constraints.
    pub layout: LayoutBox,
    /// Region-relative draw commands rendered before interaction handling.
    pub draw: Vec<DrawCommand>,
}

impl RegionSpec {
    /// Create an interactive region.
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            layout: LayoutBox::auto(),
            draw: Vec::new(),
        }
    }

    /// Override layout constraints.
    pub fn layout(mut self, layout: LayoutBox) -> Self {
        self.layout = layout;
        self
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
    /// Layout constraints.
    pub layout: LayoutBox,
    /// Active state.
    pub active: bool,
}

impl IndicatorSpec {
    /// Create an indicator.
    pub const fn new(active: bool) -> Self {
        Self {
            layout: LayoutBox::auto(),
            active,
        }
    }

    /// Override layout constraints.
    pub const fn layout(mut self, layout: LayoutBox) -> Self {
        self.layout = layout;
        self
    }
}
