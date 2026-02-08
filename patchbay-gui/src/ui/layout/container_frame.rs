/// Shared input specification for root/panel frame containers.
#[derive(Clone, Copy, Debug)]
struct ContainerFrameSpec<'a> {
    /// Stable key used for cached size lookup.
    key: &'a str,
    /// Optional title rendered in the header.
    title: Option<&'a str>,
    /// Padding applied around content.
    padding: i32,
    /// Optional explicit header height override.
    header_height: Option<i32>,
    /// Optional requested size supplied by caller.
    requested_size: Option<Size>,
    /// Frame origin in window coordinates.
    origin: Point,
    /// Frame background color.
    background: Color,
    /// Frame outline color.
    outline: Color,
}

/// Shared runtime values derived from a frame spec.
#[derive(Clone, Copy, Debug)]
struct ContainerFrameResolved<'a> {
    /// Widget id for state cache updates.
    id: WidgetId,
    /// Display title copied from spec for rendering.
    title: Option<&'a str>,
    /// Non-negative content padding.
    padding: i32,
    /// Header height resolved from style or typography defaults.
    header_height: i32,
    /// Requested size from caller.
    requested_size: Option<Size>,
    /// Origin used for drawing and cursor updates.
    origin: Point,
    /// Outer rect used to draw frame shell.
    outer_rect: Rect,
    /// Content origin after padding and header.
    content_origin: Point,
    /// Content rect passed to children.
    content_rect: Rect,
    /// Fill color.
    background: Color,
    /// Stroke color.
    outline: Color,
}

/// Controls how explicit size requests interact with measured content.
#[derive(Clone, Copy, Debug)]
enum ExplicitSizePolicy {
    /// Keep at least the explicit size while allowing larger measured content.
    PreserveExplicitMinimum,
    /// Always keep explicit size when it is provided.
    PreferExplicit,
}

/// Post-measure side effects for a container frame render.
#[derive(Clone, Copy, Debug, Default)]
struct ContainerFrameEffects {
    /// Move layout cursor below the measured frame.
    advance_layout_cursor: bool,
    /// Publish measured size as the root-frame size.
    update_root_frame_size: bool,
}

/// Final frame data returned to panel/root wrappers.
#[derive(Clone, Copy, Debug)]
struct ContainerFrameResult {
    /// The outer bounds of the frame.
    outer_rect: Rect,
    /// The content rectangle available to children.
    content_rect: Rect,
    /// The measured size persisted for future passes.
    measured_size: Size,
}

/// Run the full root/panel frame lifecycle and return measured metadata.
fn render_container_frame<F>(
    ui: &mut Ui<'_>,
    spec: ContainerFrameSpec<'_>,
    explicit_size_policy: ExplicitSizePolicy,
    effects: ContainerFrameEffects,
    mut f: F,
) -> ContainerFrameResult
where
    F: FnMut(&mut Ui<'_>, Rect),
{
    let id = WidgetId::from_label(spec.key);
    let resolved = resolve_container_frame(spec, id, ui.state.layout.get(id), ui.theme.text_scale);
    draw_container_frame_shell(ui, resolved);
    let measured_bounds = measure_container_frame_content(ui, &resolved, &mut f);
    let measured_size = finalize_container_frame_size(&resolved, measured_bounds, explicit_size_policy);
    persist_container_frame_state(ui, &resolved, measured_size, effects);
    ContainerFrameResult {
        outer_rect: resolved.outer_rect,
        content_rect: resolved.content_rect,
        measured_size,
    }
}

/// Resolve defaults, cached sizing and geometry for a frame render pass.
fn resolve_container_frame(
    spec: ContainerFrameSpec<'_>,
    id: WidgetId,
    cached_size: Option<Size>,
    text_scale: u32,
) -> ContainerFrameResolved<'_> {
    let padding = spec.padding.max(0);
    let text_scale_i32 = text_scale.min(i32::MAX as u32) as i32;
    let header_height =
        resolve_container_header_height(spec.title, spec.header_height, text_scale_i32);
    let fallback_size = fallback_container_size(padding, header_height);
    let resolved_size = resolve_container_size(spec.requested_size, cached_size, fallback_size);
    let outer_rect = Rect {
        origin: spec.origin,
        size: resolved_size,
    };
    let content_origin = Point {
        x: spec.origin.x + padding,
        y: spec.origin.y + padding + header_height,
    };
    let content_rect = Rect {
        origin: content_origin,
        size: Size {
            width: resolved_size.width.saturating_sub((padding * 2) as u32),
            height: resolved_size
                .height
                .saturating_sub((padding * 2 + header_height) as u32),
        },
    };
    ContainerFrameResolved {
        id,
        title: spec.title,
        padding,
        header_height,
        requested_size: spec.requested_size,
        origin: spec.origin,
        outer_rect,
        content_origin,
        content_rect,
        background: spec.background,
        outline: spec.outline,
    }
}

/// Resolve header height using explicit override or typography-aware defaults.
fn resolve_container_header_height(title: Option<&str>, explicit: Option<i32>, text_scale: i32) -> i32 {
    explicit.unwrap_or_else(|| {
        if title.is_some() {
            (8 * text_scale + 4).max(0)
        } else {
            0
        }
    })
}

/// Fallback size used when neither explicit nor cached size is available.
fn fallback_container_size(padding: i32, header_height: i32) -> Size {
    Size {
        width: (padding * 2 + 160).max(0) as u32,
        height: (padding * 2 + header_height + 80).max(0) as u32,
    }
}

/// Resolve size from explicit request, cache, or fallback defaults.
fn resolve_container_size(
    requested_size: Option<Size>,
    cached_size: Option<Size>,
    fallback_size: Size,
) -> Size {
    requested_size.or(cached_size).unwrap_or(fallback_size)
}

/// Draw the frame shell and optional header title.
fn draw_container_frame_shell(ui: &mut Ui<'_>, resolved: ContainerFrameResolved<'_>) {
    ui.fill_rect_clipped(resolved.outer_rect, resolved.background);
    ui.stroke_rect_clipped(resolved.outer_rect, 1, resolved.outline);
    draw_container_frame_title(ui, resolved);
}

/// Draw frame title text and track its footprint when present.
fn draw_container_frame_title(ui: &mut Ui<'_>, resolved: ContainerFrameResolved<'_>) {
    let Some(title) = resolved.title else {
        return;
    };
    let title_pos = Point {
        x: resolved.origin.x + resolved.padding,
        y: resolved.origin.y + resolved.padding,
    };
    ui.draw_text_internal(title_pos, title, ui.theme.text, ui.theme.text_scale);
    ui.track_rect_internal(Rect {
        origin: title_pos,
        size: text_size(title, ui.theme.text_scale),
    });
}

/// Measure children rendered inside the frame content rect.
fn measure_container_frame_content<F>(
    ui: &mut Ui<'_>,
    resolved: &ContainerFrameResolved<'_>,
    mut f: F,
) -> Option<Rect>
where
    F: FnMut(&mut Ui<'_>, Rect),
{
    ui.push_bounds();
    ui.with_layout(resolved.content_origin, |ui| f(ui, resolved.content_rect));
    ui.pop_bounds()
}

/// Compute final measured size according to explicit-size policy.
fn finalize_container_frame_size(
    resolved: &ContainerFrameResolved<'_>,
    measured_bounds: Option<Rect>,
    explicit_size_policy: ExplicitSizePolicy,
) -> Size {
    let measured_content = measured_container_content_size(resolved.content_origin, measured_bounds);
    let measured_size = Size {
        width: measured_content.width + (resolved.padding * 2) as u32,
        height: measured_content.height + (resolved.padding * 2 + resolved.header_height) as u32,
    };
    match (explicit_size_policy, resolved.requested_size) {
        (ExplicitSizePolicy::PreserveExplicitMinimum, Some(explicit)) => Size {
            width: explicit.width.max(measured_size.width),
            height: explicit.height.max(measured_size.height),
        },
        (ExplicitSizePolicy::PreferExplicit, Some(explicit)) => explicit,
        (_, None) => measured_size,
    }
}

/// Convert measured bounds into content-space size.
fn measured_container_content_size(content_origin: Point, measured_bounds: Option<Rect>) -> Size {
    let Some(bounds) = measured_bounds else {
        return Size {
            width: 0,
            height: 0,
        };
    };
    let max_x = bounds.origin.x + bounds.size.width as i32;
    let max_y = bounds.origin.y + bounds.size.height as i32;
    Size {
        width: (max_x - content_origin.x).max(0) as u32,
        height: (max_y - content_origin.y).max(0) as u32,
    }
}

/// Persist measured size and apply container-specific post effects.
fn persist_container_frame_state(
    ui: &mut Ui<'_>,
    resolved: &ContainerFrameResolved<'_>,
    measured_size: Size,
    effects: ContainerFrameEffects,
) {
    ui.state.layout.set(resolved.id, measured_size);
    ui.track_rect_internal(resolved.outer_rect);
    if effects.advance_layout_cursor {
        ui.layout.cursor.y = resolved.origin.y + measured_size.height as i32 + ui.layout.spacing;
    }
    if effects.update_root_frame_size {
        ui.state.set_root_frame_size(measured_size);
    }
}
