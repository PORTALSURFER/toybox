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
