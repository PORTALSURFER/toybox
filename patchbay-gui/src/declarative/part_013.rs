
/// Return node layout constraints.
fn node_layout(node: &Node) -> LayoutBox {
    match node {
        Node::Panel(panel) => panel.layout,
        Node::Row(flex) | Node::Column(flex) => flex.layout,
        Node::Grid(grid) => grid.layout,
        Node::Absolute(absolute) => absolute.layout,
        Node::Label(label) => label.layout,
        Node::Spacer(spacer) => LayoutBox::fixed(spacer.size.width, spacer.size.height),
        Node::Knob(knob) => knob.layout,
        Node::Slider(slider) => slider.layout,
        Node::Toggle(toggle) => toggle.layout,
        Node::Button(button) => button.layout,
        Node::Dropdown(dropdown) => dropdown.layout,
        Node::Region(region) => LayoutBox::fixed(region.size.width, region.size.height),
        Node::Indicator(indicator) => LayoutBox::fixed(indicator.size.width, indicator.size.height),
    }
}

/// Compute header height for titled containers.
fn panel_header_height(title: Option<&str>, tokens: &ThemeTokens) -> i32 {
    if title.is_some() {
        (8 * tokens.typography.text_scale as i32 + tokens.spacing.xs).max(0)
    } else {
        0
    }
}

/// Inset a rectangle by edge insets.
fn inset_rect(rect: Rect, insets: EdgeInsets) -> Rect {
    let left = insets.left.max(0) as u32;
    let right = insets.right.max(0) as u32;
    let top = insets.top.max(0) as u32;
    let bottom = insets.bottom.max(0) as u32;

    Rect {
        origin: Point {
            x: rect.origin.x + left as i32,
            y: rect.origin.y + top as i32,
        },
        size: Size {
            width: rect.size.width.saturating_sub(left + right),
            height: rect.size.height.saturating_sub(top + bottom),
        },
    }
}

/// Offset a point by an origin.
fn offset_point(point: Point, origin: Point) -> Point {
    Point {
        x: point.x + origin.x,
        y: point.y + origin.y,
    }
}

/// Offset a rectangle by an origin.
fn offset_rect(rect: Rect, origin: Point) -> Rect {
    Rect {
        origin: offset_point(rect.origin, origin),
        size: rect.size,
    }
}

/// Format control values for generated labels.
fn format_value(value: f32) -> String {
    let mut text = if value.abs() >= 1.0 {
        format!("{value:.2}")
    } else if value.abs() >= 0.1 {
        format!("{value:.3}")
    } else {
        format!("{value:.4}")
    };
    if let Some(dot) = text.find('.') {
        while text.ends_with('0') && text.len() > dot + 2 {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
    }
    if text == "-0" { "0".to_string() } else { text }
}

/// Measure monospaced bitmap text bounds at a given scale.
fn text_size(text: &str, scale: u32) -> Size {
    let scale = scale.max(1) as i32;
    let mut max_cols = 0i32;
    let mut lines = 1i32;
    let mut current = 0i32;
    for ch in text.chars() {
        if ch == '\n' {
            max_cols = max_cols.max(current);
            current = 0;
            lines += 1;
        } else {
            current += 1;
        }
    }
    max_cols = max_cols.max(current);
    Size {
        width: (max_cols * 6 * scale).max(0) as u32,
        height: (lines * 8 * scale).max(0) as u32,
    }
}

/// Major axis marker for flex layout.
#[derive(Clone, Copy)]
enum Axis {
    /// Horizontal row axis.
    Horizontal,
    /// Vertical column axis.
    Vertical,
}

impl Axis {
    /// Return main-axis length from a size.
    fn main(self, size: Size) -> u32 {
        match self {
            Self::Horizontal => size.width,
            Self::Vertical => size.height,
        }
    }

    /// Return cross-axis length from a size.
    fn cross(self, size: Size) -> u32 {
        match self {
            Self::Horizontal => size.height,
            Self::Vertical => size.width,
        }
    }

    /// Return main-axis origin coordinate.
    fn origin_main(self, origin: Point) -> i32 {
        match self {
            Self::Horizontal => origin.x,
            Self::Vertical => origin.y,
        }
    }

    /// Return cross-axis origin coordinate.
    fn origin_cross(self, origin: Point) -> i32 {
        match self {
            Self::Horizontal => origin.y,
            Self::Vertical => origin.x,
        }
    }

    /// Return main-axis length constraint from a layout box.
    fn main_length(self, layout: LayoutBox) -> Length {
        match self {
            Self::Horizontal => layout.width,
            Self::Vertical => layout.height,
        }
    }

    /// Return cross-axis length constraint from a layout box.
    fn cross_length(self, layout: LayoutBox) -> Length {
        match self {
            Self::Horizontal => layout.height,
            Self::Vertical => layout.width,
        }
    }

    /// Compose a rectangle from axis-oriented values.
    fn compose_rect(self, main: i32, cross: i32, main_size: i32, cross_size: i32) -> Rect {
        match self {
            Self::Horizontal => Rect {
                origin: Point { x: main, y: cross },
                size: Size {
                    width: main_size.max(0) as u32,
                    height: cross_size.max(0) as u32,
                },
            },
            Self::Vertical => Rect {
                origin: Point { x: cross, y: main },
                size: Size {
                    width: cross_size.max(0) as u32,
                    height: main_size.max(0) as u32,
                },
            },
        }
    }
}
