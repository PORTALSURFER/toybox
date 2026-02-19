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

/// Main-axis sizing mode for strict slot placement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotMainSize {
    /// Fixed main-axis slot size in pixels.
    Fixed(u32),
    /// Share remaining space with relative fill weight.
    Fill(u16),
    /// Percent of parent main-axis extent.
    Percent(u8),
    /// Content/intrinsic sizing behavior.
    Intrinsic,
}

/// Cross-axis sizing mode for strict slot placement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotCrossSize {
    /// Fixed cross-axis slot size in pixels.
    Fixed(u32),
    /// Fill available cross-axis extent.
    Fill,
    /// Content/intrinsic cross-axis behavior.
    Intrinsic,
}

/// Parent-owned slot metadata describing child placement constraints.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotParams {
    /// Main-axis slot sizing behavior.
    pub size_main: SlotMainSize,
    /// Cross-axis slot sizing behavior.
    pub size_cross: SlotCrossSize,
    /// Optional minimum width constraint for widget children.
    pub min_width: Option<u32>,
    /// Optional maximum width constraint for widget children.
    pub max_width: Option<u32>,
    /// Optional minimum height constraint for widget children.
    pub min_height: Option<u32>,
    /// Optional maximum height constraint for widget children.
    pub max_height: Option<u32>,
    /// Margin applied around the child inside its slot bounds.
    pub margin: EdgeInsets,
    /// Optional horizontal alignment override.
    pub align_x_override: Option<SlotAlign>,
    /// Optional vertical alignment override.
    pub align_y_override: Option<SlotAlign>,
}

impl SlotParams {
    /// Percent-based main-axis split with default fill cross-axis behavior.
    pub const fn percent(percent: u8) -> Self {
        Self {
            size_main: SlotMainSize::Percent(percent),
            size_cross: SlotCrossSize::Fill,
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            margin: EdgeInsets::all(0),
            align_x_override: None,
            align_y_override: None,
        }
    }

    /// Fill-based main-axis split with default fill cross-axis behavior.
    pub const fn fill(weight: u16) -> Self {
        let clamped = if weight == 0 { 1 } else { weight };
        Self {
            size_main: SlotMainSize::Fill(clamped),
            ..Self::percent(0)
        }
    }

    /// Intrinsic main-axis behavior with default fill cross-axis behavior.
    pub const fn intrinsic() -> Self {
        Self {
            size_main: SlotMainSize::Intrinsic,
            ..Self::percent(0)
        }
    }

    /// Fixed main-axis behavior with default fill cross-axis behavior.
    pub const fn fixed(main_size: u32) -> Self {
        Self {
            size_main: SlotMainSize::Fixed(main_size),
            ..Self::percent(0)
        }
    }

    /// Set cross-axis behavior.
    pub const fn cross_size(mut self, size_cross: SlotCrossSize) -> Self {
        self.size_cross = size_cross;
        self
    }

    /// Set slot margin.
    pub const fn margin(mut self, margin: EdgeInsets) -> Self {
        self.margin = margin;
        self
    }

    /// Set optional slot width constraints.
    pub const fn width_bounds(mut self, min: Option<u32>, max: Option<u32>) -> Self {
        self.min_width = min;
        self.max_width = max;
        self
    }

    /// Set optional slot height constraints.
    pub const fn height_bounds(mut self, min: Option<u32>, max: Option<u32>) -> Self {
        self.min_height = min;
        self.max_height = max;
        self
    }

    /// Set horizontal/vertical alignment overrides.
    pub const fn align(mut self, align_x: SlotAlign, align_y: SlotAlign) -> Self {
        self.align_x_override = Some(align_x);
        self.align_y_override = Some(align_y);
        self
    }
}

/// Child node paired with a slot sizing definition.
#[derive(Clone, Debug)]
pub struct Slot {
    /// Child node rendered inside the slot.
    pub node: Node,
    /// Parent-owned slot placement parameters.
    pub params: SlotParams,
}

/// Backward-compatible weighted-slot alias.
pub type WeightedSlot = Slot;

/// Create a fractional slot child.
pub fn fraction_slot(node: Node, percent: u8) -> Slot {
    Slot {
        node,
        params: SlotParams::percent(percent),
    }
}

/// Create a fill slot child.
pub fn fill_slot(node: Node) -> Slot {
    Slot {
        node,
        params: SlotParams::fill(1),
    }
}

/// Create a weighted slot child.
///
/// Weights are clamped to at least `1` and used as fill weights.
pub fn weighted_slot(node: Node, weight: u16) -> WeightedSlot {
    Slot {
        node,
        params: SlotParams::fill(weight),
    }
}

impl Slot {
    /// Create a slot child with explicit placement params.
    pub const fn with_params(node: Node, params: SlotParams) -> Self {
        Self {
            node,
            params,
        }
    }

    /// Set both axis alignments for this slot.
    pub fn align(mut self, align_x: SlotAlign, align_y: SlotAlign) -> Self {
        self.params.align_x_override = Some(align_x);
        self.params.align_y_override = Some(align_y);
        self
    }

    /// Set horizontal alignment for this slot.
    pub fn align_x(mut self, align_x: SlotAlign) -> Self {
        self.params.align_x_override = Some(align_x);
        self
    }

    /// Set vertical alignment for this slot.
    pub fn align_y(mut self, align_y: SlotAlign) -> Self {
        self.params.align_y_override = Some(align_y);
        self
    }

    /// Set slot margin.
    pub fn margin(mut self, margin: EdgeInsets) -> Self {
        self.params.margin = margin;
        self
    }

    /// Set slot cross-axis behavior.
    pub fn cross_size(mut self, size_cross: SlotCrossSize) -> Self {
        self.params.size_cross = size_cross;
        self
    }

    /// Set optional width constraints for widget children.
    pub fn width_bounds(mut self, min: Option<u32>, max: Option<u32>) -> Self {
        self.params.min_width = min;
        self.params.max_width = max;
        self
    }

    /// Set optional height constraints for widget children.
    pub fn height_bounds(mut self, min: Option<u32>, max: Option<u32>) -> Self {
        self.params.min_height = min;
        self.params.max_height = max;
        self
    }
}
