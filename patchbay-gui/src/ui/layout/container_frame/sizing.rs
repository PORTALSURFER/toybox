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
