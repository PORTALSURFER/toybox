/// Cross-axis alignment in flex layouts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    /// Start alignment.
    #[default]
    Start,
    /// Center alignment.
    Center,
    /// End alignment.
    End,
    /// Stretch across available cross-axis space.
    Stretch,
}

/// Main-axis distribution in flex layouts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Justify {
    /// Pack items at the start.
    #[default]
    Start,
    /// Center items in available main-axis space.
    Center,
    /// Pack items at the end.
    End,
    /// Distribute remaining space between items.
    SpaceBetween,
    /// Distribute remaining space around items.
    SpaceAround,
    /// Distribute remaining space evenly, including edges.
    SpaceEvenly,
}
