/// Input request for resolving one grid axis.
struct GridAxisResolveRequest<'a> {
    /// Track-size definitions for the resolved axis.
    tracks: &'a [TrackSize],
    /// Total number of grid columns.
    columns: usize,
    /// Total number of grid rows.
    rows: usize,
    /// Gap value between tracks on this axis.
    gap: i32,
    /// Available axis size before gap subtraction.
    available: u32,
    /// Whether the resolved axis is columns (`true`) or rows (`false`).
    is_columns: bool,
    /// Intrinsic child measurements used for `Auto` tracks.
    intrinsic: &'a [Size],
}

/// Axis selection for grid resolution.
#[derive(Copy, Clone, Debug)]
enum GridAxis {
    /// Resolve track sizes along the horizontal column axis.
    Columns,
    /// Resolve track sizes along the vertical row axis.
    Rows,
}

impl GridAxis {
    /// Return the axis index for a flattened grid child index.
    fn index_for_item(self, item: usize, columns: usize) -> usize {
        match self {
            Self::Columns => item % columns,
            Self::Rows => item / columns,
        }
    }

    /// Read the intrinsic size component matching this axis.
    fn intrinsic_size(self, measured: Size) -> u32 {
        match self {
            Self::Columns => measured.width,
            Self::Rows => measured.height,
        }
    }
}

/// Parameters required to resolve one grid axis.
struct GridAxisPlan<'a> {
    /// Track-size definitions for the resolved axis.
    tracks: &'a [TrackSize],
    /// Number of tracks that must be produced on this axis.
    axis_count: usize,
    /// Grid column count used to map flattened child indices.
    columns: usize,
    /// Gap size between adjacent tracks on this axis.
    gap: i32,
    /// Total axis space available before gap subtraction.
    available: u32,
    /// Axis mode that controls index and intrinsic component lookup.
    axis: GridAxis,
    /// Intrinsic child measurements used for `Auto` track sizing.
    intrinsic: &'a [Size],
}
