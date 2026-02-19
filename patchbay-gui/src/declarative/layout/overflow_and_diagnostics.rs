/// Resolve a child rectangle with the selected overflow policy.
fn overflow_rect_with_policy(
    rect: Rect,
    bounds: Rect,
    overflow_policy: OverflowPolicy,
    container_kind: ContainerKind,
    layout_diagnostics: &mut Vec<LayoutDiagnostic>,
) -> Option<Rect> {
    match overflow_policy {
        OverflowPolicy::Clip => {
            let clipped = clip_rect_to_bounds(rect, bounds);
            if let Some(clipped_rect) = clipped {
                if clipped_rect != rect {
                    record_layout_diagnostic(
                        layout_diagnostics,
                        container_kind,
                        LayoutDiagnosticCode::OverflowClipped,
                        "layout rect clipped to container bounds",
                        rect,
                        bounds,
                    );
                }
            } else {
                record_layout_diagnostic(
                    layout_diagnostics,
                    container_kind,
                    LayoutDiagnosticCode::OverflowSkippedDisjoint,
                    "layout rect does not intersect container bounds",
                    rect,
                    bounds,
                );
            }
            clipped
        }
        OverflowPolicy::Compress => {
            let compressed = compress_rect_to_bounds(rect, bounds);
            if let Some(compressed_rect) = compressed {
                if compressed_rect != rect {
                    record_layout_diagnostic(
                        layout_diagnostics,
                        container_kind,
                        LayoutDiagnosticCode::OverflowCompressed,
                        "layout rect compressed to fit container bounds",
                        rect,
                        bounds,
                    );
                }
            } else {
                record_layout_diagnostic(
                    layout_diagnostics,
                    container_kind,
                    LayoutDiagnosticCode::OverflowSkippedCollapsedBounds,
                    "container bounds collapsed; child skipped",
                    rect,
                    bounds,
                );
            }
            compressed
        }
    }
}

/// Compress a rectangle so it fully fits inside container bounds.
fn compress_rect_to_bounds(rect: Rect, bounds: Rect) -> Option<Rect> {
    if bounds.size.width == 0 || bounds.size.height == 0 {
        return None;
    }
    let width = rect.size.width.min(bounds.size.width);
    let height = rect.size.height.min(bounds.size.height);
    if width == 0 || height == 0 {
        return None;
    }

    let max_x = bounds
        .origin
        .x
        .saturating_add(bounds.size.width as i32)
        .saturating_sub(width as i32);
    let max_y = bounds
        .origin
        .y
        .saturating_add(bounds.size.height as i32)
        .saturating_sub(height as i32);

    Some(Rect {
        origin: Point {
            x: rect.origin.x.clamp(bounds.origin.x, max_x),
            y: rect.origin.y.clamp(bounds.origin.y, max_y),
        },
        size: Size { width, height },
    })
}

/// Record one runtime layout diagnostic.
fn record_layout_diagnostic(
    diagnostics: &mut Vec<LayoutDiagnostic>,
    container_kind: ContainerKind,
    code: LayoutDiagnosticCode,
    message: &'static str,
    requested_rect: Rect,
    bounds: Rect,
) {
    diagnostics.push(LayoutDiagnostic {
        level: LayoutDiagnosticLevel::Warning,
        code,
        container: layout_container_kind(container_kind),
        message,
        requested_rect,
        bounds,
    });
}

/// Convert internal debug container kind into public diagnostic kind.
fn layout_container_kind(kind: ContainerKind) -> LayoutContainerKind {
    match kind {
        ContainerKind::RootFrame => LayoutContainerKind::RootFrame,
        ContainerKind::Panel => LayoutContainerKind::Panel,
        ContainerKind::PaddingBox => LayoutContainerKind::PaddingBox,
        ContainerKind::AlignBox => LayoutContainerKind::AlignBox,
        ContainerKind::AspectBox => LayoutContainerKind::AspectBox,
        ContainerKind::Flex => LayoutContainerKind::Flex,
        ContainerKind::Grid => LayoutContainerKind::Grid,
        ContainerKind::Absolute => LayoutContainerKind::Absolute,
        ContainerKind::Stack => LayoutContainerKind::Stack,
        ContainerKind::ScrollView => LayoutContainerKind::ScrollView,
        ContainerKind::Wrap => LayoutContainerKind::Wrap,
        ContainerKind::SwitchLayout => LayoutContainerKind::SwitchLayout,
    }
}
