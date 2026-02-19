/// Alignment options for slot placement on each axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SlotAlign {
    /// Place content at axis start.
    #[default]
    Start,
    /// Center content on the axis.
    Center,
    /// Place content at axis end.
    End,
    /// Stretch content to the full slot extent on this axis.
    Stretch,
}

/// Slot sizing mode for canonical row/column slot layouts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotTrack {
    /// Fractional slot sized as a percentage of parent bounds.
    Fraction(u8),
    /// Fill slot that shares remaining space with other fill siblings.
    Fill,
}

/// Child node paired with a slot sizing definition.
#[derive(Clone, Debug)]
pub struct Slot {
    /// Child node rendered inside the slot.
    pub node: Node,
    /// Slot sizing definition.
    pub size: SlotTrack,
    /// Horizontal alignment inside the slot.
    pub align_x: SlotAlign,
    /// Vertical alignment inside the slot.
    pub align_y: SlotAlign,
}

/// Backward-compatible weighted-slot alias.
pub type WeightedSlot = Slot;

/// Create a fractional slot child.
pub fn fraction_slot(node: Node, percent: u8) -> Slot {
    Slot {
        node,
        size: SlotTrack::Fraction(percent),
        align_x: SlotAlign::Start,
        align_y: SlotAlign::Start,
    }
}

/// Create a fill slot child.
pub fn fill_slot(node: Node) -> Slot {
    Slot {
        node,
        size: SlotTrack::Fill,
        align_x: SlotAlign::Start,
        align_y: SlotAlign::Start,
    }
}

/// Create a weighted slot child.
///
/// Weights are normalized to percentages and clamped to at least `1`.
pub fn weighted_slot(node: Node, weight: u16) -> WeightedSlot {
    let percent = weight.clamp(1, 100) as u8;
    fraction_slot(node, percent)
}

impl Slot {
    /// Set both axis alignments for this slot.
    pub fn align(mut self, align_x: SlotAlign, align_y: SlotAlign) -> Self {
        self.align_x = align_x;
        self.align_y = align_y;
        self
    }

    /// Set horizontal alignment for this slot.
    pub fn align_x(mut self, align_x: SlotAlign) -> Self {
        self.align_x = align_x;
        self
    }

    /// Set vertical alignment for this slot.
    pub fn align_y(mut self, align_y: SlotAlign) -> Self {
        self.align_y = align_y;
        self
    }
}
