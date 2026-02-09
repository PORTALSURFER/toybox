/// Section sizing mode for canonical row/column section layouts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectionSize {
    /// Fractional section sized as a percentage of parent bounds.
    ///
    /// Percentages are resolved first against the total parent extent.
    Fraction(u8),
    /// Fill section that shares remaining space with other fill siblings.
    Fill,
}

/// Child node paired with a section sizing definition.
#[derive(Clone, Debug)]
pub struct SectionChild {
    /// Child node rendered inside the section.
    pub node: Node,
    /// Section sizing definition.
    pub size: SectionSize,
}

/// Backward-compatible alias for section children.
pub type WeightedChild = SectionChild;

/// Create a fraction-based section child.
pub fn fraction(node: Node, percent: u8) -> SectionChild {
    SectionChild {
        node,
        size: SectionSize::Fraction(percent),
    }
}

/// Create a fill section child.
pub fn fill_section(node: Node) -> SectionChild {
    SectionChild {
        node,
        size: SectionSize::Fill,
    }
}

/// Backward-compatible weighted section helper.
///
/// This maps weights into fractional percentages for strict section sizing.
pub fn weighted(node: Node, weight: u16) -> WeightedChild {
    let percent = weight.clamp(1, 100) as u8;
    fraction(node, percent)
}
