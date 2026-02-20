/// Grid template describing rows/columns.
#[derive(Clone, Debug)]
pub struct GridTemplate {
    /// Column tracks.
    pub columns: Vec<TrackSize>,
    /// Optional row tracks. Missing rows default to `Auto`.
    pub rows: Vec<TrackSize>,
    /// Horizontal distribution for leftover width.
    pub justify_x: Justify,
    /// Grid padding.
    pub padding: EdgeInsets,
}

/// Grid semantic role for strict declarative validation and sizing rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridKind {
    /// General-purpose grid with legacy track behavior.
    Standard,
    /// Canonical vertical slot container.
    SlotColumn,
    /// Canonical horizontal slot container.
    SlotRow,
}
