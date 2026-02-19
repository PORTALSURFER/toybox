/// Grid container specification.
#[derive(Clone, Debug)]
pub struct GridSpec {
    /// Layout constraints for this container.
    pub layout: LayoutBox,
    /// Grid track template.
    pub template: GridTemplate,
    /// Slot children in row-major order.
    pub(crate) children: Vec<Node>,
    /// Grid semantic role.
    pub kind: GridKind,
}

impl GridSpec {
    /// Create a grid specification.
    pub fn new(template: GridTemplate, children: Vec<Node>) -> Self {
        let children = children
            .into_iter()
            .map(Node::slot)
            .collect();
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

    /// Borrow the ordered slot children.
    pub fn children(&self) -> &[Node] {
        &self.children
    }
}
