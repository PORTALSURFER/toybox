fn client_size_changed(current: Size, next: Size) -> bool {
    current != next
}

fn resolved_layout_size_for_resize_request(
    requested: Size,
    host_client: Option<(u32, u32)>,
    configured_aspect_ratio: Option<f32>,
) -> Size {
    debug_assert!(requested.width > 0 && requested.height > 0);
    debug_assert!(matches!(
        configured_aspect_ratio,
        Some(ratio) if ratio.is_finite() && ratio > 0.0
    ) || configured_aspect_ratio.is_none());

    let mut resolved = host_client
        .map(|(width, height)| Size {
            width: width.max(1),
            height: height.max(1),
        })
        .unwrap_or(Size {
            width: requested.width.max(1),
            height: requested.height.max(1),
        });

    if let Some(ratio) = configured_aspect_ratio {
        if ratio.is_finite() && ratio > 0.0 {
            let (width, height) = enforce_aspect_min(resolved.width, resolved.height, ratio);
            resolved = Size { width, height };
        }
    }
    debug_assert!(resolved.width > 0 && resolved.height > 0);

    resolved
}

fn enforce_aspect_min(width: u32, height: u32, aspect: f32) -> (u32, u32) {
    debug_assert!(width > 0 && height > 0);
    if !aspect.is_finite() || aspect <= 0.0 {
        return (width.max(1), height.max(1));
    }
    let width_from_height = ceil_to_u32((height as f32 * aspect).max(1.0));
    let height_from_width = ceil_to_u32((width as f32 / aspect).max(1.0));
    if width_from_height >= width {
        (width_from_height, height.max(1))
    } else {
        (width.max(1), height_from_width)
    }
}

fn ceil_to_u32(raw: f32) -> u32 {
    debug_assert!(raw.is_finite());
    raw.ceil().max(1.0).min(u32::MAX as f32) as u32
}
