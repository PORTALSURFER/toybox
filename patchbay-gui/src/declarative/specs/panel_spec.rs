/// Panel container specification.
#[derive(Clone, Debug)]
pub struct PanelSpec {
    /// Stable panel key.
    pub key: String,
    /// Optional title.
    pub title: Option<String>,
    /// Inner padding.
    pub padding: i32,
    /// Optional background color override.
    pub background: Option<Color>,
    /// Optional outline color override.
    pub outline: Option<Color>,
    /// Optional header height override.
    pub header_height: Option<i32>,
    /// Layout constraints.
    pub layout: LayoutBox,
    /// Panel content.
    pub content: Box<Node>,
}

impl PanelSpec {
    /// Create a panel with key and content.
    pub fn new(key: impl Into<String>, content: Node) -> Self {
        Self {
            key: key.into(),
            title: None,
            padding: 12,
            background: None,
            outline: None,
            header_height: None,
            layout: LayoutBox::auto(),
            content: Box::new(content),
        }
    }

    /// Set panel title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Override panel padding.
    pub fn padding(mut self, padding: i32) -> Self {
        self.padding = padding;
        self
    }

    /// Override panel background color.
    pub fn background(mut self, background: Color) -> Self {
        self.background = Some(background);
        self
    }

    /// Override panel outline color.
    pub fn outline(mut self, outline: Color) -> Self {
        self.outline = Some(outline);
        self
    }

    /// Override panel header height.
    pub fn header_height(mut self, header_height: i32) -> Self {
        self.header_height = Some(header_height);
        self
    }

    /// Override panel layout constraints.
    pub fn layout(mut self, layout: LayoutBox) -> Self {
        self.layout = layout;
        self
    }
}
