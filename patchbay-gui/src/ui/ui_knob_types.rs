/// Immutable label inputs for knob rendering.
#[derive(Clone, Copy)]
struct KnobLabels<'a> {
    /// Name label displayed above the knob.
    name: &'a str,
    /// Value label displayed below the knob.
    value: &'a str,
}

impl<'a> KnobLabels<'a> {
    /// Build a new knob label pair.
    fn new(name: &'a str, value: &'a str) -> Self {
        Self { name, value }
    }

    /// Return the normalized uppercase name label.
    fn normalized_name(self) -> String {
        normalize_knob_name_label(self.name)
    }

    /// Return the normalized value label.
    fn normalized_value(self) -> String {
        normalize_knob_value_label(self.value)
    }
}

/// Inclusive value range used by knob interactions.
#[derive(Clone, Copy)]
struct KnobRange {
    /// Minimum permitted value.
    min: f32,
    /// Maximum permitted value.
    max: f32,
}

impl KnobRange {
    /// Build a normalized range from tuple input.
    fn from_tuple(range: (f32, f32)) -> Self {
        let (a, b) = range;
        if a <= b {
            Self { min: a, max: b }
        } else {
            Self { min: b, max: a }
        }
    }

    /// Return the span width with a small non-zero floor.
    fn span(self) -> f32 {
        (self.max - self.min).max(1.0e-6)
    }

    /// Clamp `value` into this range.
    fn clamp(self, value: f32) -> f32 {
        value.clamp(self.min, self.max)
    }

    /// Convert `value` into a 0..1 normalized position.
    fn normalized(self, value: f32) -> f32 {
        ((value - self.min) / self.span()).clamp(0.0, 1.0)
    }
}

/// Resolved geometry for a single knob block.
#[derive(Clone, Copy)]
struct KnobGeometry {
    /// Outer block rectangle including labels and dial hit area.
    block_rect: Rect,
    /// Dial rectangle used for drawing the core knob.
    knob_rect: Rect,
    /// Expanded interactive hit rectangle around the dial.
    hit_rect: Rect,
    /// Dial center point in canvas coordinates.
    center: Point,
    /// Dial core radius.
    radius: i32,
    /// Dial side length in pixels.
    knob_size: i32,
    /// Vertical gap between dial and labels.
    label_gap: i32,
}

/// Inputs for a single knob render pass.
#[derive(Clone, Copy)]
struct KnobRenderSpec<'a> {
    /// Stable widget id.
    id: WidgetId,
    /// Label content to render.
    labels: KnobLabels<'a>,
    /// Interaction range limits.
    range: KnobRange,
}

impl<'a> KnobRenderSpec<'a> {
    /// Build a render spec from primitive knob arguments.
    fn from_args(id: WidgetId, name_label: &'a str, value_label: &'a str, range: (f32, f32)) -> Self {
        Self {
            id,
            labels: KnobLabels::new(name_label, value_label),
            range: KnobRange::from_tuple(range),
        }
    }
}

/// Rectangle-scoped rendering controls for knobs.
#[derive(Clone, Copy)]
struct KnobRectSpec {
    /// Bounding rectangle for clipped knob rendering.
    rect: Rect,
    /// Preferred knob diameter before clamping to available space.
    desired_diameter: u32,
}

impl KnobRectSpec {
    /// Build a rect-scoped knob spec.
    fn new(rect: Rect, desired_diameter: u32) -> Self {
        Self {
            rect,
            desired_diameter,
        }
    }
}
