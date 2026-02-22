impl Node {
    /// Set explicit control size for control-widget nodes.
    ///
    /// Applies to knob/slider/toggle/button/dropdown/tab-bar; other node kinds are
    /// returned unchanged.
    pub fn control_size(mut self, size: Size) -> Self {
        match &mut self {
            Self::Knob(knob) => knob.control_size = Some(size),
            Self::Slider(slider) => slider.control_size = Some(size),
            Self::Toggle(toggle) => toggle.control_size = Some(size),
            Self::Button(button) => button.control_size = Some(size),
            Self::Dropdown(dropdown) => dropdown.control_size = Some(size),
            Self::TabBar(tab_bar) => tab_bar.control_size = Some(size),
            _ => {}
        }
        self
    }

    /// Set button label text for button nodes.
    ///
    /// Non-button node kinds are returned unchanged.
    pub fn button_label(mut self, label: impl Into<String>) -> Self {
        if let Self::Button(button) = &mut self {
            button.label = Some(label.into());
        }
        self
    }

    /// Set selected option/tab index for dropdown and tab-bar nodes.
    ///
    /// Non-dropdown/tab-bar node kinds are returned unchanged.
    pub fn selected(mut self, selected: usize) -> Self {
        match &mut self {
            Self::Dropdown(dropdown) => dropdown.selected = selected,
            Self::TabBar(tab_bar) => tab_bar.selected = selected,
            _ => {}
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

    /// Set tab labels for tab-bar nodes.
    ///
    /// Non-tab-bar node kinds are returned unchanged.
    pub fn tab_labels(mut self, labels: Vec<String>) -> Self {
        if let Self::TabBar(tab_bar) = &mut self {
            tab_bar.tab_labels = Some(labels);
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

    /// Override selected option row fill color for dropdown nodes.
    ///
    /// Non-dropdown node kinds are returned unchanged.
    pub fn dropdown_selected_option_background_color(
        mut self,
        color: Color,
    ) -> Self {
        if let Self::Dropdown(dropdown) = &mut self {
            dropdown.selected_option_background_override = Some(color);
        }
        self
    }

    /// Set double-click reset default value for knob and slider nodes.
    ///
    /// Non-knob/slider node kinds are returned unchanged.
    pub fn default_value(mut self, default_value: f32) -> Self {
        match &mut self {
            Self::Knob(knob) => knob.default_value = default_value,
            Self::Slider(slider) => slider.default_value = default_value,
            _ => {}
        }
        self
    }

    /// Set color role for knob/slider/toggle/button/tab-bar control nodes.
    ///
    /// Non-target node kinds are returned unchanged.
    pub fn color_role(mut self, role: WidgetColorRole) -> Self {
        match &mut self {
            Self::Knob(knob) => knob.color_role = Some(role),
            Self::Slider(slider) => slider.color_role = Some(role),
            Self::Toggle(toggle) => toggle.color_role = Some(role),
            Self::Button(button) => button.color_role = Some(role),
            Self::TabBar(tab_bar) => tab_bar.color_role = Some(role),
            _ => {}
        }
        self
    }

    /// Set disabled state for knob/slider/toggle/button/tab-bar control nodes.
    ///
    /// Non-target node kinds are returned unchanged.
    pub fn disabled(mut self, disabled: bool) -> Self {
        match &mut self {
            Self::Knob(knob) => knob.disabled = disabled,
            Self::Slider(slider) => slider.disabled = disabled,
            Self::Toggle(toggle) => toggle.disabled = disabled,
            Self::Button(button) => button.disabled = disabled,
            Self::TabBar(tab_bar) => tab_bar.disabled = disabled,
            _ => {}
        }
        self
    }

    /// Set focused state for knob/slider/toggle/button/tab-bar control nodes.
    ///
    /// Non-target node kinds are returned unchanged.
    pub fn focused(mut self, focused: bool) -> Self {
        match &mut self {
            Self::Knob(knob) => knob.focused = focused,
            Self::Slider(slider) => slider.focused = focused,
            Self::Toggle(toggle) => toggle.focused = focused,
            Self::Button(button) => button.focused = focused,
            Self::TabBar(tab_bar) => tab_bar.focused = focused,
            _ => {}
        }
        self
    }

    /// Override curve-editor model for curve-editor nodes.
    ///
    /// Non-curve-editor node kinds are returned unchanged.
    pub fn curve_model(mut self, model: CurveModel) -> Self {
        if let Self::CurveEditor(curve_editor) = &mut self {
            curve_editor.model = model;
        }
        self
    }

    /// Override curve-editor style payload for curve-editor nodes.
    ///
    /// Non-curve-editor node kinds are returned unchanged.
    pub fn curve_style(mut self, style: CurveEditorStyle) -> Self {
        if let Self::CurveEditor(curve_editor) = &mut self {
            curve_editor.style = style;
        }
        self
    }

    /// Override curve-editor interaction settings for curve-editor nodes.
    ///
    /// Non-curve-editor node kinds are returned unchanged.
    pub fn curve_interaction(mut self, interaction: CurveInteractionOptions) -> Self {
        if let Self::CurveEditor(curve_editor) = &mut self {
            curve_editor.interaction = interaction;
        }
        self
    }

    /// Override curve-editor optional playhead position for curve-editor nodes.
    ///
    /// Non-curve-editor node kinds are returned unchanged.
    pub fn curve_playhead_x(mut self, playhead_x: Option<f32>) -> Self {
        if let Self::CurveEditor(curve_editor) = &mut self {
            curve_editor.playhead_x = playhead_x;
        }
        self
    }

    /// Override EQ attractor surface model for EQ attractor surface nodes.
    ///
    /// Non-EQ-attractor-surface node kinds are returned unchanged.
    pub fn eq_attractor_model(mut self, model: EqAttractorSurfaceModel) -> Self {
        if let Self::EqAttractorSurface(surface) = &mut self {
            surface.model = model;
        }
        self
    }

    /// Override EQ attractor surface style for EQ attractor surface nodes.
    ///
    /// Non-EQ-attractor-surface node kinds are returned unchanged.
    pub fn eq_attractor_style(mut self, style: EqAttractorSurfaceStyle) -> Self {
        if let Self::EqAttractorSurface(surface) = &mut self {
            surface.style = style;
        }
        self
    }
}
