fn client_size_changed(current: Size, next: Size) -> bool {
    current != next
}

fn resolved_layout_size_for_resize_request(
    requested: Size,
    host_client: Option<(u32, u32)>,
    configured_aspect_ratio: Option<f32>,
) -> Size {
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

    resolved
}

fn enforce_aspect_min(width: u32, height: u32, aspect: f32) -> (u32, u32) {
    if !aspect.is_finite() || aspect <= 0.0 {
        return (width.max(1), height.max(1));
    }
    let width_from_height = (height as f32 * aspect).ceil().max(1.0) as u32;
    let height_from_width = (width as f32 / aspect).ceil().max(1.0) as u32;
    if width_from_height >= width {
        (width_from_height, height.max(1))
    } else {
        (width.max(1), height_from_width)
    }
}
