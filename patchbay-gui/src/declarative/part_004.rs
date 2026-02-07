
impl EdgeInsets {
    /// Uniform insets.
    pub const fn all(value: i32) -> Self {
        Self {
            left: value,
            right: value,
            top: value,
            bottom: value,
        }
    }

    /// Symmetric horizontal + vertical insets.
    pub const fn symmetric(horizontal: i32, vertical: i32) -> Self {
        Self {
            left: horizontal,
            right: horizontal,
            top: vertical,
            bottom: vertical,
        }
    }
}

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

/// Flex container specification.
#[derive(Clone, Debug)]
pub struct FlexSpec {
    /// Layout constraints for this container.
    pub layout: LayoutBox,
    /// Gap between children.
    pub gap: i32,
    /// Container padding.
    pub padding: EdgeInsets,
    /// Cross-axis alignment.
    pub align: Align,
    /// Main-axis distribution.
    pub justify: Justify,
    /// Child nodes.
    pub children: Vec<Node>,
}

impl FlexSpec {
    /// Create a row spec.
    pub fn row(children: Vec<Node>) -> Self {
        Self {
            layout: LayoutBox::auto(),
            gap: 12,
            padding: EdgeInsets::default(),
            align: Align::Start,
            justify: Justify::Start,
            children,
        }
    }

    /// Create a column spec.
    pub fn column(children: Vec<Node>) -> Self {
        Self {
            layout: LayoutBox::auto(),
            gap: 12,
            padding: EdgeInsets::default(),
            align: Align::Start,
            justify: Justify::Start,
            children,
        }
    }

    /// Override layout constraints.
    pub fn layout(mut self, layout: LayoutBox) -> Self {
        self.layout = layout;
        self
    }

    /// Override gap.
    pub fn gap(mut self, gap: i32) -> Self {
        self.gap = gap;
        self
    }

    /// Override padding.
    pub fn padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;
        self
    }

    /// Set uniform container padding.
    pub fn pad_all(mut self, value: i32) -> Self {
        self.padding = EdgeInsets::all(value);
        self
    }

    /// Set horizontal and vertical container padding.
    pub fn pad_xy(mut self, horizontal: i32, vertical: i32) -> Self {
        self.padding = EdgeInsets::symmetric(horizontal, vertical);
        self
    }

    /// Override cross-axis alignment.
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Override main-axis distribution.
    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    /// Align children to the cross-axis start.
    pub fn align_start(mut self) -> Self {
        self.align = Align::Start;
        self
    }

    /// Center children on the cross-axis.
    pub fn align_center(mut self) -> Self {
        self.align = Align::Center;
        self
    }

    /// Align children to the cross-axis end.
    pub fn align_end(mut self) -> Self {
        self.align = Align::End;
        self
    }

    /// Stretch children across the cross-axis.
    pub fn align_stretch(mut self) -> Self {
        self.align = Align::Stretch;
        self
    }

    /// Pack children at the main-axis start.
    pub fn justify_start(mut self) -> Self {
        self.justify = Justify::Start;
        self
    }

    /// Center children on the main axis.
    pub fn justify_center(mut self) -> Self {
        self.justify = Justify::Center;
        self
    }

    /// Pack children at the main-axis end.
    pub fn justify_end(mut self) -> Self {
        self.justify = Justify::End;
        self
    }

    /// Distribute available space between items.
    pub fn justify_space_between(mut self) -> Self {
        self.justify = Justify::SpaceBetween;
        self
    }

    /// Distribute available space around items.
    pub fn justify_space_around(mut self) -> Self {
        self.justify = Justify::SpaceAround;
        self
    }

    /// Distribute available space evenly across edges and gaps.
    pub fn justify_space_evenly(mut self) -> Self {
        self.justify = Justify::SpaceEvenly;
        self
    }
}

/// Grid track sizing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackSize {
    /// Fixed track size.
    Px(u32),
    /// Track size from intrinsic content.
    Auto,
    /// Track size as a percentage of parent axis space.
    Percent(u8),
    /// Track that receives equal shares of remaining axis space.
    Fill,
    /// Fractional track fill weight.
    Fr(u16),
}

impl TrackSize {
    /// Return fractional weight.
    fn fr_weight(self) -> u32 {
        match self {
            Self::Fr(weight) => weight.max(1) as u32,
            _ => 0,
        }
    }
}

/// Grid template describing rows/columns.
#[derive(Clone, Debug)]
pub struct GridTemplate {
    /// Column tracks.
    pub columns: Vec<TrackSize>,
    /// Optional row tracks. Missing rows default to `Auto`.
    pub rows: Vec<TrackSize>,
    /// Gap between columns in pixels.
    pub column_gap: i32,
    /// Gap between rows in pixels.
    pub row_gap: i32,
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
    /// Canonical vertical section container.
    SectionColumn,
    /// Canonical horizontal section container.
    SectionRow,
}
