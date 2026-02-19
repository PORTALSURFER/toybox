/// Flex container specification.
#[derive(Clone, Debug)]
pub struct FlexSpec {
    /// Layout constraints for this container.
    pub(crate) layout: ContainerLayout,
    /// Gap between children.
    pub gap: i32,
    /// Container padding.
    pub padding: EdgeInsets,
    /// Cross-axis alignment.
    pub align: Align,
    /// Main-axis distribution.
    pub justify: Justify,
    /// Slot children.
    pub(crate) children: Vec<Node>,
}

impl FlexSpec {
    /// Create a row spec.
    pub fn row(children: Vec<Node>) -> Self {
        let children = children
            .into_iter()
            .map(Node::slot)
            .collect();
        Self {
            layout: ContainerLayout::auto(),
            gap: 12,
            padding: EdgeInsets::default(),
            align: Align::Start,
            justify: Justify::Start,
            children,
        }
    }

    /// Create a column spec.
    pub fn column(children: Vec<Node>) -> Self {
        let children = children
            .into_iter()
            .map(Node::slot)
            .collect();
        Self {
            layout: ContainerLayout::auto(),
            gap: 12,
            padding: EdgeInsets::default(),
            align: Align::Start,
            justify: Justify::Start,
            children,
        }
    }

    /// Override layout constraints.
    pub fn layout(mut self, layout: ContainerLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Override gap.
    pub fn gap(mut self, gap: i32) -> Self {
        self.gap = gap;
        self
    }

    /// Override padding.
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

    /// Override cross-axis alignment.
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Override main-axis distribution.
    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    /// Align children to the cross-axis start.
    pub fn align_start(mut self) -> Self {
        self.align = Align::Start;
        self
    }

    /// Center children on the cross-axis.
    pub fn align_center(mut self) -> Self {
        self.align = Align::Center;
        self
    }

    /// Align children to the cross-axis end.
    pub fn align_end(mut self) -> Self {
        self.align = Align::End;
        self
    }

    /// Stretch children across the cross-axis.
    pub fn align_stretch(mut self) -> Self {
        self.align = Align::Stretch;
        self
    }

    /// Pack children at the main-axis start.
    pub fn justify_start(mut self) -> Self {
        self.justify = Justify::Start;
        self
    }

    /// Center children on the main axis.
    pub fn justify_center(mut self) -> Self {
        self.justify = Justify::Center;
        self
    }

    /// Pack children at the main-axis end.
    pub fn justify_end(mut self) -> Self {
        self.justify = Justify::End;
        self
    }

    /// Distribute available space between items.
    pub fn justify_space_between(mut self) -> Self {
        self.justify = Justify::SpaceBetween;
        self
    }

    /// Distribute available space around items.
    pub fn justify_space_around(mut self) -> Self {
        self.justify = Justify::SpaceAround;
        self
    }

    /// Distribute available space evenly across edges and gaps.
    pub fn justify_space_evenly(mut self) -> Self {
        self.justify = Justify::SpaceEvenly;
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
}
