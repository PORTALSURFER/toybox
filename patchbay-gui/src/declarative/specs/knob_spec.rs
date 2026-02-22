/// Knob widget specification.
#[derive(Clone, Debug)]
pub struct KnobSpec {
    /// Stable widget key.
    pub key: String,
    /// Current value.
    pub value: f32,
    /// Value range.
    pub range: (f32, f32),
    /// Value restored by core double-click reset interactions.
    pub default_value: f32,
    /// Optional explicit control size.
    pub control_size: Option<Size>,
    /// Optional role used to resolve interaction-state colors.
    pub color_role: Option<WidgetColorRole>,
    /// Disable pointer interaction and render disabled visuals.
    pub disabled: bool,
    /// Render focus affordances for keyboard/selection focus.
    pub focused: bool,
    /// Layout constraints.
    pub layout: LayoutBox,
}
