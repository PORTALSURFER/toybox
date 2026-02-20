/// Mutable render-state threaded through declarative node traversal.
struct RenderCtx<'a> {
    /// Theme tokens used for sizing and rendering.
    tokens: &'a ThemeTokens,
    /// Collected UI actions for the current frame.
    actions: &'a mut Vec<UiAction>,
    /// Candidate container rectangles for debug-border selection.
    debug_border_candidates: &'a mut Vec<DebugBorderCandidate>,
    /// Runtime layout diagnostics collected during this render pass.
    layout_diagnostics: &'a mut Vec<LayoutDiagnostic>,
    /// Per-node geometry diagnostics collected when enabled.
    node_layout_diagnostics: &'a mut Vec<LayoutNodeDiagnostic>,
    /// Root diagnostics collection mode.
    diagnostics_mode: LayoutDiagnosticsMode,
    /// Root content width used by responsive containers.
    root_content_width: u32,
    /// Deterministic path segments for the active traversal branch.
    node_path: Vec<String>,
    /// Parent rectangle stack aligned with `node_path`.
    parent_rects: Vec<Rect>,
    /// Pending reason flags for the next `render_node` invocation.
    pending_node_reasons: Vec<Vec<LayoutNodeDiagnosticReason>>,
    /// Monotonic traversal sequence counter for stable node-path IDs.
    node_sequence: usize,
    /// Current container depth in the render tree.
    depth: usize,
}

/// Queue one per-node reason flag for the next rendered child.
fn queue_next_node_reason(ctx: &mut RenderCtx<'_>, reason: LayoutNodeDiagnosticReason) {
    queue_next_node_reasons(ctx, vec![reason]);
}

/// Queue per-node reason flags for the next rendered child.
fn queue_next_node_reasons(ctx: &mut RenderCtx<'_>, reasons: Vec<LayoutNodeDiagnosticReason>) {
    if ctx.diagnostics_mode != LayoutDiagnosticsMode::PerNode || reasons.is_empty() {
        return;
    }
    ctx.pending_node_reasons.push(reasons);
}

/// Return overflow reason when a child rectangle was adjusted.
fn overflow_reason(
    requested_rect: Rect,
    resolved_rect: Rect,
    overflow_policy: OverflowPolicy,
) -> Option<LayoutNodeDiagnosticReason> {
    if requested_rect == resolved_rect {
        return None;
    }
    Some(match overflow_policy {
        OverflowPolicy::Clip => LayoutNodeDiagnosticReason::OverflowClipped,
        OverflowPolicy::Compress => LayoutNodeDiagnosticReason::OverflowCompressed,
    })
}

/// Record one per-node layout diagnostic entry when enabled.
fn record_node_layout_diagnostic(
    ctx: &mut RenderCtx<'_>,
    node: &Node,
    rect: Rect,
    parent_rect: Option<Rect>,
    pending_reasons: Vec<LayoutNodeDiagnosticReason>,
) {
    if ctx.diagnostics_mode != LayoutDiagnosticsMode::PerNode {
        return;
    }

    let measured = measure_node(node, ctx.tokens);
    let measured_rect = Rect {
        origin: rect.origin,
        size: measured,
    };
    let mut reasons = vec![
        LayoutNodeDiagnosticReason::Measured,
        LayoutNodeDiagnosticReason::Resolved,
    ];
    for reason in pending_reasons {
        push_reason(&mut reasons, reason);
    }
    if let Some(parent) = parent_rect
        && rect.origin != parent.origin
    {
        push_reason(&mut reasons, LayoutNodeDiagnosticReason::Aligned);
    }
    if rect.size.width > measured.width || rect.size.height > measured.height {
        push_reason(&mut reasons, LayoutNodeDiagnosticReason::ClampedMin);
    }
    if rect.size.width < measured.width || rect.size.height < measured.height {
        push_reason(&mut reasons, LayoutNodeDiagnosticReason::ClampedMax);
    }
    ctx.node_layout_diagnostics.push(LayoutNodeDiagnostic {
        node_path: ctx.node_path.join("/"),
        node_kind: node_layout_kind(node),
        measured_rect,
        resolved_rect: rect,
        reasons,
        container: node_container_kind(node),
    });
}

/// Append one reason flag unless it already exists.
fn push_reason(reasons: &mut Vec<LayoutNodeDiagnosticReason>, reason: LayoutNodeDiagnosticReason) {
    if !reasons.contains(&reason) {
        reasons.push(reason);
    }
}

/// Resolve per-node diagnostic kind from declarative node variant.
fn node_layout_kind(node: &Node) -> LayoutNodeKind {
    match node {
        Node::Slot(_) => LayoutNodeKind::Slot,
        Node::Panel(_) => LayoutNodeKind::Panel,
        Node::PaddingBox(_) => LayoutNodeKind::PaddingBox,
        Node::AlignBox(_) => LayoutNodeKind::AlignBox,
        Node::AspectBox(_) => LayoutNodeKind::AspectBox,
        Node::Row(_) => LayoutNodeKind::Row,
        Node::Column(_) => LayoutNodeKind::Column,
        Node::Grid(_) => LayoutNodeKind::Grid,
        Node::Absolute(_) => LayoutNodeKind::Absolute,
        Node::Stack(_) => LayoutNodeKind::Stack,
        Node::ScrollView(_) => LayoutNodeKind::ScrollView,
        Node::Wrap(_) => LayoutNodeKind::Wrap,
        Node::SwitchLayout(_) => LayoutNodeKind::SwitchLayout,
        Node::TextBox(_) => LayoutNodeKind::TextBox,
        Node::Spacer(_) => LayoutNodeKind::Spacer,
        Node::Knob(_) => LayoutNodeKind::Knob,
        Node::Slider(_) => LayoutNodeKind::Slider,
        Node::Toggle(_) => LayoutNodeKind::Toggle,
        Node::Button(_) => LayoutNodeKind::Button,
        Node::Dropdown(_) => LayoutNodeKind::Dropdown,
        Node::Region(_) => LayoutNodeKind::Region,
        Node::Indicator(_) => LayoutNodeKind::Indicator,
    }
}

/// Resolve optional container kind for node diagnostics.
fn node_container_kind(node: &Node) -> Option<LayoutContainerKind> {
    Some(match node {
        Node::Slot(_) => LayoutContainerKind::Slot,
        Node::Panel(_) => LayoutContainerKind::Panel,
        Node::PaddingBox(_) => LayoutContainerKind::PaddingBox,
        Node::AlignBox(_) => LayoutContainerKind::AlignBox,
        Node::AspectBox(_) => LayoutContainerKind::AspectBox,
        Node::Row(_) | Node::Column(_) => LayoutContainerKind::Flex,
        Node::Grid(_) => LayoutContainerKind::Grid,
        Node::Absolute(_) => LayoutContainerKind::Absolute,
        Node::Stack(_) => LayoutContainerKind::Stack,
        Node::ScrollView(_) => LayoutContainerKind::ScrollView,
        Node::Wrap(_) => LayoutContainerKind::Wrap,
        Node::SwitchLayout(_) => LayoutContainerKind::SwitchLayout,
        _ => return None,
    })
}

/// Resolve a deterministic diagnostic path segment for one node visit.
fn node_path_segment(node: &Node, sequence: usize) -> String {
    let base = match node {
        Node::Slot(_) => "slot".to_string(),
        Node::Panel(panel) => format!("panel:{}", sanitize_path_segment(&panel.key)),
        Node::PaddingBox(_) => "padding-box".to_string(),
        Node::AlignBox(_) => "align-box".to_string(),
        Node::AspectBox(_) => "aspect-box".to_string(),
        Node::Row(_) => "row".to_string(),
        Node::Column(_) => "column".to_string(),
        Node::Grid(_) => "grid".to_string(),
        Node::Absolute(_) => "absolute".to_string(),
        Node::Stack(_) => "stack".to_string(),
        Node::ScrollView(_) => "scroll-view".to_string(),
        Node::Wrap(_) => "wrap".to_string(),
        Node::SwitchLayout(_) => "switch-layout".to_string(),
        Node::TextBox(_) => "textbox".to_string(),
        Node::Spacer(_) => "spacer".to_string(),
        Node::Knob(knob) => format!("knob:{}", sanitize_path_segment(&knob.key)),
        Node::Slider(slider) => format!("slider:{}", sanitize_path_segment(&slider.key)),
        Node::Toggle(toggle) => format!("toggle:{}", sanitize_path_segment(&toggle.key)),
        Node::Button(button) => format!("button:{}", sanitize_path_segment(&button.key)),
        Node::Dropdown(dropdown) => format!("dropdown:{}", sanitize_path_segment(&dropdown.key)),
        Node::Region(region) => format!("region:{}", sanitize_path_segment(&region.key)),
        Node::Indicator(_) => "indicator".to_string(),
    };
    format!("{base}[{sequence}]")
}

/// Replace path separator characters in diagnostic path segments.
fn sanitize_path_segment(raw: &str) -> String {
    raw.replace('/', "_")
}

/// Render a node subtree and collect actions.
fn render_node(node: &Node, rect: Rect, ui: &mut Ui<'_>, ctx: &mut RenderCtx<'_>) {
    let collect_node_diagnostics = ctx.diagnostics_mode == LayoutDiagnosticsMode::PerNode;
    if collect_node_diagnostics {
        let sequence = ctx.node_sequence;
        ctx.node_sequence = ctx.node_sequence.saturating_add(1);
        ctx.node_path.push(node_path_segment(node, sequence));
        let parent_rect = ctx.parent_rects.last().copied();
        let pending_reasons = ctx.pending_node_reasons.pop().unwrap_or_default();
        record_node_layout_diagnostic(ctx, node, rect, parent_rect, pending_reasons);
        ctx.parent_rects.push(rect);
    }

    ui.with_clip(rect, |ui| match node {
        Node::Slot(slot) => {
            collect_container_debug_border_candidate(
                ctx.debug_border_candidates,
                ui,
                rect,
                ContainerKind::Slot,
                ctx.depth,
            );
            render_node(&slot.child, rect, ui, ctx)
        }
        Node::Panel(panel) => render_panel(panel, rect, ui, ctx),
        Node::PaddingBox(padding_box) => render_padding_box(padding_box, rect, ui, ctx),
        Node::AlignBox(align_box) => render_align_box(align_box, rect, ui, ctx),
        Node::AspectBox(aspect_box) => render_aspect_box(aspect_box, rect, ui, ctx),
        Node::Row(flex) => render_flex(flex, rect, ui, Axis::Horizontal, ctx),
        Node::Column(flex) => render_flex(flex, rect, ui, Axis::Vertical, ctx),
        Node::Grid(grid) => render_grid(grid, rect, ui, ctx),
        Node::Absolute(absolute) => render_absolute(absolute, rect, ui, ctx),
        Node::Stack(stack) => render_stack(stack, rect, ui, ctx),
        Node::ScrollView(scroll_view) => render_scroll_view(scroll_view, rect, ui, ctx),
        Node::Wrap(wrap) => render_wrap(wrap, rect, ui, ctx),
        Node::SwitchLayout(switch_layout) => render_switch_layout(switch_layout, rect, ui, ctx),
        Node::TextBox(text_box) => render_text_box(text_box, rect, ui, ctx.tokens, ctx.actions),
        Node::Spacer(_) => {}
        Node::Knob(knob) => render_knob(knob, rect, ui, ctx.tokens, ctx.actions),
        Node::Slider(slider) => render_slider(slider, rect, ui, ctx.tokens, ctx.actions),
        Node::Toggle(toggle) => render_toggle(toggle, rect, ui, ctx.tokens, ctx.actions),
        Node::Button(button) => render_button(button, rect, ui, ctx.tokens, ctx.actions),
        Node::Dropdown(dropdown) => render_dropdown(dropdown, rect, ui, ctx.tokens, ctx.actions),
        Node::Region(region) => render_region(region, rect, ui, ctx.actions),
        Node::Indicator(indicator) => render_indicator(indicator, rect, ui),
    });

    if collect_node_diagnostics {
        ctx.parent_rects.pop();
        ctx.node_path.pop();
    }
}
