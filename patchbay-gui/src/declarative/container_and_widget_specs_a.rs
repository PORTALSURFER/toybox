
impl GridTemplate {
    /// Build a grid template from column tracks.
    pub fn new(columns: Vec<TrackSize>) -> Self {
        Self {
            columns,
            rows: Vec::new(),
            column_gap: 0,
            row_gap: 0,
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

    /// Override track gap.
    pub fn gap(mut self, gap: i32) -> Self {
        self.column_gap = gap;
        self.row_gap = gap;
        self
    }

    /// Override column and row gaps.
    pub fn gap_xy(mut self, column_gap: i32, row_gap: i32) -> Self {
        self.column_gap = column_gap;
        self.row_gap = row_gap;
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

/// Grid container specification.
#[derive(Clone, Debug)]
pub struct GridSpec {
    /// Layout constraints for this container.
    pub layout: LayoutBox,
    /// Grid track template.
    pub template: GridTemplate,
    /// Child nodes in row-major order.
    pub children: Vec<Node>,
    /// Grid semantic role.
    pub kind: GridKind,
}

impl GridSpec {
    /// Create a grid specification.
    pub fn new(template: GridTemplate, children: Vec<Node>) -> Self {
        Self {
            layout: LayoutBox::auto(),
            template,
            children,
            kind: GridKind::Standard,
        }
    }

    /// Override layout constraints.
    pub fn layout(mut self, layout: LayoutBox) -> Self {
        self.layout = layout;
        self
    }
}

/// Panel container specification.
#[derive(Clone, Debug)]
pub struct PanelSpec {
    /// Stable panel key.
    pub key: String,
    /// Optional title.
    pub title: Option<String>,
    /// Inner padding.
    pub padding: i32,
    /// Optional background color override.
    pub background: Option<Color>,
    /// Optional outline color override.
    pub outline: Option<Color>,
    /// Optional header height override.
    pub header_height: Option<i32>,
    /// Layout constraints.
    pub layout: LayoutBox,
    /// Panel content.
    pub content: Box<Node>,
}

impl PanelSpec {
    /// Create a panel with key and content.
    pub fn new(key: impl Into<String>, content: Node) -> Self {
        Self {
            key: key.into(),
            title: None,
            padding: 12,
            background: None,
            outline: None,
            header_height: None,
            layout: LayoutBox::auto(),
            content: Box::new(content),
        }
    }

    /// Set panel title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Override panel padding.
    pub fn padding(mut self, padding: i32) -> Self {
        self.padding = padding;
        self
    }

    /// Override panel background color.
    pub fn background(mut self, background: Color) -> Self {
        self.background = Some(background);
        self
    }

    /// Override panel outline color.
    pub fn outline(mut self, outline: Color) -> Self {
        self.outline = Some(outline);
        self
    }

    /// Override panel header height.
    pub fn header_height(mut self, header_height: i32) -> Self {
        self.header_height = Some(header_height);
        self
    }

    /// Override panel layout constraints.
    pub fn layout(mut self, layout: LayoutBox) -> Self {
        self.layout = layout;
        self
    }
}

/// Absolute-positioned container specification.
#[derive(Clone, Debug)]
pub struct AbsoluteSpec {
    /// Layout constraints.
    pub layout: LayoutBox,
    /// Positioned children.
    pub children: Vec<AbsoluteChild>,
}

impl AbsoluteSpec {
    /// Create an absolute container.
    pub fn new(children: Vec<AbsoluteChild>) -> Self {
        Self {
            layout: LayoutBox::auto(),
            children,
        }
    }

    /// Override layout constraints.
    pub fn layout(mut self, layout: LayoutBox) -> Self {
        self.layout = layout;
        self
    }
}

/// Positioned child node.
#[derive(Clone, Debug)]
pub struct AbsoluteChild {
    /// Child origin relative to the container.
    pub origin: Point,
    /// Child node.
    pub node: Node,
}

impl AbsoluteChild {
    /// Create a positioned child.
    pub fn new(origin: Point, node: Node) -> Self {
        Self { origin, node }
    }
}

/// Label specification.
#[derive(Clone, Debug)]
pub struct LabelSpec {
    /// Label text.
    pub text: String,
    /// Optional text color override.
    pub color: Option<Color>,
    /// Layout constraints.
    pub layout: LayoutBox,
}

impl LabelSpec {
    /// Create a text label.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: None,
            layout: LayoutBox::auto(),
        }
    }

    /// Override layout constraints.
    pub fn layout(mut self, layout: LayoutBox) -> Self {
        self.layout = layout;
        self
    }
}

/// Spacer specification.
#[derive(Clone, Debug)]
pub struct SpacerSpec {
    /// Spacer size.
    pub size: Size,
}

impl SpacerSpec {
    /// Create a fixed spacer.
    pub const fn new(size: Size) -> Self {
        Self { size }
    }
}

/// Knob widget specification.
#[derive(Clone, Debug)]
pub struct KnobSpec {
    /// Stable widget key.
    pub key: String,
    /// Label displayed above the knob.
    pub label: String,
    /// Optional value label.
    pub value_label: Option<String>,
    /// Current value.
    pub value: f32,
    /// Value range.
    pub range: (f32, f32),
    /// Layout constraints.
    pub layout: LayoutBox,
}
