/// Label specification.
#[derive(Clone, Debug)]
pub struct LabelSpec {
    /// Label text.
    pub text: String,
    /// Optional text color override.
    pub color: Option<Color>,
    /// Layout constraints.
    pub layout: LayoutBox,
}

impl LabelSpec {
    /// Create a text label.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: None,
            layout: LayoutBox::auto(),
        }
    }

    /// Override layout constraints.
    pub fn layout(mut self, layout: LayoutBox) -> Self {
        self.layout = layout;
        self
    }
}
