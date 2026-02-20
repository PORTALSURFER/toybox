/// Pick exactly one container debug-border target for this frame.
///
/// Selection order:
/// 1. deepest container depth
/// 2. smallest area (more specific subsection)
/// 3. latest rendered candidate (stable tie-breaker)
fn select_container_debug_border_candidate(
    candidates: &[DebugBorderCandidate],
) -> Option<DebugBorderCandidate> {
    let mut best_index: Option<usize> = None;
    for (index, candidate) in candidates.iter().copied().enumerate() {
        match best_index {
            None => best_index = Some(index),
            Some(current_index) => {
                let current = candidates[current_index];
                let better_depth = candidate.depth > current.depth;
                let same_depth = candidate.depth == current.depth;
                let smaller_area = candidate_area(candidate) < candidate_area(current);
                let same_area = candidate_area(candidate) == candidate_area(current);
                let later_render = index > current_index;
                if better_depth || (same_depth && (smaller_area || (same_area && later_render))) {
                    best_index = Some(index);
                }
            }
        }
    }
    best_index.map(|index| candidates[index])
}

/// Compute a comparable area metric for candidate ranking.
fn candidate_area(candidate: DebugBorderCandidate) -> u64 {
    u64::from(candidate.rect.size.width) * u64::from(candidate.rect.size.height)
}

/// Return whether a candidate should be collected for the current debug mode.
fn should_collect_container_debug_border_candidate(
    kind: ContainerKind,
    depth: usize,
    pointer_inside: bool,
) -> bool {
    #[cfg(feature = "layout-debug-borders")]
    {
        return should_collect_container_debug_border_candidate_with_mode(
            kind,
            depth,
            pointer_inside,
            should_draw_all_layout_debug_borders(),
        );
    }
    #[cfg(not(feature = "layout-debug-borders"))]
    {
        should_collect_container_debug_border_candidate_with_mode(kind, depth, pointer_inside, false)
    }
}

/// Return whether a candidate should be collected under one explicit mode.
fn should_collect_container_debug_border_candidate_with_mode(
    kind: ContainerKind,
    depth: usize,
    pointer_inside: bool,
    draw_all: bool,
) -> bool {
    kind != ContainerKind::RootFrame && depth > 1 && (draw_all || pointer_inside)
}

#[cfg(feature = "layout-debug-borders")]
fn should_draw_all_layout_debug_borders() -> bool {
    should_draw_all_layout_debug_borders_from_env(std::env::var("PATCHBAY_DEBUG_ALL_LAYOUT_BORDERS"))
}

#[cfg(feature = "layout-debug-borders")]
fn should_draw_all_layout_debug_borders_from_env(
    env_value: Result<String, std::env::VarError>,
) -> bool {
    matches!(env_value.ok().as_deref(), Some("1" | "true" | "True" | "TRUE" | "yes" | "Yes" | "YES" | "on" | "On" | "ON"))
}

/// Return the exact debug border rectangle for one node candidate.
///
/// Debug overlays should map 1:1 to resolved slot/container rectangles so
/// shared edges between adjacent slots do not appear inset.
fn debug_border_draw_rect(rect: Rect, thickness: u32) -> Option<Rect> {
    if thickness == 0 || rect.size.width == 0 || rect.size.height == 0 {
        return None;
    }
    Some(rect)
}
