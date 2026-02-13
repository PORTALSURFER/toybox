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
