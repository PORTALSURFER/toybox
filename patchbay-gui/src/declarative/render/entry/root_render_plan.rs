/// Clamp a size so both dimensions are at least one pixel.
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
    let surface_width = surface.width.max(1) as f32;
    let surface_height = surface.height.max(1) as f32;
    let scaled_width = match scale_mode {
        RootScaleMode::None => (layout_size.width.max(1) as f32 * resolved_scale)
            .round()
            .max(1.0)
            .min(surface_width),
        RootScaleMode::UniformFit => (layout_size.width.max(1) as f32 * resolved_scale)
            .round()
            .max(1.0)
            .min(surface_width),
    };
    let scaled_height = match scale_mode {
        RootScaleMode::None => (layout_size.height.max(1) as f32 * resolved_scale)
            .round()
            .max(1.0)
            .min(surface_height),
        RootScaleMode::UniformFit => (layout_size.height.max(1) as f32 * resolved_scale)
            .round()
            .max(1.0)
            .min(surface_height),
    };

    let (origin_x, origin_y): (f32, f32) = match scale_mode {
        RootScaleMode::None => (0.0, 0.0),
        RootScaleMode::UniformFit => (
            (surface_width - scaled_width) / 2.0,
            (surface_height - scaled_height) / 2.0,
        ),
    };

    Rect {
        origin: Point {
            x: origin_x.round() as i32,
            y: origin_y.round() as i32,
        },
        size: Size {
            width: scaled_width as u32,
            height: scaled_height as u32,
        },
    }
}

fn resolve_surface_scale(
    layout_size: Size,
    content_rect_surface: Size,
    resolved_scale: f32,
    scale_mode: RootScaleMode,
) -> (f32, f32) {
    match scale_mode {
        RootScaleMode::None => {
            let layout_width = layout_size.width.max(1) as f32;
            let layout_height = layout_size.height.max(1) as f32;
            let content_width = content_rect_surface.width.max(1) as f32;
            let content_height = content_rect_surface.height.max(1) as f32;
            (content_width / layout_width, content_height / layout_height)
        }
        RootScaleMode::UniformFit => (resolved_scale, resolved_scale),
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
    let (scale_x, scale_y) = resolve_surface_scale(
        layout_size,
        content_rect_surface.size,
        resolved_scale,
        spec.root.scale_mode,
    );
    let transform = RootTransform {
        scale_x,
        scale_y,
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
