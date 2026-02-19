/// Scroll-view container specification.
///
/// This container provides a clipped viewport over a single slotted child.
#[derive(Clone, Debug)]
pub struct ScrollViewSpec {
    /// Layout constraints for this container.
    pub(crate) layout: ContainerLayout,
    /// Viewport padding applied before clipping.
    pub padding: EdgeInsets,
    /// Horizontal scroll offset in pixels.
    pub offset_x: i32,
    /// Vertical scroll offset in pixels.
    pub offset_y: i32,
    /// Slotted content node.
    pub(crate) content: Box<Node>,
}

impl ScrollViewSpec {
    /// Create a scroll view that hosts a single content subtree.
    pub fn new(content: Node) -> Self {
        Self {
            layout: ContainerLayout::auto(),
            padding: EdgeInsets::default(),
            offset_x: 0,
            offset_y: 0,
            content: Box::new(Node::slot(content)),
        }
    }

    /// Override layout constraints.
    pub fn layout(mut self, layout: ContainerLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Set scroll-view overflow behavior.
    pub fn overflow(mut self, overflow_policy: OverflowPolicy) -> Self {
        self.layout = self.layout.overflow(overflow_policy);
        self
    }

    /// Set viewport padding.
    pub fn padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;
        self
    }

    /// Set uniform viewport padding.
    pub fn pad_all(mut self, value: i32) -> Self {
        self.padding = EdgeInsets::all(value);
        self
    }

    /// Set horizontal and vertical viewport padding.
    pub fn pad_xy(mut self, horizontal: i32, vertical: i32) -> Self {
        self.padding = EdgeInsets::symmetric(horizontal, vertical);
        self
    }

    /// Set horizontal scroll offset.
    pub fn offset_x(mut self, offset_x: i32) -> Self {
        self.offset_x = offset_x;
        self
    }

    /// Set vertical scroll offset.
    pub fn offset_y(mut self, offset_y: i32) -> Self {
        self.offset_y = offset_y;
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
