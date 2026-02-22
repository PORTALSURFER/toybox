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
        let preview_point =
            (region.hovered && !region.alt_down && hovered_point.is_none() && direct_segment.is_some())
                .then(|| preview_point_on_curve(model, region.local_pointer, rect))
                .flatten();
        let hovered_segment = (region.hovered && preview_point.is_none())
            .then(|| {
                find_segment_hit_within(model, region.local_pointer, segment_near_radius, rect)
            })
            .flatten();
        CurveEditorVisualState {
            selected_point: runtime.selected_point,
            hovered_point,
            hovered_segment,
            preview_point,
        }
    }

    /// Reduce one frame of curve-editor interaction into model/runtime state.
    fn reduce_curve_editor_interaction(
        &self,
        model: &mut crate::declarative::CurveModel,
        runtime: &mut CurveEditorRuntimeState,
        interaction: crate::declarative::CurveInteractionOptions,
        region: RegionResponse,
        rect: Rect,
    ) -> bool {
        let mut changed = false;
        let local_pointer = region.local_pointer;
        let raw_local_pointer = region.raw_local_pointer;
        let normalized_pointer = curve_point_from_local(local_pointer, rect);
        let raw_normalized_pointer = curve_point_from_local(raw_local_pointer, rect);

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
                runtime.drag_mode = Some(CurveEditorDragMode::MovePoint {
                    origin_index: index,
                    origin_model: model.clone(),
                    start_pointer: local_pointer,
                    dragging: false,
                });
                return false;
            }
            if !region.alt_down
                && find_segment_hit_within(model, local_pointer, segment_direct_radius, rect).is_some()
            {
                let preview =
                    preview_point_on_curve(model, local_pointer, rect).unwrap_or(normalized_pointer);
                let inserted_index = insert_point(
                    model,
                    preview,
                    interaction.max_points,
                    interaction.min_point_spacing_x.max(1.0e-6),
                );
                runtime.selected_point = Some(inserted_index);
                runtime.drag_mode = Some(CurveEditorDragMode::MovePoint {
                    origin_index: inserted_index,
                    origin_model: model.clone(),
                    start_pointer: local_pointer,
                    dragging: false,
                });
                enforce_endpoint_mode(model, interaction.endpoint_mode);
                return true;
            }
            if let Some(index) = find_segment_hit_within(model, local_pointer, segment_near_radius, rect)
            {
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
                } else {
                    let right_index = (index + 1).min(model.points.len().saturating_sub(1));
                    Some(CurveEditorDragMode::MoveSegment {
                        index,
                        start_pointer: local_pointer,
                        start_left_x: model.points[index].x,
                        start_right_x: model.points[right_index].x,
                        start_left_y: model.points[index].y,
                        start_right_y: model.points[right_index].y,
                        dragging: false,
                    })
                };
                return false;
            }
            if let Some(index) = find_point_hit_within(model, local_pointer, node_insert_guard, rect) {
                runtime.selected_point = Some(index);
                runtime.drag_mode = Some(CurveEditorDragMode::MovePoint {
                    origin_index: index,
                    origin_model: model.clone(),
                    start_pointer: local_pointer,
                    dragging: false,
                });
                return false;
            }
            let inserted_index = insert_point(
                model,
                normalized_pointer,
                interaction.max_points,
                interaction.min_point_spacing_x.max(1.0e-6),
            );
            runtime.selected_point = Some(inserted_index);
            runtime.drag_mode = Some(CurveEditorDragMode::MovePoint {
                origin_index: inserted_index,
                origin_model: model.clone(),
                start_pointer: local_pointer,
                dragging: false,
            });
            enforce_endpoint_mode(model, interaction.endpoint_mode);
            return true;
        }

        if region.dragged && let Some(mut drag_mode) = runtime.drag_mode.take() {
            match drag_mode {
                    CurveEditorDragMode::MovePoint {
                        origin_index,
                        origin_model,
                        start_pointer,
                        mut dragging,
                    } => {
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
                            });
                            return false;
                        }
                        dragging = true;
                        let (recomputed, moved_index) = recompute_move_point_from_origin(
                            &origin_model,
                            origin_index,
                            raw_normalized_pointer,
                            interaction.min_point_spacing_x.max(1.0e-6),
                            interaction.push_through_threshold_px,
                            rect,
                            interaction.endpoint_mode,
                        );
                        *model = recomputed;
                        runtime.selected_point = Some(moved_index);
                        drag_mode = CurveEditorDragMode::MovePoint {
                            origin_index,
                            origin_model,
                            start_pointer,
                            dragging,
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

/// Scale one integer curve metric by current editor rectangle size.
fn scaled_curve_i32(base: i32, rect: Rect) -> i32 {
    (base as f32 * curve_scale_for_rect(rect)).round().max(1.0) as i32
}
