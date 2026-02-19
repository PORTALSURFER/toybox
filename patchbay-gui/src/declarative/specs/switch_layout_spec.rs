/// Width-rule for selecting one responsive layout subtree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SwitchWidthRange {
    /// Optional inclusive lower width bound.
    pub min_inclusive: Option<u32>,
    /// Optional exclusive upper width bound.
    pub max_exclusive: Option<u32>,
}

impl SwitchWidthRange {
    /// Create a width range with optional inclusive min and exclusive max.
    pub const fn new(min_inclusive: Option<u32>, max_exclusive: Option<u32>) -> Self {
        Self {
            min_inclusive,
            max_exclusive,
        }
    }

    /// Match widths strictly below `max_exclusive`.
    pub const fn lt(max_exclusive: u32) -> Self {
        Self::new(None, Some(max_exclusive))
    }

    /// Match widths in `[min_inclusive, max_exclusive)`.
    pub const fn between(min_inclusive: u32, max_exclusive: u32) -> Self {
        Self::new(Some(min_inclusive), Some(max_exclusive))
    }

    /// Match widths at or above `min_inclusive`.
    pub const fn ge(min_inclusive: u32) -> Self {
        Self::new(Some(min_inclusive), None)
    }

    /// Return `true` when `width` matches this range.
    pub const fn contains(self, width: u32) -> bool {
        let min_ok = match self.min_inclusive {
            Some(min) => width >= min,
            None => true,
        };
        let max_ok = match self.max_exclusive {
            Some(max) => width < max,
            None => true,
        };
        min_ok && max_ok
    }
}

/// One ordered `SwitchLayout` case containing a width rule and slotted child.
#[derive(Clone, Debug)]
pub struct SwitchCase {
    /// Width rule evaluated against root content width.
    pub(crate) range: SwitchWidthRange,
    /// Slotted child subtree selected when `range` matches.
    pub(crate) child: Node,
}

impl SwitchCase {
    /// Create one responsive case.
    pub fn new(range: SwitchWidthRange, child: Node) -> Self {
        Self {
            range,
            child: Node::slot(child),
        }
    }

    /// Return this case width range.
    pub fn range(&self) -> SwitchWidthRange {
        self.range
    }

    /// Borrow this case slotted child.
    pub fn child(&self) -> &Node {
        &self.child
    }
}

/// Create a switch case for `width < max_exclusive`.
pub fn when_width_lt(max_exclusive: u32, child: Node) -> SwitchCase {
    SwitchCase::new(SwitchWidthRange::lt(max_exclusive), child)
}

/// Create a switch case for `min_inclusive <= width < max_exclusive`.
pub fn when_width_between(min_inclusive: u32, max_exclusive: u32, child: Node) -> SwitchCase {
    SwitchCase::new(
        SwitchWidthRange::between(min_inclusive, max_exclusive),
        child,
    )
}

/// Create a switch case for `width >= min_inclusive`.
pub fn when_width_ge(min_inclusive: u32, child: Node) -> SwitchCase {
    SwitchCase::new(SwitchWidthRange::ge(min_inclusive), child)
}

/// Responsive container that selects one slotted child by root content width.
///
/// Selection is deterministic:
/// - cases are evaluated in declared order
/// - first matching case wins
/// - fallback is used when no case matches
#[derive(Clone, Debug)]
pub struct SwitchLayoutSpec {
    /// Layout constraints for this container.
    pub(crate) layout: ContainerLayout,
    /// Ordered width cases.
    pub(crate) cases: Vec<SwitchCase>,
    /// Fallback slotted child when no case matches.
    pub(crate) fallback: Box<Node>,
}

impl SwitchLayoutSpec {
    /// Create a switch container from ordered cases and a required fallback.
    pub fn new(cases: Vec<SwitchCase>, fallback: Node) -> Self {
        Self {
            layout: ContainerLayout::auto(),
            cases,
            fallback: Box::new(Node::slot(fallback)),
        }
    }

    /// Override layout constraints.
    pub fn layout(mut self, layout: ContainerLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Set overflow behavior for selected content.
    pub fn overflow(mut self, overflow_policy: OverflowPolicy) -> Self {
        self.layout = self.layout.overflow(overflow_policy);
        self
    }

    /// Borrow ordered width cases.
    pub fn cases(&self) -> &[SwitchCase] {
        &self.cases
    }

    /// Borrow fallback slotted child.
    pub fn fallback(&self) -> &Node {
        self.fallback.as_ref()
    }

    /// Select the child for the current `root_content_width`.
    pub fn selected_child(&self, root_content_width: u32) -> &Node {
        for case in &self.cases {
            if case.range.contains(root_content_width) {
                return &case.child;
            }
        }
        self.fallback.as_ref()
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
