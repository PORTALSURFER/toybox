/// Declarative container node kinds that can emit debug layout borders.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ContainerKind {
    /// Root frame wrapper that owns the full declarative content tree.
    RootFrame,
    /// Panel container.
    Panel,
    /// Flex layout container.
    Flex,
    /// Grid layout container.
    Grid,
    /// Absolute-positioned container.
    Absolute,
    /// Stack container.
    Stack,
    /// Scroll-view container.
    ScrollView,
    /// Wrap container.
    Wrap,
    /// Responsive switch-layout container.
    SwitchLayout,
}

/// Candidate border outline emitted while traversing container nodes.
#[derive(Clone, Copy, Debug, PartialEq)]
struct DebugBorderCandidate {
    /// Surface-space rectangle considered for debug border drawing.
    rect: Rect,
    /// Container category used for border color resolution.
    kind: ContainerKind,
    /// Nesting depth in the declarative container tree.
    depth: usize,
}

/// Return the optional debug border color for a container kind/depth pair.
fn container_debug_border_color(kind: ContainerKind, depth: usize) -> Option<Color> {
    #[cfg(feature = "layout-debug-borders")]
    {
        let _ = depth;
        match kind {
            ContainerKind::RootFrame => None,
            ContainerKind::Panel
            | ContainerKind::Flex
            | ContainerKind::Grid
            | ContainerKind::Absolute
            | ContainerKind::Stack
            | ContainerKind::ScrollView
            | ContainerKind::Wrap
            | ContainerKind::SwitchLayout => Some(Color::rgb(245, 98, 98)),
        }
    }
    #[cfg(not(feature = "layout-debug-borders"))]
    {
        let _ = (kind, depth);
        None
    }
}

/// Record a hovered layout-debug border candidate for later selection.
fn collect_container_debug_border_candidate(
    candidates: &mut Vec<DebugBorderCandidate>,
    ui: &Ui<'_>,
    rect: Rect,
    kind: ContainerKind,
    depth: usize,
) {
    // Skip root-level wrappers so debug outlines focus on meaningful inner
    // layout partitions instead of the full-window container.
    if !should_draw_container_debug_border(kind, depth, rect.contains(ui.input().pointer_pos)) {
        return;
    }
    if container_debug_border_color(kind, depth).is_none() {
        return;
    }
    candidates.push(DebugBorderCandidate { rect, kind, depth });
}
