/// Single-child slot node used as the required child envelope for containers.
#[derive(Clone, Debug)]
pub struct SlotSpec {
    /// The single child hosted by this slot.
    pub(crate) child: Box<Node>,
}

impl SlotSpec {
    /// Create a slot that hosts exactly one child node.
    pub fn new(child: Node) -> Self {
        Self {
            child: Box::new(child),
        }
    }

    /// Borrow the slot child node.
    pub fn child(&self) -> &Node {
        self.child.as_ref()
    }
}

/// Layout nodes for the declarative UI tree.
#[derive(Clone, Debug)]
pub enum Node {
    /// Container slot node.
    Slot(SlotSpec),
    /// Panel container.
    Panel(PanelSpec),
    /// Horizontal flex container.
    Row(FlexSpec),
    /// Vertical flex container.
    Column(FlexSpec),
    /// Grid container.
    Grid(GridSpec),
    /// Absolute positioning container.
    Absolute(AbsoluteSpec),
    /// Label node.
    Label(LabelSpec),
    /// Spacer node.
    Spacer(SpacerSpec),
    /// Knob control.
    Knob(KnobSpec),
    /// Slider control.
    Slider(SliderSpec),
    /// Toggle control.
    Toggle(ToggleSpec),
    /// Button control.
    Button(ButtonSpec),
    /// Dropdown control.
    Dropdown(DropdownSpec),
    /// Interactive region.
    Region(RegionSpec),
    /// Indicator node.
    Indicator(IndicatorSpec),
}

impl Node {
    /// Wrap a node into a single-child slot if it is not already slotted.
    pub fn slot(child: Node) -> Self {
        match child {
            Node::Slot(slot) => Node::Slot(slot),
            other => Node::Slot(SlotSpec::new(other)),
        }
    }
}
