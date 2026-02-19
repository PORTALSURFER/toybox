
/// Return node layout constraints.
fn node_layout(node: &Node) -> LayoutBox {
    match node {
        Node::Slot(slot) => node_layout(&slot.child),
        Node::Panel(panel) => panel.layout.to_layout_box(),
        Node::Row(flex) | Node::Column(flex) => flex.layout.to_layout_box(),
        Node::Grid(grid) => grid.layout.to_layout_box(),
        Node::Absolute(absolute) => absolute.layout.to_layout_box(),
        Node::Stack(stack) => stack.layout.to_layout_box(),
        Node::ScrollView(scroll_view) => scroll_view.layout.to_layout_box(),
        Node::Wrap(wrap) => wrap.layout.to_layout_box(),
        Node::SwitchLayout(switch_layout) => switch_layout.layout.to_layout_box(),
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

/// Return the overlap between two rectangles, if any.
fn rect_intersection(first: Rect, second: Rect) -> Option<Rect> {
    let min_x = first.origin.x.max(second.origin.x);
    let min_y = first.origin.y.max(second.origin.y);

    let first_width = i32::try_from(first.size.width).unwrap_or(i32::MAX);
    let second_width = i32::try_from(second.size.width).unwrap_or(i32::MAX);
    let first_height = i32::try_from(first.size.height).unwrap_or(i32::MAX);
    let second_height = i32::try_from(second.size.height).unwrap_or(i32::MAX);

    let first_right = first.origin.x.saturating_add(first_width);
    let second_right = second.origin.x.saturating_add(second_width);
    let max_x = first_right.min(second_right);

    let first_bottom = first.origin.y.saturating_add(first_height);
    let second_bottom = second.origin.y.saturating_add(second_height);
    let max_y = first_bottom.min(second_bottom);

    let width = (max_x - min_x).max(0);
    let height = (max_y - min_y).max(0);
    if width == 0 || height == 0 {
        return None;
    }

    Some(Rect {
        origin: Point {
            x: min_x,
            y: min_y,
        },
        size: Size {
            width: u32::try_from(width).expect("intersection width must be non-negative"),
            height: u32::try_from(height).expect("intersection height must be non-negative"),
        },
    })
}

/// Clamp a rectangle to container bounds and optionally emit overflow warnings.
fn clip_rect_to_bounds(rect: Rect, bounds: Rect) -> Option<Rect> {
    let clipped = rect_intersection(rect, bounds);
    match clipped {
        Some(clipped_rect) if clipped_rect != rect => {
            emit_layout_overflow_warning(
                rect,
                clipped_rect,
                "layout rect is partially outside parent bounds",
            );
            Some(clipped_rect)
        }
        Some(clipped_rect) => {
            debug_assert!(clipped_rect.origin.x >= bounds.origin.x);
            debug_assert!(clipped_rect.origin.y >= bounds.origin.y);
            debug_assert!(
                clipped_rect.origin.x.saturating_add(clipped_rect.size.width as i32)
                    <= bounds.origin.x.saturating_add(bounds.size.width as i32)
            );
            debug_assert!(
                clipped_rect.origin.y.saturating_add(clipped_rect.size.height as i32)
                    <= bounds.origin.y.saturating_add(bounds.size.height as i32)
            );
            Some(clipped_rect)
        }
        None => {
            emit_layout_overflow_warning(
                rect,
                bounds,
                "layout rect does not intersect parent bounds",
            );
            None
        }
    }
}

/// Emit a debug warning when a layout rectangle extends beyond container bounds.
fn emit_layout_overflow_warning(_rect: Rect, _bounds: Rect, _message: &str) {
    #[cfg(feature = "layout-overflow-warnings")]
    eprintln!(
        "patchbay-gui layout overflow: {_message}: rect ({}, {}) + ({}, {}) outside ({}, {}) + ({}, {})",
        _rect.origin.x,
        _rect.origin.y,
        _rect.size.width,
        _rect.size.height,
        _bounds.origin.x,
        _bounds.origin.y,
        _bounds.size.width,
        _bounds.size.height,
    );
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
    let scale = scale.max(1) as u64;
    let mut max_cols = 0u64;
    let mut lines = 1u64;
    let mut current = 0u64;
    for ch in text.chars() {
        if ch == '\n' {
            max_cols = max_cols.max(current);
            current = 0;
            lines = lines.saturating_add(1);
        } else {
            current = current.saturating_add(1);
        }
    }
    max_cols = max_cols.max(current);

    let width = max_cols
        .saturating_mul(6)
        .saturating_mul(scale)
        .min(u64::from(u32::MAX));
    let height = lines
        .saturating_mul(8)
        .saturating_mul(scale)
        .min(u64::from(u32::MAX));

    Size {
        width: width as u32,
        height: height as u32,
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
