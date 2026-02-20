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

    /// Center text for text box nodes.
    ///
    /// Non-text-box node kinds are returned unchanged.
    pub fn text_align_center(mut self) -> Self {
        if let Self::TextBox(text_box) = &mut self {
            text_box.align = TextBoxAlign::Center;
        }
        self
    }

    /// Enable editable text behavior on text box nodes.
    ///
    /// Non-text-box node kinds are returned unchanged.
    pub fn text_editable(mut self, key: impl Into<String>, editing: bool) -> Self {
        if let Self::TextBox(text_box) = &mut self {
            text_box.edit = Some(TextBoxEditSpec {
                key: key.into(),
                editing,
                max_chars: 64,
            });
        }
        self
    }

    /// Set editable text maximum characters for text box nodes.
    ///
    /// Non-text-box node kinds are returned unchanged.
    pub fn text_edit_max_chars(mut self, max_chars: usize) -> Self {
        if let Self::TextBox(text_box) = &mut self
            && let Some(edit) = text_box.edit.as_mut()
        {
            edit.max_chars = max_chars.max(1);
        }
        self
    }
}
