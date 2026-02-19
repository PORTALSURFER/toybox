/// Stack container specification.
///
/// Children are layered in slot order, where later slots render on top.
#[derive(Clone, Debug)]
pub struct StackSpec {
    /// Layout constraints for this container.
    pub(crate) layout: ContainerLayout,
    /// Stack padding applied before child placement.
    pub padding: EdgeInsets,
    /// Default horizontal alignment for children.
    pub align_x: SlotAlign,
    /// Default vertical alignment for children.
    pub align_y: SlotAlign,
    /// Ordered slotted children.
    pub(crate) children: Vec<Node>,
}

impl StackSpec {
    /// Create a stack specification from ordered children.
    pub fn new(children: Vec<Node>) -> Self {
        let children = children.into_iter().map(Node::slot).collect();
        Self {
            layout: ContainerLayout::auto(),
            padding: EdgeInsets::default(),
            align_x: SlotAlign::Start,
            align_y: SlotAlign::Start,
            children,
        }
    }

    /// Override layout constraints.
    pub fn layout(mut self, layout: ContainerLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Set stack overflow behavior.
    pub fn overflow(mut self, overflow_policy: OverflowPolicy) -> Self {
        self.layout = self.layout.overflow(overflow_policy);
        self
    }

    /// Override stack padding.
    pub fn padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;
        self
    }

    /// Set uniform stack padding.
    pub fn pad_all(mut self, value: i32) -> Self {
        self.padding = EdgeInsets::all(value);
        self
    }

    /// Set horizontal and vertical stack padding.
    pub fn pad_xy(mut self, horizontal: i32, vertical: i32) -> Self {
        self.padding = EdgeInsets::symmetric(horizontal, vertical);
        self
    }

    /// Set default alignment for children.
    pub fn align(mut self, align_x: SlotAlign, align_y: SlotAlign) -> Self {
        self.align_x = align_x;
        self.align_y = align_y;
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

    /// Return configured overflow behavior.
    pub fn overflow_policy(&self) -> OverflowPolicy {
        self.layout.overflow_policy()
    }
}

