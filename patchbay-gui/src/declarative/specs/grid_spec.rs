/// Grid container specification.
#[derive(Clone, Debug)]
pub struct GridSpec {
    /// Layout constraints for this container.
    pub layout: LayoutBox,
    /// Grid track template.
    pub template: GridTemplate,
    /// Child nodes in row-major order.
    pub children: Vec<Node>,
    /// Grid semantic role.
    pub kind: GridKind,
}

impl GridSpec {
    /// Create a grid specification.
    pub fn new(template: GridTemplate, children: Vec<Node>) -> Self {
        Self {
            layout: LayoutBox::auto(),
            template,
            children,
            kind: GridKind::Standard,
        }
    }

    /// Override layout constraints.
    pub fn layout(mut self, layout: LayoutBox) -> Self {
        self.layout = layout;
        self
    }
}
