/// Hard depth cap used to fail fast before recursive measure/render traversal.
const MAX_DECLARATIVE_TREE_DEPTH: usize = 700;
/// Validate the top-level UI specification.
fn validate_spec(spec: &UiSpec) -> Result<(), DeclarativeError> {
    if spec.root.key.trim().is_empty() {
        return Err(DeclarativeError::EmptyNodeKey {
            node_kind: "RootFrame",
        });
    }
    validate_layout_bounds("RootFrame", spec.root.layout)?;
    let mut seen = std::collections::HashSet::new();
    validate_unique_key(&spec.root.key, &mut seen)?;
    validate_root_slot(&spec.root.content, &mut seen)
}

/// Validate the root slot contract.
fn validate_root_slot(
    root_content: &Node,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), DeclarativeError> {
    let Node::Slot(slot) = root_content else {
        return Err(DeclarativeError::InvalidRootContent {
            node_kind: node_kind_name(root_content),
        });
    };
    if !is_container_node(&slot.child) {
        return Err(DeclarativeError::InvalidRootSlotChild {
            node_kind: node_kind_name(&slot.child),
        });
    }
    validate_tree_depth_limit(&slot.child, MAX_DECLARATIVE_TREE_DEPTH)?;
    validate_node(&slot.child, seen_keys)
}

/// Validate maximum declarative tree depth with iterative traversal.
fn validate_tree_depth_limit(root: &Node, max_depth: usize) -> Result<(), DeclarativeError> {
    let mut stack = vec![(root, 1usize)];
    while let Some((node, depth)) = stack.pop() {
        if depth > max_depth {
            return Err(DeclarativeError::TreeDepthExceeded {
                max_depth,
                actual_depth: depth,
                node_kind: node_kind_name(node),
            });
        }
        let next_depth = depth.saturating_add(1);
        node.for_each_child(|child| stack.push((child, next_depth)));
    }
    Ok(())
}
/// Validate a node subtree.
fn validate_node(
    node: &Node,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), DeclarativeError> {
    match node {
        Node::Slot(slot) => validate_slot_node(slot, seen_keys),
        Node::Panel(panel) => validate_panel_node(panel, seen_keys),
        Node::PaddingBox(padding_box) => {
            validate_container_layout("PaddingBox", padding_box.layout.to_layout_box())?;
            validate_node(padding_box.content(), seen_keys)
        }
        Node::AlignBox(align_box) => {
            validate_container_layout("AlignBox", align_box.layout.to_layout_box())?;
            validate_node(align_box.content(), seen_keys)
        }
        Node::AspectBox(aspect_box) => {
            validate_container_layout("AspectBox", aspect_box.layout.to_layout_box())?;
            validate_aspect_ratio(aspect_box.aspect_ratio)?;
            validate_node(aspect_box.content(), seen_keys)
        }
        Node::Row(flex) => {
            validate_container_layout("Row", flex.layout.to_layout_box())?;
            validate_container_children("Row", &flex.children, seen_keys)
        }
        Node::Column(flex) => {
            validate_container_layout("Column", flex.layout.to_layout_box())?;
            validate_container_children("Column", &flex.children, seen_keys)
        }
        Node::Grid(grid) => validate_grid_node(grid, seen_keys),
        Node::Absolute(absolute) => validate_absolute_node(absolute, seen_keys),
        Node::Stack(stack) => {
            validate_container_layout("Stack", stack.layout.to_layout_box())?;
            validate_container_children("Stack", &stack.children, seen_keys)
        }
        Node::ScrollView(scroll_view) => {
            validate_container_layout("ScrollView", scroll_view.layout.to_layout_box())?;
            validate_node(scroll_view.content(), seen_keys)
        }
        Node::Wrap(wrap) => {
            validate_container_layout("Wrap", wrap.layout.to_layout_box())?;
            validate_container_children("Wrap", &wrap.children, seen_keys)
        }
        Node::SwitchLayout(switch_layout) => validate_switch_layout_node(switch_layout, seen_keys),
        Node::TextBox(text_box) => validate_text_box_node(text_box, seen_keys),
        Node::Spacer(spacer) => validate_spacer_node(spacer),
        Node::Knob(knob) => validate_knob_node(knob, seen_keys),
        Node::Slider(slider) => validate_slider_node(slider, seen_keys),
        Node::Toggle(toggle) => validate_toggle_node(toggle, seen_keys),
        Node::Button(button) => validate_button_node(button, seen_keys),
        Node::Dropdown(dropdown) => validate_dropdown_node(dropdown, seen_keys),
        Node::Region(region) => validate_region_node(region, seen_keys),
        Node::Indicator(indicator) => validate_indicator_node(indicator),
    }
}

/// Validate text-box constraints and optional editable contract.
fn validate_text_box_node(
    text_box: &TextBoxSpec,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), DeclarativeError> {
    validate_layout_bounds("TextBox", text_box.layout)?;
    if let Some(edit) = text_box.edit.as_ref() {
        validate_non_empty_key(&edit.key, "TextBox")?;
        validate_unique_key(&edit.key, seen_keys)?;
    }
    Ok(())
}

/// Validate panel key constraints and recurse into panel slot content.
fn validate_panel_node(
    panel: &PanelSpec,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), DeclarativeError> {
    validate_container_layout("Panel", panel.layout.to_layout_box())?;
    validate_non_empty_key(&panel.key, "Panel")?;
    validate_unique_key(&panel.key, seen_keys)?;
    validate_node(&panel.content, seen_keys)
}

/// Validate slot child constraints and recurse.
fn validate_slot_node(
    slot: &SlotSpec,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), DeclarativeError> {
    if matches!(slot.child.as_ref(), Node::Slot(_)) {
        return Err(DeclarativeError::InvalidSlotChild {
            node_kind: "Slot",
        });
    }
    if !(is_container_node(&slot.child) || is_widget_node(&slot.child)) {
        return Err(DeclarativeError::InvalidSlotChild {
            node_kind: node_kind_name(&slot.child),
        });
    }
    validate_node(&slot.child, seen_keys)
}

/// Validate that a container's direct children are slot nodes.
fn validate_container_children(
    container_kind: &'static str,
    children: &[Node],
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), DeclarativeError> {
    for child in children {
        if !matches!(child, Node::Slot(_)) {
            return Err(DeclarativeError::InvalidContainerChild {
                container_kind,
                node_kind: node_kind_name(child),
            });
        }
        validate_node(child, seen_keys)?;
    }
    Ok(())
}

/// Validate a grid node and recurse through children.
fn validate_grid_node(
    grid: &GridSpec,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), DeclarativeError> {
    validate_container_layout("Grid", grid.layout.to_layout_box())?;
    validate_no_fixed_grid_tracks(grid)?;
    if grid.template.columns.is_empty() {
        return Err(DeclarativeError::EmptyGridColumns);
    }
    if matches!(grid.kind, GridKind::SlotColumn | GridKind::SlotRow) {
        validate_slot_tracks(grid)?;
    }
    validate_container_children("Grid", &grid.children, seen_keys)
}

/// Validate absolute-positioned children recursively.
fn validate_absolute_node(
    absolute: &AbsoluteSpec,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), DeclarativeError> {
    validate_container_layout("Absolute", absolute.layout.to_layout_box())?;
    for child in &absolute.children {
        if !matches!(child.node, Node::Slot(_)) {
            return Err(DeclarativeError::InvalidContainerChild {
                container_kind: "Absolute",
                node_kind: node_kind_name(&child.node),
            });
        }
        validate_node(&child.node, seen_keys)?;
    }
    Ok(())
}

/// Validate switch-layout cases, fallback, and child slot structure.
fn validate_switch_layout_node(
    switch_layout: &SwitchLayoutSpec,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), DeclarativeError> {
    validate_container_layout("SwitchLayout", switch_layout.layout.to_layout_box())?;
    validate_switch_case_ranges(switch_layout.cases())?;
    for case_entry in switch_layout.cases() {
        if !matches!(case_entry.child(), Node::Slot(_)) {
            return Err(DeclarativeError::InvalidContainerChild {
                container_kind: "SwitchLayout",
                node_kind: node_kind_name(case_entry.child()),
            });
        }
        validate_node(case_entry.child(), seen_keys)?;
    }
    if !matches!(switch_layout.fallback(), Node::Slot(_)) {
        return Err(DeclarativeError::InvalidContainerChild {
            container_kind: "SwitchLayout",
            node_kind: node_kind_name(switch_layout.fallback()),
        });
    }
    validate_node(switch_layout.fallback(), seen_keys)
}

/// Validate knob constraints.
fn validate_knob_node(
    knob: &KnobSpec,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), DeclarativeError> {
    validate_non_empty_key(&knob.key, "Knob")?;
    validate_unique_key(&knob.key, seen_keys)?;
    validate_value_range("Knob", &knob.key, knob.range)?;
    validate_layout_bounds("Knob", knob.layout)?;
    validate_control_value("Knob", &knob.key, knob.value, knob.range)?;
    validate_optional_control_size("Knob", &knob.key, knob.control_size)
}

/// Validate slider constraints.
fn validate_slider_node(
    slider: &SliderSpec,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), DeclarativeError> {
    validate_non_empty_key(&slider.key, "Slider")?;
    validate_unique_key(&slider.key, seen_keys)?;
    validate_value_range("Slider", &slider.key, slider.range)?;
    validate_layout_bounds("Slider", slider.layout)?;
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
    validate_layout_bounds("Toggle", toggle.layout)?;
    validate_optional_control_size("Toggle", &toggle.key, toggle.control_size)
}

/// Validate button constraints.
fn validate_button_node(
    button: &ButtonSpec,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), DeclarativeError> {
    validate_non_empty_key(&button.key, "Button")?;
    validate_unique_key(&button.key, seen_keys)?;
    validate_layout_bounds("Button", button.layout)?;
    validate_optional_control_size("Button", &button.key, button.control_size)
}

/// Validate dropdown constraints.
fn validate_dropdown_node(
    dropdown: &DropdownSpec,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), DeclarativeError> {
    validate_non_empty_key(&dropdown.key, "Dropdown")?;
    validate_unique_key(&dropdown.key, seen_keys)?;
    validate_layout_bounds("Dropdown", dropdown.layout)?;
    validate_dropdown_selection(dropdown)?;
    validate_optional_control_size("Dropdown", &dropdown.key, dropdown.control_size)
}

/// Validate region constraints.
fn validate_region_node(
    region: &RegionSpec,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), DeclarativeError> {
    validate_non_empty_key(&region.key, "Region")?;
    validate_unique_key(&region.key, seen_keys)?;
    validate_layout_bounds("Region", region.layout)
}

/// Validate spacer constraints.
fn validate_spacer_node(spacer: &SpacerSpec) -> Result<(), DeclarativeError> {
    validate_layout_bounds("Spacer", spacer.layout)
}

/// Validate indicator constraints.
fn validate_indicator_node(indicator: &IndicatorSpec) -> Result<(), DeclarativeError> {
    validate_layout_bounds("Indicator", indicator.layout)
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
        Node::Panel(_)
            | Node::PaddingBox(_)
            | Node::AlignBox(_)
            | Node::AspectBox(_)
            | Node::Row(_)
            | Node::Column(_)
            | Node::Grid(_)
            | Node::Absolute(_)
            | Node::Stack(_)
            | Node::ScrollView(_)
            | Node::Wrap(_)
            | Node::SwitchLayout(_)
    )
}

/// Return true when a node acts as a widget leaf.
fn is_widget_node(node: &Node) -> bool {
    matches!(
        node,
        Node::TextBox(_)
            | Node::Spacer(_)
            | Node::Knob(_)
            | Node::Slider(_)
            | Node::Toggle(_)
            | Node::Button(_)
            | Node::Dropdown(_)
            | Node::Region(_)
            | Node::Indicator(_)
    )
}

/// Return a stable node-kind name for diagnostics.
fn node_kind_name(node: &Node) -> &'static str {
    match node {
        Node::Slot(_) => "Slot",
        Node::Panel(_) => "Panel",
        Node::PaddingBox(_) => "PaddingBox",
        Node::AlignBox(_) => "AlignBox",
        Node::AspectBox(_) => "AspectBox",
        Node::Row(_) => "Row",
        Node::Column(_) => "Column",
        Node::Grid(_) => "Grid",
        Node::Absolute(_) => "Absolute",
        Node::Stack(_) => "Stack",
        Node::ScrollView(_) => "ScrollView",
        Node::Wrap(_) => "Wrap",
        Node::SwitchLayout(_) => "SwitchLayout",
        Node::TextBox(_) => "TextBox",
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
