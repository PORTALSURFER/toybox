/// Mutable render-state threaded through declarative node traversal.
struct RenderCtx<'a> {
    /// Theme tokens used for sizing and rendering.
    tokens: &'a ThemeTokens,
    /// Collected UI actions for the current frame.
    actions: &'a mut Vec<UiAction>,
    /// Candidate container rectangles for debug-border selection.
    debug_border_candidates: &'a mut Vec<DebugBorderCandidate>,
    /// Current container depth in the render tree.
    depth: usize,
}

/// Render a node subtree and collect actions.
fn render_node(node: &Node, rect: Rect, ui: &mut Ui<'_>, ctx: &mut RenderCtx<'_>) {
    ui.with_clip(rect, |ui| match node {
        Node::Panel(panel) => render_panel(panel, rect, ui, ctx),
        Node::Row(flex) => render_flex(flex, rect, ui, Axis::Horizontal, ctx),
        Node::Column(flex) => render_flex(flex, rect, ui, Axis::Vertical, ctx),
        Node::Grid(grid) => render_grid(grid, rect, ui, ctx),
        Node::Absolute(absolute) => render_absolute(absolute, rect, ui, ctx),
        Node::Label(label) => render_label(label, rect, ui, ctx.tokens),
        Node::Spacer(_) => {}
        Node::Knob(knob) => render_knob(knob, rect, ui, ctx.tokens, ctx.actions),
        Node::Slider(slider) => render_slider(slider, rect, ui, ctx.tokens, ctx.actions),
        Node::Toggle(toggle) => render_toggle(toggle, rect, ui, ctx.tokens, ctx.actions),
        Node::Button(button) => render_button(button, rect, ui, ctx.tokens, ctx.actions),
        Node::Dropdown(dropdown) => render_dropdown(dropdown, rect, ui, ctx.tokens, ctx.actions),
        Node::Region(region) => render_region(region, rect, ui, ctx.actions),
        Node::Indicator(indicator) => render_indicator(indicator, rect, ui),
    });
}
