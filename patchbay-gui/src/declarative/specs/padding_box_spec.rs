/// Single-slot padding container specification.
///
/// This container insets its content slot by parent-owned padding and places
/// the child at the padded origin.
#[derive(Clone, Debug)]
pub struct PaddingBoxSpec {
    /// Layout constraints for this container.
    pub(crate) layout: ContainerLayout,
    /// Padding applied before child placement.
    pub padding: EdgeInsets,
    /// Slotted child subtree.
    pub(crate) content: Box<Node>,
}

impl PaddingBoxSpec {
    /// Create a padding-box container with one slotted child.
    pub fn new(content: Node) -> Self {
        Self {
            layout: ContainerLayout::auto(),
            padding: EdgeInsets::default(),
            content: Box::new(Node::slot(content)),
        }
    }

    /// Override layout constraints.
    pub fn layout(mut self, layout: ContainerLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Set overflow behavior for the slotted child.
    pub fn overflow(mut self, overflow_policy: OverflowPolicy) -> Self {
        self.layout = self.layout.overflow(overflow_policy);
        self
    }

    /// Override container padding.
    pub fn padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;
        self
    }

    /// Set uniform container padding.
    pub fn pad_all(mut self, value: i32) -> Self {
        self.padding = EdgeInsets::all(value);
        self
    }

    /// Set horizontal and vertical container padding.
    pub fn pad_xy(mut self, horizontal: i32, vertical: i32) -> Self {
        self.padding = EdgeInsets::symmetric(horizontal, vertical);
        self
    }

    /// Borrow the content slot node.
    pub fn content(&self) -> &Node {
        self.content.as_ref()
    }

    /// Borrow container layout constraints.
    pub fn container_layout(&self) -> ContainerLayout {
        self.layout
    }

    /// Return configured overflow behavior.
    pub fn overflow_policy(&self) -> OverflowPolicy {
        self.layout.overflow_policy()
    }
}
