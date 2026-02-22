/// Validate strict slot-track definitions for canonical slot grids.
fn validate_slot_tracks(grid: &GridSpec) -> Result<(), DeclarativeError> {
    let tracks = match grid.kind {
        GridKind::SlotColumn => &grid.template.rows,
        GridKind::SlotRow => &grid.template.columns,
        GridKind::Standard => return Ok(()),
    };
    if tracks.is_empty() {
        return Err(DeclarativeError::InvalidSlotTrack);
    }

    let mut total_percent = 0u16;
    let mut flexible_count = 0usize;
    for track in tracks {
        match *track {
            TrackSize::Percent(percent) => {
                total_percent = total_percent.saturating_add(percent as u16);
            }
            TrackSize::Fill | TrackSize::Fr(_) => {
                flexible_count = flexible_count.saturating_add(1)
            }
            TrackSize::Auto => {}
            TrackSize::Px(_) => return Err(DeclarativeError::InvalidFixedGridTrack { axis: "slot" }),
        }
    }
    if total_percent > 100 {
        return Err(DeclarativeError::InvalidSlotFractions {
            total_percent,
            fill_count: flexible_count,
        });
    }
    Ok(())
}

/// Validate that grid templates do not use fixed-pixel tracks.
fn validate_no_fixed_grid_tracks(grid: &GridSpec) -> Result<(), DeclarativeError> {
    if grid
        .template
        .columns
        .iter()
        .any(|track| matches!(track, TrackSize::Px(_)))
    {
        return Err(DeclarativeError::InvalidFixedGridTrack { axis: "columns" });
    }
    if grid
        .template
        .rows
        .iter()
        .any(|track| matches!(track, TrackSize::Px(_)))
    {
        return Err(DeclarativeError::InvalidFixedGridTrack { axis: "rows" });
    }
    Ok(())
}

/// Validate that container layout remains host-derived (no absolute sizing).
fn validate_container_layout(
    container_kind: &'static str,
    layout: LayoutBox,
) -> Result<(), DeclarativeError> {
    validate_layout_bounds(container_kind, layout)?;
    let has_absolute_length =
        matches!(layout.width, Length::Px(_)) || matches!(layout.height, Length::Px(_));
    let has_bounds =
        layout.min_width.is_some()
            || layout.min_height.is_some()
            || layout.max_width.is_some()
            || layout.max_height.is_some();
    if has_absolute_length || has_bounds {
        return Err(DeclarativeError::InvalidContainerLayout { container_kind });
    }
    Ok(())
}

/// Validate min/max ordering for widget/root layout boxes.
fn validate_layout_bounds(
    node_kind: &'static str,
    layout: LayoutBox,
) -> Result<(), DeclarativeError> {
    validate_axis_bounds(node_kind, "width", layout.min_width, layout.max_width)?;
    validate_axis_bounds(node_kind, "height", layout.min_height, layout.max_height)
}

/// Validate one axis min/max ordering.
fn validate_axis_bounds(
    node_kind: &'static str,
    axis: &'static str,
    min: Option<u32>,
    max: Option<u32>,
) -> Result<(), DeclarativeError> {
    if let (Some(min), Some(max)) = (min, max)
        && min > max
    {
        return Err(DeclarativeError::InvalidLayoutBounds {
            node_kind,
            axis,
            min,
            max,
        });
    }
    Ok(())
}

/// Validate aspect-box ratio components.
fn validate_aspect_ratio(aspect_ratio: AspectRatio) -> Result<(), DeclarativeError> {
    if aspect_ratio.width == 0 || aspect_ratio.height == 0 {
        return Err(DeclarativeError::InvalidAspectRatio {
            width: aspect_ratio.width,
            height: aspect_ratio.height,
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
    if dropdown.selected >= dropdown.option_count {
        return Err(DeclarativeError::InvalidDropdownSelection {
            key: dropdown.key.clone(),
            selected: dropdown.selected,
            options_len: dropdown.option_count,
        });
    }
    Ok(())
}

/// Validate that tab-bar selection references an existing tab.
fn validate_tab_bar_selection(tab_bar: &TabBarSpec) -> Result<(), DeclarativeError> {
    if tab_bar.tab_count == 0 || tab_bar.selected >= tab_bar.tab_count {
        return Err(DeclarativeError::InvalidTabBarSelection {
            key: tab_bar.key.clone(),
            selected: tab_bar.selected,
            tab_count: tab_bar.tab_count,
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

/// Validate deterministic ordering for switch-layout width cases.
fn validate_switch_case_ranges(cases: &[SwitchCase]) -> Result<(), DeclarativeError> {
    for (case_index, case_entry) in cases.iter().enumerate() {
        let range = case_entry.range();
        if let (Some(min), Some(max)) = (range.min_inclusive, range.max_exclusive)
            && min >= max
        {
            return Err(DeclarativeError::InvalidSwitchCaseRange {
                case_index,
                min_inclusive: range.min_inclusive,
                max_exclusive: range.max_exclusive,
            });
        }
    }

    for (case_index, case_entry) in cases.iter().enumerate().skip(1) {
        let previous = cases[case_index - 1].range();
        let current = case_entry.range();
        let current_min = current.min_inclusive.unwrap_or(0);
        let Some(previous_max) = previous.max_exclusive else {
            return Err(DeclarativeError::InvalidSwitchCaseOrder {
                previous_case_index: case_index - 1,
                case_index,
                previous_max_exclusive: u32::MAX,
                case_min_inclusive: current_min,
            });
        };
        if current_min < previous_max {
            return Err(DeclarativeError::InvalidSwitchCaseOrder {
                previous_case_index: case_index - 1,
                case_index,
                previous_max_exclusive: previous_max,
                case_min_inclusive: current_min,
            });
        }
    }
    Ok(())
}
