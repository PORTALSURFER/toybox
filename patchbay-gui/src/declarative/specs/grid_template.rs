impl GridTemplate {
    /// Build a grid template from column tracks.
    pub fn new(columns: Vec<TrackSize>) -> Self {
        Self {
            columns,
            rows: Vec::new(),
            justify_x: Justify::Start,
            padding: EdgeInsets::default(),
        }
    }

    /// Build a template with `count` equal fractional columns.
    pub fn columns_fr(count: usize) -> Self {
        let count = count.max(1);
        Self::new(vec![TrackSize::Fr(1); count])
    }

    /// Override row tracks.
    pub fn rows(mut self, rows: Vec<TrackSize>) -> Self {
        self.rows = rows;
        self
    }

    /// Override rows with equal fractional tracks.
    pub fn rows_fr(mut self, count: usize) -> Self {
        let count = count.max(1);
        self.rows = vec![TrackSize::Fr(1); count];
        self
    }

    /// Set uniform grid padding.
    pub fn pad_all(mut self, value: i32) -> Self {
        self.padding = EdgeInsets::all(value);
        self
    }

    /// Set horizontal and vertical grid padding.
    pub fn pad_xy(mut self, horizontal: i32, vertical: i32) -> Self {
        self.padding = EdgeInsets::symmetric(horizontal, vertical);
        self
    }

    /// Pack columns from the left edge.
    pub fn justify_start(mut self) -> Self {
        self.justify_x = Justify::Start;
        self
    }

    /// Center packed columns in available width.
    pub fn justify_center(mut self) -> Self {
        self.justify_x = Justify::Center;
        self
    }

    /// Pack columns against the right edge.
    pub fn justify_end(mut self) -> Self {
        self.justify_x = Justify::End;
        self
    }

    /// Distribute leftover width between columns.
    pub fn justify_space_between(mut self) -> Self {
        self.justify_x = Justify::SpaceBetween;
        self
    }

    /// Distribute leftover width around columns.
    pub fn justify_space_around(mut self) -> Self {
        self.justify_x = Justify::SpaceAround;
        self
    }

    /// Distribute leftover width evenly including edges.
    pub fn justify_space_evenly(mut self) -> Self {
        self.justify_x = Justify::SpaceEvenly;
        self
    }

    /// Override padding.
    pub fn padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;
        self
    }
}
