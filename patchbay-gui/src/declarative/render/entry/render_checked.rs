/// Render a UI specification and collect typed actions.
///
/// # Errors
/// Returns [`DeclarativeError`] when validation fails.
pub fn render_checked(
    spec: &UiSpec,
    ui: &mut Ui<'_>,
    origin: Point,
) -> Result<RenderResult, DeclarativeError> {
    let mut engine = LayoutEngineState::default();
    render_checked_with_engine(spec, ui, origin, &mut engine)
}

/// Render a UI specification using persistent layout engine state.
///
/// The engine tracks a deterministic node registry and subtree-aware measure
/// cache so repeated renders with unchanged subtrees avoid redundant work.
///
/// # Errors
/// Returns [`DeclarativeError`] when validation fails.
pub fn render_checked_with_engine(
    spec: &UiSpec,
    ui: &mut Ui<'_>,
    origin: Point,
    engine: &mut LayoutEngineState,
) -> Result<RenderResult, DeclarativeError> {
    validate_spec(spec)?;
    let tokens = root_theme_tokens(spec);
    engine.sync_registry(spec);
    engine.apply_measure_invalidations();
    let measured_root = measure_root_frame_with_engine(&spec.root, &tokens, engine);
    let plan = root_render_plan(spec, ui, measured_root);
    let mut actions = Vec::new();
    let mut debug_border_candidates = Vec::new();
    let mut layout_diagnostics = Vec::new();
    let mut node_layout_diagnostics = Vec::new();
    let response = render_root_frame_and_collect(
        spec,
        ui,
        RootRenderPassState {
            origin,
            tokens: &tokens,
            resolved: plan.layout_size,
            actions: &mut actions,
            debug_border_candidates: &mut debug_border_candidates,
            layout_diagnostics: &mut layout_diagnostics,
            node_layout_diagnostics: &mut node_layout_diagnostics,
            diagnostics_mode: spec.root.layout_diagnostics_mode,
        },
    );
    append_structural_gap_diagnostics(&mut layout_diagnostics, engine.take_structural_gaps());
    draw_layout_debug_borders(ui, &debug_border_candidates);
    engine.consume_layout_dirty();
    Ok(build_render_result(
        plan,
        response,
        actions,
        layout_diagnostics,
        node_layout_diagnostics,
    ))
}

/// Resolve root-level theme tokens for a checked render pass.
fn root_theme_tokens(spec: &UiSpec) -> ThemeTokens {
    spec.root.tokens.unwrap_or_default()
}

/// Append layout diagnostics derived from structural gap entries.
fn append_structural_gap_diagnostics(
    diagnostics: &mut Vec<LayoutDiagnostic>,
    gaps: Vec<StructuralGapEntry>,
) {
    if gaps.is_empty() {
        return;
    }
    for gap in gaps {
        diagnostics.push(structural_gap_diagnostic(gap.reason));
    }
}

/// Build one structured runtime diagnostic for a missing-node gap event.
fn structural_gap_diagnostic(reason: StructuralGapReason) -> LayoutDiagnostic {
    let empty = Rect {
        origin: Point { x: 0, y: 0 },
        size: Size {
            width: 0,
            height: 0,
        },
    };
    LayoutDiagnostic {
        level: LayoutDiagnosticLevel::Warning,
        code: LayoutDiagnosticCode::StructuralGapDetected,
        container: LayoutContainerKind::RootFrame,
        message: reason.diagnostic_message(),
        requested_rect: empty,
        bounds: empty,
    }
}

/// Build the root render plan using the current UI window size.
fn root_render_plan(spec: &UiSpec, ui: &Ui<'_>, measured_root: Size) -> RootRenderPlan {
    plan_root_render_with_measured(spec, ui.input().window_size, Some(measured_root))
}

/// Render the root frame while collecting actions and debug candidates.
fn render_root_frame_and_collect(
    spec: &UiSpec,
    ui: &mut Ui<'_>,
    state: RootRenderPassState<'_>,
) -> crate::ui::RootFrameResponse {
    let style = root_frame_style(spec, state.tokens);
    let mut ctx = RenderCtx {
        tokens: state.tokens,
        actions: state.actions,
        debug_border_candidates: state.debug_border_candidates,
        layout_diagnostics: state.layout_diagnostics,
        node_layout_diagnostics: state.node_layout_diagnostics,
        diagnostics_mode: state.diagnostics_mode,
        root_content_width: state.resolved.width,
        node_path: Vec::new(),
        parent_rects: Vec::new(),
        pending_node_reasons: Vec::new(),
        node_sequence: 0,
        depth: 1,
        curve_segment_move: None,
        curve_point_horizontal_constraint: None,
    };
    let response =
        ui.root_frame_with_key_at(&spec.root.key, style, Some(state.resolved), state.origin, |ui, rect| {
            ctx.root_content_width = rect.size.width;
            render_node(&spec.root.content, rect, ui, &mut ctx);
        });
    collect_container_debug_border_candidate(
        ctx.debug_border_candidates,
        ui,
        response.outer_rect,
        ContainerKind::RootFrame,
        0,
    );
    response
}

/// Mutable state bundle shared during one root-frame render pass.
struct RootRenderPassState<'a> {
    /// Root origin where the frame should be rendered.
    origin: Point,
    /// Resolved theme tokens used for rendering.
    tokens: &'a ThemeTokens,
    /// Resolved root layout size.
    resolved: Size,
    /// Collected typed actions for this frame.
    actions: &'a mut Vec<UiAction>,
    /// Candidate debug border targets discovered during render.
    debug_border_candidates: &'a mut Vec<DebugBorderCandidate>,
    /// Runtime layout diagnostics discovered during render.
    layout_diagnostics: &'a mut Vec<LayoutDiagnostic>,
    /// Per-node geometry diagnostics discovered during render.
    node_layout_diagnostics: &'a mut Vec<LayoutNodeDiagnostic>,
    /// Root diagnostics collection mode.
    diagnostics_mode: LayoutDiagnosticsMode,
}

/// Build the root frame style from root spec and resolved tokens.
fn root_frame_style<'a>(spec: &'a UiSpec, tokens: &ThemeTokens) -> RootFrameStyle<'a> {
    RootFrameStyle {
        title: spec.root.title.as_deref(),
        padding: spec.root.padding,
        background: Some(tokens.colors.surface),
        outline: Some(tokens.colors.border),
        header_height: Some(panel_header_height(spec.root.title.as_deref(), tokens)),
    }
}

/// Draw the selected container debug border if a valid candidate exists.
fn draw_selected_container_debug_border(
    ui: &mut Ui<'_>,
    debug_border_candidates: &[DebugBorderCandidate],
) {
    if let Some(candidate) = select_container_debug_border_candidate(debug_border_candidates)
        && let Some(color) = container_debug_border_color(candidate.kind, candidate.depth)
        && let Some(draw_rect) = debug_border_draw_rect(candidate.rect, 1)
    {
        ui.debug_stroke_rect(draw_rect, 1, color);
    }
}

/// Draw all matching layout border candidates for dense debug inspection.
#[cfg(feature = "layout-debug-borders")]
fn draw_all_container_debug_borders(ui: &mut Ui<'_>, candidates: &[DebugBorderCandidate]) {
    for candidate in candidates {
        let Some(color) = container_debug_border_color(candidate.kind, candidate.depth) else {
            continue;
        };
        if let Some(draw_rect) = debug_border_draw_rect(candidate.rect, 1) {
            ui.debug_stroke_rect(draw_rect, 1, color);
        }
    }
}

/// Render debug borders according to the configured debug mode.
fn draw_layout_debug_borders(ui: &mut Ui<'_>, candidates: &[DebugBorderCandidate]) {
    #[cfg(feature = "layout-debug-borders")]
    if should_draw_all_layout_debug_borders() {
        draw_all_container_debug_borders(ui, candidates);
        return;
    }

    draw_selected_container_debug_border(ui, candidates);
}

/// Build the final checked render result payload.
fn build_render_result(
    plan: RootRenderPlan,
    response: crate::ui::RootFrameResponse,
    actions: Vec<UiAction>,
    layout_diagnostics: Vec<LayoutDiagnostic>,
    node_layout_diagnostics: Vec<LayoutNodeDiagnostic>,
) -> RenderResult {
    let overflow = LayoutOverflowSummary::from_diagnostics(&layout_diagnostics);
    RenderResult {
        measured_size: plan.layout_size,
        actions,
        resolved_scale: plan.resolved_scale,
        content_rect: response.content_rect,
        layout_diagnostics,
        overflow,
        node_layout_diagnostics,
    }
}
