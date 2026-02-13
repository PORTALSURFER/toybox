impl Node {
    /// Set explicit control size for slider/toggle/button/dropdown nodes.
    ///
    /// Non-control node kinds and knobs are returned unchanged.
    pub fn control_size(mut self, size: Size) -> Self {
        match &mut self {
            Self::Slider(slider) => slider.control_size = Some(size),
            Self::Toggle(toggle) => toggle.control_size = Some(size),
            Self::Button(button) => button.control_size = Some(size),
            Self::Dropdown(dropdown) => dropdown.control_size = Some(size),
            _ => {}
        }
        self
    }

    /// Set value label text for knob nodes.
    ///
    /// Non-knob node kinds are returned unchanged.
    pub fn value_label(mut self, value_label: impl Into<String>) -> Self {
        if let Self::Knob(knob) = &mut self {
            knob.value_label = Some(value_label.into());
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
}
