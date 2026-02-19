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
    Node::Spacer(SpacerSpec::new(size))
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

/// Create an interactive region node.
pub fn region(key: impl Into<String>, size: Size) -> Node {
    Node::Region(RegionSpec::new(key, size))
}

/// Create a structured surface node for custom graphics content.
pub fn surface(
    key: impl Into<String>,
    size: Size,
    commands: Vec<SurfaceCommand>,
) -> Node {
    Node::Region(
        RegionSpec::new(key, size).draw_commands(commands),
    )
}

/// Create an indicator node.
pub fn indicator(size: Size, active: bool) -> Node {
    Node::Indicator(IndicatorSpec::new(size, active))
}
