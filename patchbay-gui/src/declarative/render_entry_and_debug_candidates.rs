
impl Default for SpacingTokens {
    fn default() -> Self {
        Self {
            xs: 4,
            sm: 8,
            md: 12,
            lg: 16,
        }
    }
}

/// Control-size token set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlTokens {
    /// Default knob diameter.
    pub knob_diameter: u32,
    /// Default slider width.
    pub slider_width: u32,
    /// Default slider height.
    pub slider_height: u32,
    /// Default toggle width.
    pub toggle_width: u32,
    /// Default toggle height.
    pub toggle_height: u32,
    /// Default button width.
    pub button_width: u32,
    /// Default button height.
    pub button_height: u32,
    /// Default dropdown width.
    pub dropdown_width: u32,
    /// Default dropdown height.
    pub dropdown_height: u32,
}

impl Default for ControlTokens {
    fn default() -> Self {
        Self {
            knob_diameter: 32,
            slider_width: 180,
            slider_height: 28,
            toggle_width: 64,
            toggle_height: 28,
            button_width: 120,
            button_height: 28,
            dropdown_width: 180,
            dropdown_height: 28,
        }
    }
}

/// Root design tokens for declarative rendering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ThemeTokens {
    /// Color token set.
    pub colors: ColorTokens,
    /// Typography token set.
    pub typography: TypographyTokens,
    /// Spacing token set.
    pub spacing: SpacingTokens,
    /// Control token set.
    pub controls: ControlTokens,
}

impl ThemeTokens {
    /// Build declarative tokens from a semantic palette and default sizing.
    pub const fn from_palette(palette: MainPalette) -> Self {
        Self {
            colors: ColorTokens::from_palette(palette),
            typography: TypographyTokens { text_scale: 2 },
            spacing: SpacingTokens {
                xs: 4,
                sm: 8,
                md: 12,
                lg: 16,
            },
            controls: ControlTokens {
                knob_diameter: 32,
                slider_width: 180,
                slider_height: 28,
                toggle_width: 64,
                toggle_height: 28,
                button_width: 120,
                button_height: 28,
                dropdown_width: 180,
                dropdown_height: 28,
            },
        }
    }

    /// Return the canonical declarative token set.
    pub const fn main() -> Self {
        Self::from_palette(MainPalette::main())
    }
}

/// Measure the required size for a UI specification.
///
/// # Errors
/// Returns [`DeclarativeError`] when validation fails.
pub fn measure_checked(spec: &UiSpec) -> Result<Size, DeclarativeError> {
    validate_spec(spec)?;
    let tokens = spec.root.tokens.unwrap_or_default();
    Ok(measure_root_frame(&spec.root, &tokens))
}

fn clamp_non_zero_size(size: Size) -> Size {
    Size {
        width: size.width.max(1),
        height: size.height.max(1),
    }
}

/// Resolve root render scale from viewport and root scaling policy.
fn resolve_root_scale(root: &RootFrameSpec, measured: Size, surface: Size) -> f32 {
    let design = clamp_non_zero_size(root.design_size.unwrap_or(measured));
    let zoom_override = root.zoom_override.unwrap_or(1.0);
    let base = match root.scale_mode {
        RootScaleMode::None => 1.0,
        RootScaleMode::UniformFit => {
            let fit_width = surface.width.max(1) as f32 / design.width as f32;
            let fit_height = surface.height.max(1) as f32 / design.height as f32;
            fit_width.min(fit_height)
        }
    };
    (base * zoom_override).clamp(0.1, 8.0)
}

/// Resolve the design-space viewport used for root layout.
fn resolve_root_layout_viewport(root: &RootFrameSpec, measured: Size, surface: Size) -> Size {
    match root.scale_mode {
        RootScaleMode::None => clamp_non_zero_size(surface),
        RootScaleMode::UniformFit => clamp_non_zero_size(root.design_size.unwrap_or(measured)),
    }
}

/// Resolve surface-space output bounds for transformed root content.
fn resolve_surface_content_rect(
    layout_size: Size,
    surface: Size,
    resolved_scale: f32,
    scale_mode: RootScaleMode,
) -> Rect {
    let scaled_width = (layout_size.width.max(1) as f32 * resolved_scale)
        .round()
        .max(1.0);
    let scaled_height = (layout_size.height.max(1) as f32 * resolved_scale)
        .round()
        .max(1.0);

    let (origin_x, origin_y): (f32, f32) = match scale_mode {
        RootScaleMode::None => (0.0, 0.0),
        RootScaleMode::UniformFit => (0.0, 0.0),
    };

    Rect {
        origin: Point {
            x: origin_x.round() as i32,
            y: origin_y.round() as i32,
        },
        size: Size {
            width: match scale_mode {
                RootScaleMode::None => scaled_width as u32,
                RootScaleMode::UniformFit => surface.width.max(1),
            },
            height: match scale_mode {
                RootScaleMode::None => scaled_height as u32,
                RootScaleMode::UniformFit => surface.height.max(1),
            },
        },
    }
}

/// Build resolved root render metadata for a UI frame.
pub(crate) fn plan_root_render(spec: &UiSpec, surface_size: Size) -> RootRenderPlan {
    let surface = clamp_non_zero_size(surface_size);
    let tokens = spec.root.tokens.unwrap_or_default();
    let measured = clamp_non_zero_size(measure_root_frame(&spec.root, &tokens));
    let resolved_scale = resolve_root_scale(&spec.root, measured, surface);
    let layout_viewport = resolve_root_layout_viewport(&spec.root, measured, surface);
    let layout_size =
        clamp_non_zero_size(resolve_size(spec.root.layout, measured, layout_viewport));
    let content_rect_surface =
        resolve_surface_content_rect(layout_size, surface, resolved_scale, spec.root.scale_mode);
    let transform = RootTransform {
        scale_x: content_rect_surface.size.width.max(1) as f32 / layout_size.width as f32,
        scale_y: content_rect_surface.size.height.max(1) as f32 / layout_size.height as f32,
        offset_x: content_rect_surface.origin.x as f32,
        offset_y: content_rect_surface.origin.y as f32,
        content_rect_design: Rect {
            origin: Point { x: 0, y: 0 },
            size: layout_size,
        },
        content_rect_surface,
    };

    RootRenderPlan {
        layout_size,
        resolved_scale,
        transform,
    }
}

/// Render a UI specification and collect typed actions.
///
/// # Errors
/// Returns [`DeclarativeError`] when validation fails.
pub fn render_checked(
    spec: &UiSpec,
    ui: &mut Ui<'_>,
    origin: Point,
) -> Result<RenderResult, DeclarativeError> {
    validate_spec(spec)?;
    let tokens = spec.root.tokens.unwrap_or_default();
    let plan = plan_root_render(spec, ui.input().window_size);
    let resolved = plan.layout_size;

    let style = RootFrameStyle {
        title: spec.root.title.as_deref(),
        padding: spec.root.padding,
        background: Some(tokens.colors.surface),
        outline: Some(tokens.colors.border),
        header_height: Some(panel_header_height(spec.root.title.as_deref(), &tokens)),
    };

    let mut actions = Vec::new();
    let mut debug_border_candidates = Vec::new();
    let response = {
        let mut ctx = RenderCtx {
            tokens: &tokens,
            actions: &mut actions,
            debug_border_candidates: &mut debug_border_candidates,
            depth: 1,
        };
        let response =
            ui.root_frame_with_key_at(&spec.root.key, style, Some(resolved), origin, |ui, rect| {
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
    };
    if let Some(candidate) = select_container_debug_border_candidate(&debug_border_candidates)
        && let Some(color) = container_debug_border_color(candidate.kind, candidate.depth)
        && let Some(draw_rect) = debug_border_draw_rect(candidate.rect, 1)
    {
        ui.debug_stroke_rect(draw_rect, 1, color);
    }

    Ok(RenderResult {
        measured_size: resolved,
        actions,
        resolved_scale: plan.resolved_scale,
        content_rect: response.content_rect,
    })
}

/// Declarative container node kinds that can emit debug layout borders.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ContainerKind {
    RootFrame,
    Panel,
    Flex,
    Grid,
    Absolute,
}

/// Candidate border outline emitted while traversing container nodes.
#[derive(Clone, Copy, Debug, PartialEq)]
struct DebugBorderCandidate {
    rect: Rect,
    kind: ContainerKind,
    depth: usize,
}

/// Return the optional debug border color for a container kind/depth pair.
fn container_debug_border_color(kind: ContainerKind, depth: usize) -> Option<Color> {
    #[cfg(feature = "layout-debug-borders")]
    {
        let _ = depth;
        match kind {
            ContainerKind::RootFrame => None,
            ContainerKind::Panel
            | ContainerKind::Flex
            | ContainerKind::Grid
            | ContainerKind::Absolute => Some(Color::rgb(245, 98, 98)),
        }
    }
    #[cfg(not(feature = "layout-debug-borders"))]
    {
        let _ = (kind, depth);
        None
    }
}

/// Record a hovered layout-debug border candidate for later selection.
fn collect_container_debug_border_candidate(
    candidates: &mut Vec<DebugBorderCandidate>,
    ui: &Ui<'_>,
    rect: Rect,
    kind: ContainerKind,
    depth: usize,
) {
    // Skip root-level wrappers so debug outlines focus on meaningful inner
    // layout partitions instead of the full-window container.
    if !should_draw_container_debug_border(kind, depth, rect.contains(ui.input().pointer_pos)) {
        return;
    }
    if container_debug_border_color(kind, depth).is_none() {
        return;
    }
    candidates.push(DebugBorderCandidate { rect, kind, depth });
}
