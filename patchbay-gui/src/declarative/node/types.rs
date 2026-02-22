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
    /// Single-slot padding container.
    PaddingBox(PaddingBoxSpec),
    /// Single-slot alignment container.
    AlignBox(AlignBoxSpec),
    /// Single-slot aspect-ratio container.
    AspectBox(AspectBoxSpec),
    /// Horizontal flex container.
    Row(FlexSpec),
    /// Vertical flex container.
    Column(FlexSpec),
    /// Grid container.
    Grid(GridSpec),
    /// Absolute positioning container.
    Absolute(AbsoluteSpec),
    /// Stack overlay container.
    Stack(StackSpec),
    /// Scroll-view viewport container.
    ScrollView(ScrollViewSpec),
    /// Flow/wrap container.
    Wrap(WrapSpec),
    /// Width-based responsive switch container.
    SwitchLayout(SwitchLayoutSpec),
    /// Text box widget.
    TextBox(TextBoxSpec),
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
    /// Tab-bar control.
    TabBar(TabBarSpec),
    /// Curve editor widget.
    CurveEditor(CurveEditorSpec),
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

    /// Visit children in deterministic declaration order.
    pub(crate) fn for_each_child<'a>(&'a self, mut visit: impl FnMut(&'a Node)) {
        self.for_each_indexed_child(|_, child| visit(child));
    }

    /// Visit children with deterministic indices in declaration order.
    ///
    /// For `SwitchLayout`, fallback is visited after all explicit cases.
    pub(crate) fn for_each_indexed_child<'a>(&'a self, mut visit: impl FnMut(usize, &'a Node)) {
        match self {
            Node::Slot(slot) => visit(0, slot.child()),
            Node::Panel(panel) => visit(0, panel.content()),
            Node::PaddingBox(padding_box) => visit(0, padding_box.content()),
            Node::AlignBox(align_box) => visit(0, align_box.content()),
            Node::AspectBox(aspect_box) => visit(0, aspect_box.content()),
            Node::Row(flex) | Node::Column(flex) => {
                for (index, child) in flex.children.iter().enumerate() {
                    visit(index, child);
                }
            }
            Node::Grid(grid) => {
                for (index, child) in grid.children.iter().enumerate() {
                    visit(index, child);
                }
            }
            Node::Absolute(absolute) => {
                for (index, child) in absolute.children.iter().enumerate() {
                    visit(index, child.node());
                }
            }
            Node::Stack(stack) => {
                for (index, child) in stack.children.iter().enumerate() {
                    visit(index, child);
                }
            }
            Node::ScrollView(scroll_view) => visit(0, scroll_view.content()),
            Node::Wrap(wrap) => {
                for (index, child) in wrap.children.iter().enumerate() {
                    visit(index, child);
                }
            }
            Node::SwitchLayout(switch_layout) => {
                for (index, case_entry) in switch_layout.cases().iter().enumerate() {
                    visit(index, case_entry.child());
                }
                visit(switch_layout.cases().len(), switch_layout.fallback());
            }
            Node::TextBox(_)
            | Node::Spacer(_)
            | Node::Knob(_)
            | Node::Slider(_)
            | Node::Toggle(_)
            | Node::Button(_)
            | Node::Dropdown(_)
            | Node::TabBar(_)
            | Node::CurveEditor(_)
            | Node::Region(_)
            | Node::Indicator(_) => {}
        }
    }
}
