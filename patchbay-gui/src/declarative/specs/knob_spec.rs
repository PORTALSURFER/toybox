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
