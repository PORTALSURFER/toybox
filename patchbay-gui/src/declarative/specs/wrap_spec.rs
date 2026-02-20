/// Wrap container specification.
///
/// Children flow along the horizontal axis and wrap into new rows.
#[derive(Clone, Debug)]
pub struct WrapSpec {
    /// Layout constraints for this container.
    pub(crate) layout: ContainerLayout,
    /// Container padding.
    pub padding: EdgeInsets,
    /// Main-axis distribution for each wrapped row.
    pub justify: Justify,
    /// Ordered slotted children.
    pub(crate) children: Vec<Node>,
}

impl WrapSpec {
    /// Create a wrap specification from ordered children.
    pub fn new(children: Vec<Node>) -> Self {
        let children = children.into_iter().map(Node::slot).collect();
        Self {
            layout: ContainerLayout::auto(),
            padding: EdgeInsets::default(),
            justify: Justify::Start,
            children,
        }
    }

    /// Override layout constraints.
    pub fn layout(mut self, layout: ContainerLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Set wrap overflow behavior.
    pub fn overflow(mut self, overflow_policy: OverflowPolicy) -> Self {
        self.layout = self.layout.overflow(overflow_policy);
        self
    }

    /// Set container padding.
    pub fn padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;
        self
    }

    /// Set uniform padding.
    pub fn pad_all(mut self, value: i32) -> Self {
        self.padding = EdgeInsets::all(value);
        self
    }

    /// Set horizontal and vertical padding.
    pub fn pad_xy(mut self, horizontal: i32, vertical: i32) -> Self {
        self.padding = EdgeInsets::symmetric(horizontal, vertical);
        self
    }

    /// Override row justification.
    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
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
