/// Single-slot alignment container specification.
///
/// This container positions one slotted child inside its resolved rectangle
/// according to deterministic slot-alignment rules.
#[derive(Clone, Debug)]
pub struct AlignBoxSpec {
    /// Layout constraints for this container.
    pub(crate) layout: ContainerLayout,
    /// Horizontal alignment for child placement.
    pub align_x: SlotAlign,
    /// Vertical alignment for child placement.
    pub align_y: SlotAlign,
    /// Slotted child subtree.
    pub(crate) content: Box<Node>,
}

impl AlignBoxSpec {
    /// Create an alignment container with one slotted child.
    pub fn new(content: Node) -> Self {
        Self {
            layout: ContainerLayout::auto(),
            align_x: SlotAlign::Start,
            align_y: SlotAlign::Start,
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

    /// Set horizontal and vertical child alignment.
    pub fn align(mut self, align_x: SlotAlign, align_y: SlotAlign) -> Self {
        self.align_x = align_x;
        self.align_y = align_y;
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
