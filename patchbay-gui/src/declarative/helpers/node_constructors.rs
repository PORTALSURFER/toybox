/// Create a grid container node.
pub fn grid(template: GridTemplate, children: Vec<Node>) -> Node {
    Node::Grid(GridSpec::new(template, children))
}

/// Create a panel container node.
pub fn panel(key: impl Into<String>, content: Node) -> Node {
    Node::Panel(PanelSpec::new(key, content))
}

/// Create a text box node.
pub fn textbox(text: impl Into<String>) -> Node {
    Node::TextBox(TextBoxSpec::new(text))
}

/// Create a fixed-size spacer node.
pub fn spacer(size: Size) -> Node {
    Node::Spacer(
        SpacerSpec::new().layout(LayoutBox::fixed(size.width, size.height).max(size.width, size.height)),
    )
}

/// Create a knob control node.
pub fn knob(
    key: impl Into<String>,
    value: f32,
    range: (f32, f32),
) -> Node {
    Node::Knob(KnobSpec::new(key, value, range))
}

/// Create a slider control node.
pub fn slider(
    key: impl Into<String>,
    value: f32,
    range: (f32, f32),
) -> Node {
    Node::Slider(SliderSpec::new(key, value, range))
}

/// Create a toggle control node.
pub fn toggle(key: impl Into<String>, value: bool) -> Node {
    Node::Toggle(ToggleSpec::new(key, value))
}

/// Create a button control node.
pub fn button(key: impl Into<String>) -> Node {
    Node::Button(ButtonSpec::new(key))
}

/// Create a dropdown control node.
pub fn dropdown(
    key: impl Into<String>,
    option_count: usize,
    selected: usize,
) -> Node {
    Node::Dropdown(DropdownSpec::new(key, option_count, selected))
}

/// Create a tab-bar control node.
pub fn tabbar(
    key: impl Into<String>,
    tab_count: usize,
    selected: usize,
) -> Node {
    Node::TabBar(TabBarSpec::new(key, tab_count, selected))
}

/// Create a curve-editor widget node.
pub fn curve_editor(key: impl Into<String>, model: CurveModel) -> Node {
    Node::CurveEditor(CurveEditorSpec::new(key, model))
}

/// Create an EQ attractor editing surface widget node.
///
/// # Examples
/// ```
/// use patchbay_gui::declarative::{
///     EqAttractor, EqAttractorSurfaceModel, EqAttractorSurfaceStyle, eq_attractor_surface,
/// };
///
/// let model = EqAttractorSurfaceModel::new(vec![EqAttractor::new(1, 0.4, 0.6)]);
/// let style = EqAttractorSurfaceStyle::default();
/// let _node = eq_attractor_surface("eq-surface", model, style);
/// ```
pub fn eq_attractor_surface(
    key: impl Into<String>,
    model: EqAttractorSurfaceModel,
    style: EqAttractorSurfaceStyle,
) -> Node {
    Node::EqAttractorSurface(EqAttractorSurfaceSpec::new(key, model, style))
}

/// Create an interactive region node.
pub fn region(key: impl Into<String>, size: Size) -> Node {
    Node::Region(
        RegionSpec::new(key).layout(LayoutBox::fixed(size.width, size.height).max(size.width, size.height)),
    )
}

/// Create a structured surface node for custom graphics content.
pub fn surface(
    key: impl Into<String>,
    size: Size,
    commands: Vec<SurfaceCommand>,
) -> Node {
    Node::Region(
        RegionSpec::new(key)
            .layout(LayoutBox::fixed(size.width, size.height).max(size.width, size.height))
            .draw_commands(commands),
    )
}

/// Create an indicator node.
pub fn indicator(size: Size, active: bool) -> Node {
    Node::Indicator(
        IndicatorSpec::new(active).layout(LayoutBox::fixed(size.width, size.height).max(size.width, size.height)),
    )
}
