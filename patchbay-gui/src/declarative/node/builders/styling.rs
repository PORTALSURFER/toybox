impl Node {
    /// Set title for panel nodes.
    ///
    /// Non-panel node kinds are returned unchanged.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        if let Self::Panel(panel) = &mut self {
            panel.title = Some(title.into());
        }
        self
    }

    /// Set background color for panel nodes.
    ///
    /// Non-panel node kinds are returned unchanged.
    pub fn background(mut self, color: Color) -> Self {
        if let Self::Panel(panel) = &mut self {
            panel.background = Some(color);
        }
        self
    }

    /// Set outline color for panel nodes.
    ///
    /// Non-panel node kinds are returned unchanged.
    pub fn outline(mut self, color: Color) -> Self {
        if let Self::Panel(panel) = &mut self {
            panel.outline = Some(color);
        }
        self
    }

    /// Set text color for text box nodes.
    ///
    /// Non-text-box node kinds are returned unchanged.
    pub fn text_color(mut self, color: Color) -> Self {
        if let Self::TextBox(text_box) = &mut self {
            text_box.color = Some(color);
        }
        self
    }
}
