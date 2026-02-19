/// Deterministic integer aspect ratio.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AspectRatio {
    /// Horizontal ratio component.
    pub width: u32,
    /// Vertical ratio component.
    pub height: u32,
}

impl AspectRatio {
    /// Create an aspect ratio from integer components.
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

/// Single-slot aspect-ratio container specification.
///
/// This container computes an aspect-constrained rectangle inside its own
/// bounds, aligns that rectangle deterministically, then lays out its slotted
/// child inside it.
#[derive(Clone, Debug)]
pub struct AspectBoxSpec {
    /// Layout constraints for this container.
    pub(crate) layout: ContainerLayout,
    /// Target aspect ratio for the child placement area.
    pub aspect_ratio: AspectRatio,
    /// Horizontal alignment for the aspect-constrained child area.
    pub align_x: SlotAlign,
    /// Vertical alignment for the aspect-constrained child area.
    pub align_y: SlotAlign,
    /// Slotted child subtree.
    pub(crate) content: Box<Node>,
}

impl AspectBoxSpec {
    /// Create an aspect-box container with one slotted child.
    pub fn new(content: Node, aspect_ratio: AspectRatio) -> Self {
        Self {
            layout: ContainerLayout::auto(),
            aspect_ratio,
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

    /// Set target aspect ratio.
    pub fn aspect_ratio(mut self, aspect_ratio: AspectRatio) -> Self {
        self.aspect_ratio = aspect_ratio;
        self
    }

    /// Set horizontal and vertical alignment for the aspect-constrained area.
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
