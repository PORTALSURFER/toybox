/// Typed interaction actions emitted by declarative rendering.
#[derive(Clone, Debug, PartialEq)]
pub enum UiAction {
    /// Knob value update.
    KnobChanged {
        /// Stable widget key.
        key: String,
        /// New widget value.
        value: f32,
    },
    /// Slider value update.
    SliderChanged {
        /// Stable widget key.
        key: String,
        /// New widget value.
        value: f32,
    },
    /// Toggle value update.
    ToggleChanged {
        /// Stable widget key.
        key: String,
        /// New widget value.
        value: bool,
    },
    /// Button click event.
    ButtonPressed {
        /// Stable widget key.
        key: String,
    },
    /// Dropdown selection event.
    DropdownSelected {
        /// Stable widget key.
        key: String,
        /// Selected option index.
        index: usize,
    },
    /// Region hover state update.
    RegionHover {
        /// Stable widget key.
        key: String,
        /// True when the pointer is currently inside the region.
        hovered: bool,
        /// Pointer position relative to the region bounds.
        local_pointer: Point,
    },
    /// Region interaction event.
    RegionInteracted {
        /// Stable widget key.
        key: String,
        /// Interaction kind.
        kind: RegionInteractionKind,
        /// Pointer position relative to the interacted region.
        local_pointer: Point,
        /// Unclamped pointer position relative to the region origin.
        raw_local_pointer: Point,
        /// Whether Alt was held during this interaction frame.
        alt_down: bool,
    },
}

/// Declarative drawing command for region rendering.
#[derive(Clone, Debug, PartialEq)]
pub enum DrawCommand {
    /// Fill a rectangle at a region-relative position.
    FillRect {
        /// Region-relative rectangle.
        rect: Rect,
        /// Fill color.
        color: Color,
    },
    /// Stroke a rectangle at a region-relative position.
    StrokeRect {
        /// Region-relative rectangle.
        rect: Rect,
        /// Stroke thickness in pixels.
        thickness: u32,
        /// Stroke color.
        color: Color,
    },
    /// Fill a circle at a region-relative center.
    FillCircle {
        /// Region-relative center point.
        center: Point,
        /// Circle radius in pixels.
        radius: i32,
        /// Fill color.
        color: Color,
    },
    /// Stroke a circle at a region-relative center.
    StrokeCircle {
        /// Region-relative center point.
        center: Point,
        /// Circle radius in pixels.
        radius: i32,
        /// Stroke thickness in pixels.
        thickness: i32,
        /// Stroke color.
        color: Color,
    },
    /// Draw a line between two region-relative points.
    Line {
        /// Region-relative start point.
        start: Point,
        /// Region-relative end point.
        end: Point,
        /// Line color.
        color: Color,
    },
    /// Draw text at a region-relative origin.
    Text {
        /// Region-relative text origin.
        origin: Point,
        /// Text content.
        text: String,
        /// Text color.
        color: Color,
        /// Bitmap text scale.
        scale: u32,
    },
}

/// Specific region interaction type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionInteractionKind {
    /// Primary press began.
    Pressed,
    /// Primary press ended.
    Released,
    /// Drag in progress.
    Dragged,
    /// Secondary click occurred.
    SecondaryClicked,
    /// Double click occurred.
    DoubleClicked,
}
