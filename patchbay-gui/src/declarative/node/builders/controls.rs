impl Node {
    /// Set explicit control size for control-widget nodes.
    ///
    /// Applies to knob/slider/toggle/button/dropdown; other node kinds are
    /// returned unchanged.
    pub fn control_size(mut self, size: Size) -> Self {
        match &mut self {
            Self::Knob(knob) => knob.control_size = Some(size),
            Self::Slider(slider) => slider.control_size = Some(size),
            Self::Toggle(toggle) => toggle.control_size = Some(size),
            Self::Button(button) => button.control_size = Some(size),
            Self::Dropdown(dropdown) => dropdown.control_size = Some(size),
            _ => {}
        }
        self
    }

    /// Set selected option index for dropdown nodes.
    ///
    /// Non-dropdown node kinds are returned unchanged.
    pub fn selected(mut self, selected: usize) -> Self {
        if let Self::Dropdown(dropdown) = &mut self {
            dropdown.selected = selected;
        }
        self
    }

    /// Set option labels for dropdown nodes.
    ///
    /// Non-dropdown node kinds are returned unchanged.
    pub fn dropdown_option_labels(mut self, labels: Vec<String>) -> Self {
        if let Self::Dropdown(dropdown) = &mut self {
            dropdown.option_labels = Some(labels);
        }
        self
    }

    /// Override dropdown control background color for dropdown nodes.
    ///
    /// Non-dropdown node kinds are returned unchanged.
    pub fn dropdown_background_color(mut self, color: Color) -> Self {
        if let Self::Dropdown(dropdown) = &mut self {
            dropdown.background_override = Some(color);
        }
        self
    }

    /// Override dropdown hover fill color for dropdown nodes.
    ///
    /// Non-dropdown node kinds are returned unchanged.
    pub fn dropdown_hover_background_color(mut self, color: Color) -> Self {
        if let Self::Dropdown(dropdown) = &mut self {
            dropdown.hover_background_override = Some(color);
        }
        self
    }

    /// Override dropdown open-state control fill color for dropdown nodes.
    ///
    /// Non-dropdown node kinds are returned unchanged.
    pub fn dropdown_active_background_color(mut self, color: Color) -> Self {
        if let Self::Dropdown(dropdown) = &mut self {
            dropdown.active_background_override = Some(color);
        }
        self
    }

    /// Override dropdown control outline color for dropdown nodes.
    ///
    /// Non-dropdown node kinds are returned unchanged.
    pub fn dropdown_outline_color(mut self, color: Color) -> Self {
        if let Self::Dropdown(dropdown) = &mut self {
            dropdown.outline_override = Some(color);
        }
        self
    }

    /// Override dropdown label text color for dropdown nodes.
    ///
    /// Non-dropdown node kinds are returned unchanged.
    pub fn dropdown_text_color(mut self, color: Color) -> Self {
        if let Self::Dropdown(dropdown) = &mut self {
            dropdown.text_color_override = Some(color);
        }
        self
    }
}
