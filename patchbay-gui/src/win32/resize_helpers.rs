fn client_size_changed(current: Size, next: Size) -> bool {
    current != next
}

fn resolved_layout_size_for_resize_request(
    requested: Size,
    _host_client: Option<(u32, u32)>,
) -> Size {
    Size {
        width: requested.width.max(1),
        height: requested.height.max(1),
    }
}

#[cfg(test)]
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
