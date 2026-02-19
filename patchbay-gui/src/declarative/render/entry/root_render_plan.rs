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
    let normalized_zoom = normalize_zoom_override(zoom_override);

    #[cfg(feature = "layout-overflow-warnings")]
    if (zoom_override - normalized_zoom).abs() > f32::EPSILON {
        eprintln!(
            "patchbay-gui warning: replacing invalid root zoom override ({zoom_override:?}) with {normalized_zoom:.3}"
        );
    }

    debug_assert!(normalized_zoom > 0.0, "Zoom override must be positive");
    debug_assert!(normalized_zoom.is_finite(), "Zoom override must be finite");
    let base = match root.scale_mode {
        RootScaleMode::None => 1.0,
        RootScaleMode::UniformFit => {
            let fit_width = surface.width.max(1) as f32 / design.width as f32;
            let fit_height = surface.height.max(1) as f32 / design.height as f32;
            fit_width.min(fit_height)
        }
    };
    let scaled = base * normalized_zoom;
    debug_assert!(scaled.is_finite(), "Resolved root scale must be finite");
    scaled.clamp(0.0, 8.0)
}

/// Return a safe zoom factor for root rendering.
///
/// Invalid values (non-finite or non-positive) fall back to `1.0`.
fn normalize_zoom_override(raw_zoom: f32) -> f32 {
    if raw_zoom.is_finite() && raw_zoom > 0.0 {
        raw_zoom
    } else {
        1.0
    }
}

#[cfg(test)]
mod root_scale_tests {
    use super::normalize_zoom_override;

    #[test]
    fn normalize_zoom_override_falls_back_to_default_for_invalid_values() {
        assert_eq!(normalize_zoom_override(-1.0), 1.0);
        assert_eq!(normalize_zoom_override(0.0), 1.0);
        assert_eq!(normalize_zoom_override(f32::NAN), 1.0);
        assert_eq!(normalize_zoom_override(f32::INFINITY), 1.0);
        assert_eq!(normalize_zoom_override(2.5), 2.5);
    }
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
    debug_assert!(resolved_scale.is_finite());
    debug_assert!(layout_size.width > 0 && layout_size.height > 0);
    debug_assert!(surface.width > 0 && surface.height > 0);

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

/// Resolve per-axis scale factors from layout coordinates to surface coordinates.
///
/// In `RootScaleMode::UniformFit`, the scale is uniform, while `None` maps the
/// layout directly into the available content rectangle.
fn resolve_surface_scale(
    layout_size: Size,
    content_rect_surface: Size,
    resolved_scale: f32,
    scale_mode: RootScaleMode,
) -> (f32, f32) {
    debug_assert!(content_rect_surface.width > 0 && content_rect_surface.height > 0);
    debug_assert!(resolved_scale.is_finite());

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

/// In UniformFit mode, keep layout size within the design viewport.
fn clamp_layout_for_uniform_fit(layout_size: Size, layout_viewport: Size) -> Size {
    if layout_size.width > layout_viewport.width || layout_size.height > layout_viewport.height {
        warn_uniform_fit_clamp(layout_size, layout_viewport);
    }

    let bounded_width = layout_size.width.min(layout_viewport.width);
    let bounded_height = layout_size.height.min(layout_viewport.height);
    Size {
        width: bounded_width,
        height: bounded_height,
    }
}

#[cfg(feature = "layout-overflow-warnings")]
/// Emit a debug warning when uniform-fit clipping has to reduce the requested
/// root layout dimensions to fit the design viewport.
fn warn_uniform_fit_clamp(requested: Size, viewport: Size) {
    eprintln!(
        "patchbay-gui warning: uniform-fit root layout {}x{} exceeds viewport {}x{} and will be clamped",
        requested.width, requested.height, viewport.width, viewport.height
    );
}

#[cfg(not(feature = "layout-overflow-warnings"))]
/// No-op implementation when overflow diagnostics are disabled.
fn warn_uniform_fit_clamp(_requested: Size, _viewport: Size) {}

/// Build resolved root render metadata for a UI frame.
pub(crate) fn plan_root_render(spec: &UiSpec, surface_size: Size) -> RootRenderPlan {
    plan_root_render_with_measured(spec, surface_size, None)
}

/// Build resolved root render metadata using an optional precomputed root
/// measurement.
fn plan_root_render_with_measured(
    spec: &UiSpec,
    surface_size: Size,
    measured_override: Option<Size>,
) -> RootRenderPlan {
    let surface = clamp_non_zero_size(surface_size);
    let tokens = spec.root.tokens.unwrap_or_default();
    let measured = clamp_non_zero_size(
        measured_override.unwrap_or_else(|| measure_root_frame(&spec.root, &tokens)),
    );
    let resolved_scale = resolve_root_scale(&spec.root, measured, surface);
    let layout_viewport = resolve_root_layout_viewport(&spec.root, measured, surface);
    let layout_size =
        clamp_non_zero_size(resolve_size(spec.root.layout, measured, layout_viewport));
    let layout_size = match spec.root.scale_mode {
        RootScaleMode::None => layout_size,
        RootScaleMode::UniformFit => clamp_layout_for_uniform_fit(layout_size, layout_viewport),
    };
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

    let plan = RootRenderPlan {
        layout_size,
        resolved_scale,
        transform,
    };

    debug_assert!(plan.layout_size.width > 0 && plan.layout_size.height > 0);
    debug_assert!(plan.transform.scale_x.is_finite() && plan.transform.scale_y.is_finite());
    debug_assert!(
        plan.transform.offset_x >= 0.0 && plan.transform.offset_y >= 0.0
    );
    debug_assert!(
        plan.transform.content_rect_surface.origin.x >= 0
            && plan.transform.content_rect_surface.origin.y >= 0
    );
    debug_assert!(
        plan.transform
            .content_rect_surface
            .origin
            .x as u32
            <= surface.width
            && plan.transform
                .content_rect_surface
                .origin
                .y as u32
                <= surface.height
    );
    debug_assert!(
        (plan.transform.content_rect_surface.origin.x as u32)
            .saturating_add(plan.transform.content_rect_surface.size.width)
            <= surface.width
    );
    debug_assert!(
        (plan.transform.content_rect_surface.origin.y as u32)
            .saturating_add(plan.transform.content_rect_surface.size.height)
            <= surface.height
    );

    plan
}
