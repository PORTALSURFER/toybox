
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

/// Return whether a hovered container should render the debug layout border.
fn should_draw_container_debug_border(
    kind: ContainerKind,
    depth: usize,
    pointer_inside: bool,
) -> bool {
    kind != ContainerKind::RootFrame && depth > 1 && pointer_inside
}

/// Return a pixel-safe debug border rectangle that stays inside viewport bounds.
///
/// The debug stroke helper can lose bottom/right edges when a container reaches
/// the viewport edge. Shrinking the border box by one stroke thickness on the
/// max edges keeps all four lines visible.
fn debug_border_draw_rect(rect: Rect, thickness: u32) -> Option<Rect> {
    if thickness == 0 || rect.size.width <= thickness || rect.size.height <= thickness {
        return None;
    }
    Some(Rect {
        origin: rect.origin,
        size: Size {
            width: rect.size.width.saturating_sub(thickness),
            height: rect.size.height.saturating_sub(thickness),
        },
    })
}

/// Validate the top-level UI specification.
fn validate_spec(spec: &UiSpec) -> Result<(), DeclarativeError> {
    if spec.root.key.trim().is_empty() {
        return Err(DeclarativeError::EmptyNodeKey {
            node_kind: "RootFrame",
        });
    }
    if !is_container_node(&spec.root.content) {
        return Err(DeclarativeError::InvalidRootContent {
            node_kind: node_kind_name(&spec.root.content),
        });
    }
    let mut seen = std::collections::HashSet::new();
    validate_unique_key(&spec.root.key, &mut seen)?;
    validate_node(&spec.root.content, &mut seen)
}

/// Validate a node subtree.
fn validate_node(
    node: &Node,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), DeclarativeError> {
    match node {
        Node::Panel(panel) => validate_panel_node(panel, seen_keys)?,
        Node::Row(flex) | Node::Column(flex) => validate_flex_children(&flex.children, seen_keys)?,
        Node::Grid(grid) => validate_grid_node(grid, seen_keys)?,
        Node::Absolute(absolute) => validate_absolute_node(absolute, seen_keys)?,
        Node::Label(_) | Node::Spacer(_) | Node::Indicator(_) => {}
        Node::Knob(knob) => validate_knob_node(knob, seen_keys)?,
        Node::Slider(slider) => validate_slider_node(slider, seen_keys)?,
        Node::Toggle(toggle) => validate_toggle_node(toggle, seen_keys)?,
        Node::Button(button) => validate_button_node(button, seen_keys)?,
        Node::Dropdown(dropdown) => validate_dropdown_node(dropdown, seen_keys)?,
        Node::Region(region) => validate_region_node(region, seen_keys)?,
    }
    Ok(())
}

/// Validate panel key constraints and recurse into panel content.
fn validate_panel_node(
    panel: &PanelSpec,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), DeclarativeError> {
    validate_non_empty_key(&panel.key, "Panel")?;
    validate_unique_key(&panel.key, seen_keys)?;
    validate_node(&panel.content, seen_keys)
}

/// Validate a flat list of child nodes.
fn validate_flex_children(
    children: &[Node],
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), DeclarativeError> {
    for child in children {
        validate_node(child, seen_keys)?;
    }
    Ok(())
}

/// Validate a grid node and recurse through children.
fn validate_grid_node(
    grid: &GridSpec,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), DeclarativeError> {
    if grid.template.columns.is_empty() {
        return Err(DeclarativeError::EmptyGridColumns);
    }
    validate_section_grid_children(grid)?;
    validate_flex_children(&grid.children, seen_keys)
}

/// Validate section-grid-only track and child-container constraints.
fn validate_section_grid_children(grid: &GridSpec) -> Result<(), DeclarativeError> {
    if !matches!(grid.kind, GridKind::SectionColumn | GridKind::SectionRow) {
        return Ok(());
    }

    validate_section_tracks(grid)?;
    for child in &grid.children {
        if !is_container_node(child) {
            return Err(DeclarativeError::InvalidSectionChild {
                node_kind: node_kind_name(child),
            });
        }
    }
    Ok(())
}

/// Validate absolute-positioned children recursively.
fn validate_absolute_node(
    absolute: &AbsoluteSpec,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), DeclarativeError> {
    for child in &absolute.children {
        validate_node(&child.node, seen_keys)?;
    }
    Ok(())
}

/// Validate knob constraints.
fn validate_knob_node(
    knob: &KnobSpec,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), DeclarativeError> {
    validate_non_empty_key(&knob.key, "Knob")?;
    validate_unique_key(&knob.key, seen_keys)?;
    validate_value_range("Knob", &knob.key, knob.range)?;
    validate_control_value("Knob", &knob.key, knob.value, knob.range)
}

/// Validate slider constraints.
fn validate_slider_node(
    slider: &SliderSpec,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), DeclarativeError> {
    validate_non_empty_key(&slider.key, "Slider")?;
    validate_unique_key(&slider.key, seen_keys)?;
    validate_value_range("Slider", &slider.key, slider.range)?;
    validate_control_value("Slider", &slider.key, slider.value, slider.range)?;
    validate_optional_control_size("Slider", &slider.key, slider.control_size)
}

/// Validate toggle constraints.
fn validate_toggle_node(
    toggle: &ToggleSpec,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), DeclarativeError> {
    validate_non_empty_key(&toggle.key, "Toggle")?;
    validate_unique_key(&toggle.key, seen_keys)?;
    validate_optional_control_size("Toggle", &toggle.key, toggle.control_size)
}

/// Validate button constraints.
fn validate_button_node(
    button: &ButtonSpec,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), DeclarativeError> {
    validate_non_empty_key(&button.key, "Button")?;
    validate_unique_key(&button.key, seen_keys)?;
    validate_optional_control_size("Button", &button.key, button.control_size)
}

/// Validate dropdown constraints.
fn validate_dropdown_node(
    dropdown: &DropdownSpec,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), DeclarativeError> {
    validate_non_empty_key(&dropdown.key, "Dropdown")?;
    validate_unique_key(&dropdown.key, seen_keys)?;
    validate_dropdown_selection(dropdown)?;
    validate_optional_control_size("Dropdown", &dropdown.key, dropdown.control_size)
}

/// Validate region constraints.
fn validate_region_node(
    region: &RegionSpec,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), DeclarativeError> {
    validate_non_empty_key(&region.key, "Region")?;
    validate_unique_key(&region.key, seen_keys)
}

/// Validate an optional control size override when supplied.
fn validate_optional_control_size(
    node_kind: &'static str,
    key: &str,
    control_size: Option<Size>,
) -> Result<(), DeclarativeError> {
    if let Some(control_size) = control_size {
        validate_control_size(node_kind, key, control_size)?;
    }
    Ok(())
}

/// Return true when a node acts as a layout container.
fn is_container_node(node: &Node) -> bool {
    matches!(
        node,
        Node::Panel(_) | Node::Row(_) | Node::Column(_) | Node::Grid(_) | Node::Absolute(_)
    )
}

/// Return a stable node-kind name for diagnostics.
fn node_kind_name(node: &Node) -> &'static str {
    match node {
        Node::Panel(_) => "Panel",
        Node::Row(_) => "Row",
        Node::Column(_) => "Column",
        Node::Grid(_) => "Grid",
        Node::Absolute(_) => "Absolute",
        Node::Label(_) => "Label",
        Node::Spacer(_) => "Spacer",
        Node::Knob(_) => "Knob",
        Node::Slider(_) => "Slider",
        Node::Toggle(_) => "Toggle",
        Node::Button(_) => "Button",
        Node::Dropdown(_) => "Dropdown",
        Node::Region(_) => "Region",
        Node::Indicator(_) => "Indicator",
    }
}

/// Validate strict section-track definitions for canonical section grids.
fn validate_section_tracks(grid: &GridSpec) -> Result<(), DeclarativeError> {
    let tracks = match grid.kind {
        GridKind::SectionColumn => &grid.template.rows,
        GridKind::SectionRow => &grid.template.columns,
        GridKind::Standard => return Ok(()),
    };
    if tracks.is_empty() {
        return Err(DeclarativeError::InvalidSectionTrack);
    }

    let mut total_percent = 0u16;
    let mut fill_count = 0usize;
    for track in tracks {
        match *track {
            TrackSize::Percent(percent) => {
                total_percent = total_percent.saturating_add(percent as u16);
            }
            TrackSize::Fill => fill_count = fill_count.saturating_add(1),
            _ => return Err(DeclarativeError::InvalidSectionTrack),
        }
    }
    if total_percent > 100 || (fill_count == 0 && total_percent != 100) {
        return Err(DeclarativeError::InvalidSectionFractions {
            total_percent,
            fill_count,
        });
    }
    Ok(())
}

/// Validate that a key is non-empty.
fn validate_non_empty_key(key: &str, node_kind: &'static str) -> Result<(), DeclarativeError> {
    if key.trim().is_empty() {
        return Err(DeclarativeError::EmptyNodeKey { node_kind });
    }
    Ok(())
}

/// Validate key uniqueness.
fn validate_unique_key(
    key: &str,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), DeclarativeError> {
    if !seen_keys.insert(key.to_string()) {
        return Err(DeclarativeError::DuplicateNodeKey {
            key: key.to_string(),
        });
    }
    Ok(())
}

/// Validate a numeric value range.
fn validate_value_range(
    node_kind: &'static str,
    key: &str,
    range: (f32, f32),
) -> Result<(), DeclarativeError> {
    let (min, max) = range;
    if !min.is_finite() || !max.is_finite() || min >= max {
        return Err(DeclarativeError::InvalidValueRange {
            node_kind,
            key: key.to_string(),
        });
    }
    Ok(())
}

/// Validate an explicit control size override.
fn validate_control_size(
    node_kind: &'static str,
    key: &str,
    control_size: Size,
) -> Result<(), DeclarativeError> {
    if control_size.width == 0 || control_size.height == 0 {
        return Err(DeclarativeError::InvalidControlSize {
            node_kind,
            key: key.to_string(),
            width: control_size.width,
            height: control_size.height,
        });
    }
    Ok(())
}

/// Validate that dropdown selection references an existing option.
fn validate_dropdown_selection(dropdown: &DropdownSpec) -> Result<(), DeclarativeError> {
    if dropdown.selected >= dropdown.options.len() {
        return Err(DeclarativeError::InvalidDropdownSelection {
            key: dropdown.key.clone(),
            selected: dropdown.selected,
            options_len: dropdown.options.len(),
        });
    }
    Ok(())
}

/// Validate control value finiteness and in-range constraints.
fn validate_control_value(
    node_kind: &'static str,
    key: &str,
    value: f32,
    range: (f32, f32),
) -> Result<(), DeclarativeError> {
    let (min, max) = range;
    if !value.is_finite() || value < min || value > max {
        return Err(DeclarativeError::InvalidControlValue {
            node_kind,
            key: key.to_string(),
            value,
            min,
            max,
        });
    }
    Ok(())
}

/// Measure root frame size including header and padding.
fn measure_root_frame(frame: &RootFrameSpec, tokens: &ThemeTokens) -> Size {
    let content = measure_node(&frame.content, tokens);
    let header = panel_header_height(frame.title.as_deref(), tokens).max(0) as u32;
    let padding = frame.padding.max(0) as u32;
    let measured = Size {
        width: content.width + padding * 2,
        height: content.height + padding * 2 + header,
    };
    resolve_size(frame.layout, measured, measured)
}
