/// Grid container specification.
#[derive(Clone, Debug)]
pub struct GridSpec {
    /// Layout constraints for this container.
    pub(crate) layout: ContainerLayout,
    /// Grid track template.
    pub template: GridTemplate,
    /// Slot children in row-major order.
    pub(crate) children: Vec<Node>,
    /// Grid semantic role.
    pub(crate) kind: GridKind,
}

impl GridSpec {
    /// Create a grid specification.
    pub fn new(template: GridTemplate, children: Vec<Node>) -> Self {
        let children = children
            .into_iter()
            .map(Node::slot)
            .collect();
        Self {
            layout: ContainerLayout::auto(),
            template,
            children,
            kind: GridKind::Standard,
        }
    }

    /// Override layout constraints.
    pub fn layout(mut self, layout: ContainerLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Borrow the ordered slot children.
    pub fn children(&self) -> &[Node] {
        &self.children
    }

    /// Borrow container layout constraints.
    pub fn container_layout(&self) -> ContainerLayout {
        self.layout
    }

    /// Borrow the semantic role for this grid.
    pub fn kind(&self) -> GridKind {
        self.kind
    }
}
