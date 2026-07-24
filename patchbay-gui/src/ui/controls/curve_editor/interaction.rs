/// Base point hit radius in design pixels.
const NODE_HIT_RADIUS: i32 = 8;
/// Base near-segment hit radius in design pixels.
const SEGMENT_NEAR_HIT_RADIUS: i32 = 16;
/// Base direct-segment hit radius in design pixels.
const SEGMENT_DIRECT_HIT_RADIUS: i32 = 6;
/// Base point insert-guard radius in design pixels.
const NODE_INSERT_GUARD_RADIUS: i32 = 12;

impl<'a> Ui<'a> {
    /// Load runtime state for one curve-editor widget.
    fn begin_curve_editor_runtime(&mut self, id: WidgetId) -> CurveEditorRuntimeState {
        self.state
            .curve_editor_runtime
            .get(&id)
            .cloned()
            .unwrap_or_default()
    }

    /// Persist runtime state for one curve-editor widget.
    fn set_curve_editor_runtime(&mut self, id: WidgetId, runtime: CurveEditorRuntimeState) {
        self.state.curve_editor_runtime.insert(id, runtime);
    }

    /// Return true when pointer movement passed drag activation threshold.
    fn curve_editor_drag_threshold_reached(
        start_pointer: Point,
        local_pointer: Point,
        threshold_px: i32,
    ) -> bool {
        let threshold = threshold_px.max(0);
        distance_squared(start_pointer, local_pointer) >= i64::from(threshold * threshold)
    }

    /// Resolve per-frame visual hover/preview state.
    fn resolve_curve_editor_visual_state(
        &self,
        model: &crate::declarative::CurveModel,
        runtime: &CurveEditorRuntimeState,
        interaction: &crate::declarative::CurveInteractionOptions,
        segment_move: Option<crate::declarative::CurveSegmentMoveOptions>,
        region: RegionResponse,
        rect: Rect,
    ) -> CurveEditorVisualState {
        let node_hit_radius = scaled_curve_i32(NODE_HIT_RADIUS, rect);
        let segment_near_radius = scaled_curve_i32(SEGMENT_NEAR_HIT_RADIUS, rect);
        let segment_direct_radius = scaled_curve_i32(SEGMENT_DIRECT_HIT_RADIUS, rect);
        let hovered_point = region
            .hovered
            .then(|| find_point_hit_within(model, region.local_pointer, node_hit_radius, rect))
            .flatten();
        let direct_segment = region
            .hovered
            .then(|| {
                find_segment_hit_within(model, region.local_pointer, segment_direct_radius, rect)
            })
            .flatten();
        let near_segment = region
            .hovered
            .then(|| {
                find_segment_hit_within(model, region.local_pointer, segment_near_radius, rect)
            })
            .flatten();
        let active_move_segment = match runtime.drag_mode.as_ref() {
            Some(CurveEditorDragMode::MoveSegment { index, .. }) => Some(*index),
            _ => None,
        };
        let modifier_gates_segment_move = segment_move.is_some();
        let segment_move_modifier_down = segment_move
            .is_some_and(|options| curve_editor_modifier_down(options.modifier, region));
        let segment_move_segment = (region.hovered
            && !region.alt_down
            && hovered_point.is_none()
            && segment_move_modifier_down)
            .then_some(active_move_segment.or(near_segment))
            .flatten();
        let preview_point = (region.hovered
            && !region.alt_down
            && runtime.drag_mode.is_none()
            && hovered_point.is_none()
            && segment_move_segment.is_none()
            && direct_segment.is_some())
        .then(|| preview_point_on_curve(model, region.local_pointer, rect, &interaction.snap))
        .flatten();
        let hovered_segment = (region.hovered
            && preview_point.is_none()
            && segment_move_segment.is_none()
            && (!modifier_gates_segment_move || region.alt_down))
        .then_some(near_segment)
        .flatten();
        CurveEditorVisualState {
            selected_point: runtime.selected_point,
            hovered_point,
            hovered_segment,
            segment_move_segment,
            segment_move_highlight: segment_move.map(|options| options.highlight),
            preview_point,
        }
    }

    /// Reduce one frame of curve-editor interaction into model/runtime state.
    fn reduce_curve_editor_interaction(
        &self,
        model: &mut crate::declarative::CurveModel,
        runtime: &mut CurveEditorRuntimeState,
        interaction: crate::declarative::CurveInteractionOptions,
        decorators: CurveEditorInteractionDecorators,
        region: RegionResponse,
        rect: Rect,
    ) -> bool {
        let mut changed = false;
        let local_pointer = region.local_pointer;
        let raw_local_pointer = region.raw_local_pointer;
        let normalized_pointer = curve_point_from_local(local_pointer, rect);
        let raw_normalized_pointer = curve_point_from_local(raw_local_pointer, rect);
        let point_vertical_constraint_down = decorators.point_vertical_constraint.is_some()
            && region.shift_down
            && region.alt_down;
        let point_horizontal_constraint_down = decorators
            .point_horizontal_constraint
            .is_some_and(|_| region.shift_down)
            && !point_vertical_constraint_down;
        let segment_move = decorators.segment_move;

        // Focus loss can clear the host button state without a release frame.
        // Do not let that stale drag mode leak into the next gesture.
        if runtime.drag_mode.is_some()
            && region.active
            && !region.pressed
            && !region.dragged
            && !region.released
        {
            runtime.drag_mode = None;
        }

        // Windows reports a double click as a press + double-click in the same frame.
        // Handle deletion first so the press path cannot swallow the action.
        if region.double_clicked && interaction.double_click_delete_interior {
            let node_hit_radius = scaled_curve_i32(NODE_HIT_RADIUS, rect);
            if let Some(index) = find_point_hit_within(model, local_pointer, node_hit_radius, rect)
                && index > 0
                && index + 1 < model.points.len()
            {
                remove_interior_point(model, index);
                runtime.selected_point = None;
                runtime.drag_mode = None;
                enforce_endpoint_mode(model, interaction.endpoint_mode);
                return true;
            }
        }

        if region.pressed {
            let node_hit_radius = scaled_curve_i32(NODE_HIT_RADIUS, rect);
            let segment_near_radius = scaled_curve_i32(SEGMENT_NEAR_HIT_RADIUS, rect);
            let segment_direct_radius = scaled_curve_i32(SEGMENT_DIRECT_HIT_RADIUS, rect);
            let node_insert_guard = scaled_curve_i32(NODE_INSERT_GUARD_RADIUS, rect);
            if let Some(index) = find_point_hit_within(model, local_pointer, node_hit_radius, rect) {
                runtime.selected_point = Some(index);
                runtime.drag_mode = Some(move_point_drag_mode(
                    model,
                    index,
                    local_pointer,
                    point_horizontal_constraint_down,
                    point_vertical_constraint_down,
                ));
                return false;
            }
            let near_segment =
                find_segment_hit_within(model, local_pointer, segment_near_radius, rect);
            let modifier_gated_segment = segment_move
                .filter(|options| curve_editor_modifier_down(options.modifier, region))
                .filter(|_| !region.alt_down)
                .and(near_segment);
            if let Some(index) = modifier_gated_segment {
                runtime.drag_mode = Some(move_segment_drag_mode(model, index, local_pointer));
                return false;
            }
            if !region.alt_down
                && find_segment_hit_within(model, local_pointer, segment_direct_radius, rect).is_some()
            {
                let preview =
                    preview_point_on_curve(model, local_pointer, rect, &interaction.snap)
                        .unwrap_or_else(|| snap_curve_point(normalized_pointer, &interaction.snap));
                let inserted_index = insert_point(
                    model,
                    preview,
                    interaction.max_points,
                    interaction.min_point_spacing_x.max(1.0e-6),
                );
                runtime.selected_point = Some(inserted_index);
                runtime.drag_mode = Some(move_point_drag_mode(
                    model,
                    inserted_index,
                    local_pointer,
                    point_horizontal_constraint_down,
                    point_vertical_constraint_down,
                ));
                enforce_endpoint_mode(model, interaction.endpoint_mode);
                return true;
            }
            if let Some(index) = near_segment {
                runtime.drag_mode = if region.alt_down {
                    let start_tension = model
                        .segments
                        .get(index)
                        .copied()
                        .unwrap_or(crate::declarative::CurveSegment { tension: 0.0 })
                        .tension;
                    Some(CurveEditorDragMode::AdjustSegmentTension {
                        index,
                        start_pointer: local_pointer,
                        start_tension,
                        dragging: false,
                    })
                } else if segment_move.is_none() {
                    Some(move_segment_drag_mode(model, index, local_pointer))
                } else {
                    None
                };
                if runtime.drag_mode.is_some() {
                    return false;
                }
            }
            if let Some(index) = find_point_hit_within(model, local_pointer, node_insert_guard, rect) {
                runtime.selected_point = Some(index);
                runtime.drag_mode = Some(move_point_drag_mode(
                    model,
                    index,
                    local_pointer,
                    point_horizontal_constraint_down,
                    point_vertical_constraint_down,
                ));
                return false;
            }
            if interaction.whole_curve_offset
                && region.command_down
                && region.shift_down
                && !region.alt_down
            {
                runtime.drag_mode = Some(CurveEditorDragMode::OffsetCurve {
                    origin_model: model.clone(),
                    start_pointer: local_pointer,
                    dragging: false,
                });
                runtime.selected_point = None;
                return false;
            }
            let inserted_index = insert_point(
                model,
                snap_curve_point(normalized_pointer, &interaction.snap),
                interaction.max_points,
                interaction.min_point_spacing_x.max(1.0e-6),
            );
            runtime.selected_point = Some(inserted_index);
            runtime.drag_mode = Some(move_point_drag_mode(
                model,
                inserted_index,
                local_pointer,
                point_horizontal_constraint_down,
                point_vertical_constraint_down,
            ));
            enforce_endpoint_mode(model, interaction.endpoint_mode);
            return true;
        }

        if region.dragged && let Some(mut drag_mode) = runtime.drag_mode.take() {
            match drag_mode {
                    CurveEditorDragMode::OffsetCurve {
                        origin_model,
                        start_pointer,
                        mut dragging,
                    } => {
                        if !region.command_down || !region.shift_down || region.alt_down {
                            return false;
                        }
                        if !dragging
                            && !Self::curve_editor_drag_threshold_reached(
                                start_pointer,
                                local_pointer,
                                interaction.drag_start_threshold_px,
                            )
                        {
                            runtime.drag_mode = Some(CurveEditorDragMode::OffsetCurve {
                                origin_model,
                                start_pointer,
                                dragging,
                            });
                            return false;
                        }
                        dragging = true;
                        let width = (rect.size.width.max(2) - 1) as f32;
                        let delta = (raw_local_pointer.x - start_pointer.x) as f32 / width;
                        *model = cyclically_offset_curve_model(
                            &origin_model,
                            delta,
                            interaction.max_points,
                        );
                        enforce_endpoint_mode(model, interaction.endpoint_mode);
                        drag_mode = CurveEditorDragMode::OffsetCurve {
                            origin_model,
                            start_pointer,
                            dragging,
                        };
                        changed = true;
                    }
                    CurveEditorDragMode::MovePoint {
                        origin_index,
                        origin_model,
                        start_pointer,
                        mut dragging,
                        mut horizontal_constraint_active,
                        mut horizontal_constraint_anchor_y,
                        mut vertical_pointer_offset_y,
                        mut vertical_pointer_rebased,
                        mut vertical_constraint_active,
                        mut vertical_constraint_anchor_x,
                        mut horizontal_pointer_offset_x,
                        mut horizontal_pointer_rebased,
                    } => {
                        let visible_point = runtime
                            .selected_point
                            .and_then(|index| model.points.get(index))
                            .copied()
                            .or_else(|| origin_model.points.get(origin_index).copied());
                        let horizontal_constraint_released_this_frame =
                            !point_horizontal_constraint_down && horizontal_constraint_active;
                        let vertical_constraint_released_this_frame =
                            !point_vertical_constraint_down && vertical_constraint_active;
                        if point_vertical_constraint_down && !vertical_constraint_active {
                            vertical_constraint_anchor_x = visible_point.map(|point| point.x);
                            vertical_constraint_active = vertical_constraint_anchor_x.is_some();
                        } else if !point_vertical_constraint_down && vertical_constraint_active {
                            if let Some(anchor_x) = vertical_constraint_anchor_x {
                                horizontal_pointer_offset_x = anchor_x - raw_normalized_pointer.x;
                                horizontal_pointer_rebased = true;
                            }
                            vertical_constraint_active = false;
                            vertical_constraint_anchor_x = None;
                        }
                        if point_horizontal_constraint_down && !horizontal_constraint_active {
                            horizontal_constraint_anchor_y = visible_point.map(|point| point.y);
                            horizontal_constraint_active =
                                horizontal_constraint_anchor_y.is_some();
                        } else if !point_horizontal_constraint_down
                            && horizontal_constraint_active
                        {
                            if let Some(anchor_y) = horizontal_constraint_anchor_y {
                                vertical_pointer_offset_y = anchor_y - raw_normalized_pointer.y;
                                vertical_pointer_rebased = true;
                            }
                            horizontal_constraint_active = false;
                            horizontal_constraint_anchor_y = None;
                        }
                        if !dragging
                            && !Self::curve_editor_drag_threshold_reached(
                                start_pointer,
                                local_pointer,
                                interaction.drag_start_threshold_px,
                            )
                        {
                            runtime.drag_mode = Some(CurveEditorDragMode::MovePoint {
                                origin_index,
                                origin_model,
                                start_pointer,
                                dragging,
                                horizontal_constraint_active,
                                horizontal_constraint_anchor_y,
                                vertical_pointer_offset_y,
                                vertical_pointer_rebased,
                                vertical_constraint_active,
                                vertical_constraint_anchor_x,
                                horizontal_pointer_offset_x,
                                horizontal_pointer_rebased,
                            });
                            return false;
                        }
                        dragging = true;
                        let mut effective_pointer = raw_normalized_pointer;
                        effective_pointer.x = if vertical_constraint_active {
                            vertical_constraint_anchor_x.unwrap_or(effective_pointer.x)
                        } else if horizontal_pointer_rebased {
                            effective_pointer.x + horizontal_pointer_offset_x
                        } else {
                            effective_pointer.x
                        };
                        effective_pointer.y = if horizontal_constraint_active {
                            horizontal_constraint_anchor_y.unwrap_or(effective_pointer.y)
                        } else if vertical_pointer_rebased {
                            effective_pointer.y + vertical_pointer_offset_y
                        } else {
                            effective_pointer.y
                        };
                        let mut effective_snap = interaction.snap.clone();
                        if horizontal_constraint_active
                            || horizontal_constraint_released_this_frame
                        {
                            effective_snap.horizontal_positions.clear();
                        }
                        if vertical_constraint_active || vertical_constraint_released_this_frame {
                            effective_snap.vertical_positions.clear();
                        }
                        let (recomputed, moved_index) = recompute_move_point_from_origin(
                            &origin_model,
                            origin_index,
                            effective_pointer,
                            interaction.min_point_spacing_x.max(1.0e-6),
                            interaction.push_through_threshold_px,
                            rect,
                            interaction.endpoint_mode,
                            &effective_snap,
                        );
                        *model = recomputed;
                        runtime.selected_point = Some(moved_index);
                        drag_mode = CurveEditorDragMode::MovePoint {
                            origin_index,
                            origin_model,
                            start_pointer,
                            dragging,
                            horizontal_constraint_active,
                            horizontal_constraint_anchor_y,
                            vertical_pointer_offset_y,
                            vertical_pointer_rebased,
                            vertical_constraint_active,
                            vertical_constraint_anchor_x,
                            horizontal_pointer_offset_x,
                            horizontal_pointer_rebased,
                        };
                        changed = true;
                    }
                    CurveEditorDragMode::MoveSegment {
                        index,
                        start_pointer,
                        start_left_x,
                        start_right_x,
                        start_left_y,
                        start_right_y,
                        mut dragging,
                    } => {
                        let gated_drag_is_invalid = segment_move
                            .is_some_and(|options| {
                                !curve_editor_modifier_down(options.modifier, region)
                                    || !region.hovered
                            });
                        if gated_drag_is_invalid {
                            return false;
                        }
                        if !dragging
                            && !Self::curve_editor_drag_threshold_reached(
                                start_pointer,
                                local_pointer,
                                interaction.drag_start_threshold_px,
                            )
                        {
                            runtime.drag_mode = Some(CurveEditorDragMode::MoveSegment {
                                index,
                                start_pointer,
                                start_left_x,
                                start_right_x,
                                start_left_y,
                                start_right_y,
                                dragging,
                            });
                            return false;
                        }
                        dragging = true;
                        let curve_width = rect.size.width.max(2);
                        let curve_height = rect.size.height.max(2);
                        let delta_x =
                            (raw_local_pointer.x - start_pointer.x) as f32 / (curve_width - 1) as f32;
                        let delta_y =
                            (start_pointer.y - raw_local_pointer.y) as f32 / (curve_height - 1) as f32;
                        move_segment_translated(
                            model,
                            index,
                            (start_left_x, start_left_y),
                            (start_right_x, start_right_y),
                            (delta_x, delta_y),
                            interaction.min_point_spacing_x.max(1.0e-6),
                            interaction.endpoint_mode,
                            &interaction.snap,
                        );
                        drag_mode = CurveEditorDragMode::MoveSegment {
                            index,
                            start_pointer,
                            start_left_x,
                            start_right_x,
                            start_left_y,
                            start_right_y,
                            dragging,
                        };
                        changed = true;
                    }
                    CurveEditorDragMode::AdjustSegmentTension {
                        index,
                        start_pointer,
                        start_tension,
                        mut dragging,
                    } => {
                        if !dragging
                            && !Self::curve_editor_drag_threshold_reached(
                                start_pointer,
                                local_pointer,
                                interaction.drag_start_threshold_px,
                            )
                        {
                            runtime.drag_mode = Some(CurveEditorDragMode::AdjustSegmentTension {
                                index,
                                start_pointer,
                                start_tension,
                                dragging,
                            });
                            return false;
                        }
                        dragging = true;
                        model.clear_phase_metadata();
                        let delta =
                            tension_delta_from_drag(model, index, start_pointer, raw_local_pointer, rect);
                        if let Some(segment) = model.segments.get_mut(index) {
                            segment.tension = (start_tension + delta).clamp(
                                crate::declarative::CURVE_SEGMENT_TENSION_MIN,
                                crate::declarative::CURVE_SEGMENT_TENSION_MAX,
                            );
                            changed = true;
                        }
                        drag_mode = CurveEditorDragMode::AdjustSegmentTension {
                            index,
                            start_pointer,
                            start_tension,
                            dragging,
                        };
                    }
            }
            runtime.drag_mode = Some(drag_mode);
        }

        if region.released {
            runtime.drag_mode = None;
        }

        changed
    }
}

/// Rebuild a normalized curve after translating its phase around one cycle.
fn cyclically_offset_curve_model(
    model: &crate::declarative::CurveModel,
    delta: f32,
    max_points: usize,
) -> crate::declarative::CurveModel {
    const EPSILON: f32 = 1.0e-5;
    let origin = model.clone().normalized();
    if origin.points.len() < 2 || !delta.is_finite() {
        return origin;
    }
    let delta = delta.rem_euclid(1.0);
    if delta <= EPSILON || 1.0 - delta <= EPSILON {
        return origin;
    }

    let last = origin.points.len() - 1;
    let mut boundaries = vec![0.0_f32];
    for point in origin.points[..last].iter().copied() {
        let shifted = (point.x + delta).rem_euclid(1.0);
        if shifted > EPSILON && shifted < 1.0 - EPSILON {
            boundaries.push(shifted);
        }
    }
    boundaries.sort_by(f32::total_cmp);
    boundaries.dedup_by(|left, right| (*left - *right).abs() <= EPSILON);
    boundaries.push(1.0);

    let split_count = boundaries
        .windows(2)
        .filter(|window| !offset_interval_matches_origin(&origin, delta, window[0], window[1]))
        .count();
    let split_subdivision_count = if split_count == 0 {
        1
    } else {
        (max_points.saturating_sub(boundaries.len()) / split_count + 1).max(1)
    };
    let mut positions = Vec::with_capacity(
        boundaries.len() + split_count * split_subdivision_count.saturating_sub(1),
    );
    for window in boundaries.windows(2) {
        positions.push(window[0]);
        if !offset_interval_matches_origin(&origin, delta, window[0], window[1]) {
            let span = window[1] - window[0];
            for subdivision in 1..split_subdivision_count {
                positions.push(
                    window[0] + span * subdivision as f32 / split_subdivision_count as f32,
                );
            }
        }
    }
    positions.push(1.0);

    let seam_y = sample_wrapped(&origin, -delta);
    let points = positions
        .iter()
        .copied()
        .enumerate()
        .map(|(index, x)| crate::declarative::CurvePoint {
            x,
            y: if index == positions.len() - 1 {
                seam_y
            } else {
                sample_wrapped(&origin, x - delta)
            },
        })
        .collect::<Vec<_>>();
    let segments = positions
        .windows(2)
        .map(|window| crate::declarative::CurveSegment {
            tension: offset_segment_tension(&origin, delta, window[0], window[1]),
        })
        .collect();
    let mut offset = crate::declarative::CurveModel::new(points, segments);
    let source = origin
        .phase_source
        .as_deref()
        .cloned()
        .unwrap_or_else(|| crate::declarative::CurveModel::new(
            origin.points.clone(),
            origin.segments.clone(),
        ));
    offset.phase_source = Some(Box::new(source));
    offset.phase_offset = (if origin.phase_source.is_some() {
        origin.phase_offset + delta
    } else {
        delta
    })
    .rem_euclid(1.0);
    offset.normalized()
}

fn sample_wrapped(model: &crate::declarative::CurveModel, phase: f32) -> f32 {
    model.sample(phase.rem_euclid(1.0))
}

fn source_range(delta: f32, left: f32, right: f32) -> (f32, f32) {
    let mut source_left = (left - delta).rem_euclid(1.0);
    if source_left >= 1.0 - 1.0e-5 {
        source_left = 0.0;
    }
    (source_left, source_left + right - left)
}

fn source_segment(model: &crate::declarative::CurveModel, phase: f32) -> usize {
    let phase = phase.rem_euclid(1.0);
    if phase <= model.points[0].x {
        return 0;
    }
    if phase >= model.points[model.points.len() - 1].x {
        return model.points.len().saturating_sub(2);
    }
    let mut index = 0;
    while index + 1 < model.points.len() && phase > model.points[index + 1].x {
        index += 1;
    }
    index.min(model.points.len().saturating_sub(2))
}

fn offset_interval_matches_origin(
    model: &crate::declarative::CurveModel,
    delta: f32,
    left: f32,
    right: f32,
) -> bool {
    let (source_left, source_right) = source_range(delta, left, right);
    let segment = source_segment(model, source_left + (source_right - source_left) * 0.5);
    (source_left - model.points[segment].x).abs() <= 1.0e-5
        && (source_right - model.points[segment + 1].x).abs() <= 1.0e-5
}

fn offset_segment_tension(
    model: &crate::declarative::CurveModel,
    delta: f32,
    left: f32,
    right: f32,
) -> f32 {
    const EPSILON: f32 = 1.0e-5;
    let (source_left, source_right) = source_range(delta, left, right);
    let source_mid = source_left + (source_right - source_left) * 0.5;
    let segment = source_segment(model, source_mid);
    if offset_interval_matches_origin(model, delta, left, right) {
        return model
            .segments
            .get(segment)
            .copied()
            .unwrap_or(crate::declarative::CurveSegment { tension: 0.0 })
            .tension;
    }
    let left_y = sample_wrapped(model, source_left);
    let right_y = sample_wrapped(model, source_right);
    let span = right_y - left_y;
    if span.abs() <= EPSILON {
        return model
            .segments
            .get(segment)
            .copied()
            .unwrap_or(crate::declarative::CurveSegment { tension: 0.0 })
            .tension;
    }
    let midpoint = ((sample_wrapped(model, source_mid) - left_y) / span).clamp(0.0, 1.0);
    let exponent = if midpoint <= 0.5 {
        (-midpoint.max(EPSILON).ln() / 2.0_f32.ln()).clamp(1.0, 4.0)
    } else {
        (-(1.0 - midpoint).max(EPSILON).ln() / 2.0_f32.ln()).clamp(1.0, 4.0)
    };
    let magnitude = ((exponent - 1.0) / 3.0).clamp(0.0, 1.0);
    if midpoint <= 0.5 { magnitude } else { -magnitude }
}

/// Return whether the configured curve-editor modifier is held this frame.
fn curve_editor_modifier_down(
    modifier: crate::declarative::CurveEditorModifier,
    region: RegionResponse,
) -> bool {
    match modifier {
        crate::declarative::CurveEditorModifier::Command => region.command_down,
    }
}

/// Build one point drag and initialize its modifier constraint state.
fn move_point_drag_mode(
    model: &crate::declarative::CurveModel,
    origin_index: usize,
    start_pointer: Point,
    horizontal_constraint_active: bool,
    vertical_constraint_active: bool,
) -> CurveEditorDragMode {
    CurveEditorDragMode::MovePoint {
        origin_index,
        origin_model: model.clone(),
        start_pointer,
        dragging: false,
        horizontal_constraint_active,
        horizontal_constraint_anchor_y: horizontal_constraint_active
            .then(|| model.points[origin_index].y),
        vertical_pointer_offset_y: 0.0,
        vertical_pointer_rebased: false,
        vertical_constraint_active,
        vertical_constraint_anchor_x: vertical_constraint_active
            .then(|| model.points[origin_index].x),
        horizontal_pointer_offset_x: 0.0,
        horizontal_pointer_rebased: false,
    }
}

/// Build one segment-translation drag from the current pair geometry.
fn move_segment_drag_mode(
    model: &crate::declarative::CurveModel,
    index: usize,
    start_pointer: Point,
) -> CurveEditorDragMode {
    let right_index = (index + 1).min(model.points.len().saturating_sub(1));
    CurveEditorDragMode::MoveSegment {
        index,
        start_pointer,
        start_left_x: model.points[index].x,
        start_right_x: model.points[right_index].x,
        start_left_y: model.points[index].y,
        start_right_y: model.points[right_index].y,
        dragging: false,
    }
}

/// Scale one integer curve metric by current editor rectangle size.
fn scaled_curve_i32(base: i32, rect: Rect) -> i32 {
    scaled_curve_f32(base as f32, rect).round() as i32
}

/// Scale one floating-point curve metric by current editor rectangle size.
fn scaled_curve_f32(base: f32, rect: Rect) -> f32 {
    (base * curve_scale_for_rect(rect)).max(1.0)
}

#[cfg(test)]
mod offset_tests {
    use super::cyclically_offset_curve_model;
    use crate::declarative::{CurveModel, CurvePoint, CurveSegment};

    #[test]
    fn cyclic_offset_preserves_nonlinear_shape_across_inverse_and_repeated_gestures() {
        let origin = CurveModel::new(
            vec![
                CurvePoint::new(0.0, 0.2),
                CurvePoint::new(0.31, 0.9),
                CurvePoint::new(0.67, 0.1),
                CurvePoint::new(1.0, 0.2),
            ],
            vec![CurveSegment::new(0.9), CurveSegment::new(-0.9), CurveSegment::new(0.8)],
        );
        let delta = 0.37;
        let offset = cyclically_offset_curve_model(&origin, delta, 65);
        assert!(offset.phase_source.is_some());

        let restored = cyclically_offset_curve_model(&offset, -delta, 65);
        let mut repeated = origin.clone();
        for _ in 0..8 {
            repeated = cyclically_offset_curve_model(&repeated, delta, 65);
        }
        for index in 0..=400 {
            let phase = index as f32 / 400.0;
            assert!(
                (restored.sample(phase) - origin.sample(phase)).abs() < 1.0e-6,
                "restored phase {phase}: {} != {} (offset {})",
                restored.sample(phase),
                origin.sample(phase),
                restored.phase_offset
            );
            assert!(
                (repeated.sample(phase)
                    - origin.sample((phase - delta * 8.0).rem_euclid(1.0)))
                    .abs()
                    < 5.0e-6,
                "phase {phase}: {} != {}",
                repeated.sample(phase),
                origin.sample((phase - delta * 8.0).rem_euclid(1.0))
            );
        }
    }
}
