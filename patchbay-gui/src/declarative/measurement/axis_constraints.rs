/// Resolve a measured size against box constraints.
fn resolve_size(layout: LayoutBox, measured: Size, available: Size) -> Size {
    Size {
        width: resolve_axis(
            layout.width,
            measured.width,
            available.width,
            layout.min_width,
            layout.max_width,
        ),
        height: resolve_axis(
            layout.height,
            measured.height,
            available.height,
            layout.min_height,
            layout.max_height,
        ),
    }
}

/// Resolve a measured size against box constraints and emit runtime diagnostics
/// for invalid min/max axis bounds.
fn resolve_size_with_diagnostics(
    layout: LayoutBox,
    measured: Size,
    available: Size,
    container_kind: ContainerKind,
    layout_diagnostics: &mut Vec<LayoutDiagnostic>,
) -> Size {
    Size {
        width: resolve_axis_with_diagnostics(
            layout.width,
            measured.width,
            available.width,
            layout.min_width,
            layout.max_width,
            ConstraintDiagnosticContext {
                container_kind,
                axis: ConstraintAxis::Width,
            },
            layout_diagnostics,
        ),
        height: resolve_axis_with_diagnostics(
            layout.height,
            measured.height,
            available.height,
            layout.min_height,
            layout.max_height,
            ConstraintDiagnosticContext {
                container_kind,
                axis: ConstraintAxis::Height,
            },
            layout_diagnostics,
        ),
    }
}

/// Resolve a single-axis length against constraints.
fn resolve_axis(
    length: Length,
    measured: u32,
    available: u32,
    min: Option<u32>,
    max: Option<u32>,
) -> u32 {
    let normalized = normalize_axis_bounds(min, max, "layout-axis");
    resolve_axis_from_normalized(length, measured, available, normalized.min, normalized.max)
}

/// Resolve a single-axis length against constraints and emit diagnostics for
/// invalid min/max ordering.
fn resolve_axis_with_diagnostics(
    length: Length,
    measured: u32,
    available: u32,
    min: Option<u32>,
    max: Option<u32>,
    diagnostic_context: ConstraintDiagnosticContext,
    layout_diagnostics: &mut Vec<LayoutDiagnostic>,
) -> u32 {
    let normalized = normalize_axis_bounds(min, max, "layout-axis");
    if normalized.was_normalized {
        record_constraint_normalized_diagnostic(
            layout_diagnostics,
            diagnostic_context.container_kind,
            diagnostic_context.axis,
        );
    }
    resolve_axis_from_normalized(length, measured, available, normalized.min, normalized.max)
}

/// Static diagnostic context for one axis resolve operation.
struct ConstraintDiagnosticContext {
    /// Container kind associated with the active resolve call.
    container_kind: ContainerKind,
    /// Axis resolved by the active call.
    axis: ConstraintAxis,
}

/// Resolve one axis from normalized bounds.
fn resolve_axis_from_normalized(
    length: Length,
    measured: u32,
    available: u32,
    min: Option<u32>,
    max: Option<u32>,
) -> u32 {
    let base = match length {
        Length::Auto => measured,
        Length::Px(px) => px.max(measured),
        Length::Fill(_) => available,
    };
    let min_applied = base.max(min.unwrap_or(0));
    if let Some(max_value) = max {
        min_applied.min(max_value)
    } else {
        min_applied
    }
}

/// Normalized min/max bounds for one axis.
struct NormalizedAxisBounds {
    /// Normalized min bound.
    min: Option<u32>,
    /// Normalized max bound.
    max: Option<u32>,
    /// True when invalid bounds were normalized.
    was_normalized: bool,
}

/// Emit a warning when min/max constraints are ordered invalidly.
fn normalize_axis_bounds(
    min: Option<u32>,
    max: Option<u32>,
    axis: &'static str,
) -> NormalizedAxisBounds {
    match (min, max) {
        (Some(min_value), Some(max_value)) if min_value > max_value => {
            emit_layout_bound_warning(axis, min_value, max_value);
            NormalizedAxisBounds {
                min: Some(max_value),
                max: Some(max_value),
                was_normalized: true,
            }
        }
        _ => NormalizedAxisBounds {
            min,
            max,
            was_normalized: false,
        },
    }
}

#[cfg(feature = "layout-overflow-warnings")]
/// Emit a layout warning when axis min/max constraints are invalid and need
/// normalization.
fn emit_layout_bound_warning(axis: &'static str, min: u32, max: u32) {
    eprintln!("patchbay-gui warning: {axis} min ({min}) exceeds max ({max}); normalizing to {max}");
}

#[cfg(not(feature = "layout-overflow-warnings"))]
/// No-op when overflow-warning logs are disabled.
fn emit_layout_bound_warning(_axis: &'static str, _min: u32, _max: u32) {}
