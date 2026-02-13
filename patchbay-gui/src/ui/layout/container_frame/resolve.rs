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
fn resolve_container_header_height(
    title: Option<&str>,
    explicit: Option<i32>,
    text_scale: i32,
) -> i32 {
    explicit.unwrap_or_else(|| {
        if title.is_some() {
            (8 * text_scale + 4).max(0)
        } else {
            0
        }
    })
}

/// Fallback size used when neither explicit nor cached size is available.
pub(super) fn fallback_container_size(padding: i32, header_height: i32) -> Size {
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
