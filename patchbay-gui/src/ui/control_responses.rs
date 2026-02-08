
/// Response metadata from knob widgets.
#[derive(Clone, Copy, Debug, Default)]
pub struct KnobResponse {
    /// The knob value changed this frame.
    pub changed: bool,
    /// The pointer is hovering the knob.
    pub hovered: bool,
    /// The knob is actively being dragged.
    pub active: bool,
}

/// Response metadata from slider widgets.
#[derive(Clone, Copy, Debug, Default)]
pub struct SliderResponse {
    /// The slider value changed this frame.
    pub changed: bool,
    /// The pointer is hovering the slider.
    pub hovered: bool,
    /// The slider is actively being dragged.
    pub active: bool,
}

/// Response metadata from toggle widgets.
#[derive(Clone, Copy, Debug, Default)]
pub struct ToggleResponse {
    /// The toggle value changed this frame.
    pub changed: bool,
    /// The pointer is hovering the toggle.
    pub hovered: bool,
}

/// Response metadata from button widgets.
#[derive(Clone, Copy, Debug, Default)]
pub struct ButtonResponse {
    /// The button was clicked this frame.
    pub clicked: bool,
    /// The pointer is hovering the button.
    pub hovered: bool,
}

/// Response metadata from custom region widgets.
#[derive(Clone, Copy, Debug, Default)]
pub struct RegionResponse {
    /// The pointer is hovering the region.
    pub hovered: bool,
    /// Pointer position relative to region bounds.
    pub local_pointer: Point,
    /// Pointer position relative to region origin without bounds clamping.
    pub raw_local_pointer: Point,
    /// Whether Alt was held during this frame.
    pub alt_down: bool,
    /// The region is actively being dragged.
    pub active: bool,
    /// The primary button was pressed on the region.
    pub pressed: bool,
    /// The primary button was released on the region.
    pub released: bool,
    /// The pointer is being dragged while active.
    pub dragged: bool,
    /// The secondary button was clicked on the region.
    pub secondary_clicked: bool,
    /// The primary button was double-clicked on the region.
    pub double_clicked: bool,
}

/// Response metadata from dropdown widgets.
#[derive(Clone, Copy, Debug, Default)]
pub struct DropdownResponse {
    /// The selection changed this frame.
    pub changed: bool,
    /// The dropdown is open this frame.
    pub open: bool,
    /// The pointer is hovering the dropdown control.
    pub hovered: bool,
}
