/// Alignment options for section-child placement on each axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SectionAlign {
    /// Place content at axis start.
    #[default]
    Start,
    /// Center content on the axis.
    Center,
    /// Place content at axis end.
    End,
    /// Stretch content to the full section extent on this axis.
    Stretch,
}

/// Section sizing mode for canonical row/column section layouts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectionTrack {
    /// Fractional section sized as a percentage of parent bounds.
    ///
    /// Percentages are resolved first against the total parent extent.
    Fraction(u8),
    /// Fill section that shares remaining space with other fill siblings.
    Fill,
    /// Fixed pixel section.
    Px(u32),
}

/// Backward-compatible alias.
pub type SectionSize = SectionTrack;

/// Child node paired with a section sizing definition.
#[derive(Clone, Debug)]
pub struct SectionChild {
    /// Child node rendered inside the section.
    pub node: Node,
    /// Section sizing definition.
    pub size: SectionTrack,
    /// Horizontal alignment inside the section.
    pub align_x: SectionAlign,
    /// Vertical alignment inside the section.
    pub align_y: SectionAlign,
}

/// Backward-compatible alias for section children.
pub type WeightedChild = SectionChild;

/// Create a fraction-based section child.
pub fn fraction(node: Node, percent: u8) -> SectionChild {
    SectionChild {
        node,
        size: SectionTrack::Fraction(percent),
        align_x: SectionAlign::Start,
        align_y: SectionAlign::Start,
    }
}

/// Create a fill section child.
pub fn fill_section(node: Node) -> SectionChild {
    SectionChild {
        node,
        size: SectionTrack::Fill,
        align_x: SectionAlign::Start,
        align_y: SectionAlign::Start,
    }
}

/// Create a fixed-pixel section child.
pub fn px_section(node: Node, px: u32) -> SectionChild {
    SectionChild {
        node,
        size: SectionTrack::Px(px),
        align_x: SectionAlign::Start,
        align_y: SectionAlign::Start,
    }
}

/// Backward-compatible weighted section helper.
///
/// This maps weights into fractional percentages for strict section sizing.
pub fn weighted(node: Node, weight: u16) -> WeightedChild {
    let percent = weight.clamp(1, 100) as u8;
    fraction(node, percent)
}

impl SectionChild {
    /// Set both axis alignments for this section child.
    pub fn align(mut self, align_x: SectionAlign, align_y: SectionAlign) -> Self {
        self.align_x = align_x;
        self.align_y = align_y;
        self
    }

    /// Set horizontal alignment for this section child.
    pub fn align_x(mut self, align_x: SectionAlign) -> Self {
        self.align_x = align_x;
        self
    }

    /// Set vertical alignment for this section child.
    pub fn align_y(mut self, align_y: SectionAlign) -> Self {
        self.align_y = align_y;
        self
    }
}
