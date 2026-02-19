/// Editable text-box interaction configuration.
#[derive(Clone, Debug)]
pub struct TextBoxEditSpec {
    /// Stable action key used for edit events.
    pub key: String,
    /// True when host state currently keeps this text box in edit mode.
    pub editing: bool,
    /// Maximum accepted character count.
    pub max_chars: usize,
}

/// Text box widget specification.
#[derive(Clone, Debug)]
pub struct TextBoxSpec {
    /// Display text.
    pub text: String,
    /// Optional text color override.
    pub color: Option<Color>,
    /// Layout constraints.
    pub layout: LayoutBox,
    /// Optional editable interaction contract.
    pub edit: Option<TextBoxEditSpec>,
}

impl TextBoxSpec {
    /// Create a display-only text box.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: None,
            layout: LayoutBox::auto(),
            edit: None,
        }
    }

    /// Override layout constraints.
    pub fn layout(mut self, layout: LayoutBox) -> Self {
        self.layout = layout;
        self
    }

    /// Enable editable behavior for this text box.
    pub fn editable(mut self, key: impl Into<String>, editing: bool) -> Self {
        self.edit = Some(TextBoxEditSpec {
            key: key.into(),
            editing,
            max_chars: 64,
        });
        self
    }

    /// Override maximum character count for editable text.
    pub fn max_chars(mut self, max_chars: usize) -> Self {
        if let Some(edit) = self.edit.as_mut() {
            edit.max_chars = max_chars.max(1);
        }
        self
    }
}
