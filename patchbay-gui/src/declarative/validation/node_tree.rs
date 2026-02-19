/// Validate the top-level UI specification.
fn validate_spec(spec: &UiSpec) -> Result<(), DeclarativeError> {
    if spec.root.key.trim().is_empty() {
        return Err(DeclarativeError::EmptyNodeKey {
            node_kind: "RootFrame",
        });
    }
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
    validate_node(&slot.child, seen_keys)
}

/// Validate a node subtree.
fn validate_node(
    node: &Node,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), DeclarativeError> {
    match node {
        Node::Slot(slot) => validate_slot_node(slot, seen_keys),
        Node::Panel(panel) => validate_panel_node(panel, seen_keys),
        Node::Row(flex) => validate_container_children("Row", &flex.children, seen_keys),
        Node::Column(flex) => validate_container_children("Column", &flex.children, seen_keys),
        Node::Grid(grid) => validate_grid_node(grid, seen_keys),
        Node::Absolute(absolute) => validate_absolute_node(absolute, seen_keys),
        Node::Label(_) | Node::Spacer(_) | Node::Indicator(_) => Ok(()),
        Node::Knob(knob) => validate_knob_node(knob, seen_keys),
        Node::Slider(slider) => validate_slider_node(slider, seen_keys),
        Node::Toggle(toggle) => validate_toggle_node(toggle, seen_keys),
        Node::Button(button) => validate_button_node(button, seen_keys),
        Node::Dropdown(dropdown) => validate_dropdown_node(dropdown, seen_keys),
        Node::Region(region) => validate_region_node(region, seen_keys),
    }
}

/// Validate panel key constraints and recurse into panel slot content.
fn validate_panel_node(
    panel: &PanelSpec,
    seen_keys: &mut std::collections::HashSet<String>,
) -> Result<(), DeclarativeError> {
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

/// Return true when a node acts as a widget leaf.
fn is_widget_node(node: &Node) -> bool {
    matches!(
        node,
        Node::Label(_)
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
