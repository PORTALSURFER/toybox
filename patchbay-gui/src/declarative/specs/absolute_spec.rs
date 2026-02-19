/// Absolute-positioned container specification.
#[derive(Clone, Debug)]
pub struct AbsoluteSpec {
    /// Layout constraints.
    pub layout: LayoutBox,
    /// Positioned children.
    pub(crate) children: Vec<AbsoluteChild>,
}

impl AbsoluteSpec {
    /// Create an absolute container.
    pub fn new(children: Vec<AbsoluteChild>) -> Self {
        Self {
            layout: LayoutBox::auto(),
            children,
        }
    }

    /// Override layout constraints.
    pub fn layout(mut self, layout: LayoutBox) -> Self {
        self.layout = layout;
        self
    }

    /// Borrow the ordered positioned children.
    pub fn children(&self) -> &[AbsoluteChild] {
        &self.children
    }
}

/// Positioned child node.
#[derive(Clone, Debug)]
pub struct AbsoluteChild {
    /// Child origin relative to the container.
    pub origin: Point,
    /// Slotted child node.
    pub(crate) node: Node,
}

impl AbsoluteChild {
    /// Create a positioned child.
    pub fn new(origin: Point, node: Node) -> Self {
        Self {
            origin,
            node: Node::slot(node),
        }
    }

    /// Borrow the positioned child node.
    pub fn node(&self) -> &Node {
        &self.node
    }
}
