impl<'a> Ui<'a> {
    /// Resolve all knob pointer interaction transitions for this frame.
    fn resolve_knob_interaction(
        &mut self,
        spec: KnobRenderSpec<'_>,
        geometry: KnobGeometry,
        value: &mut f32,
    ) -> KnobResponse {
        let hovered = self.pointer_inside_clipped_rect(geometry.hit_rect);
        if hovered {
            self.state.hot = Some(spec.id);
        }
        let mut response = KnobResponse {
            hovered,
            active: self.state.active == Some(spec.id),
            changed: false,
        };
        self.update_knob_active_state(spec.id, hovered, &mut response, *value);
        let drag_changed = self.apply_knob_drag(spec.id, spec.range, value);
        let wheel_changed = self.apply_knob_wheel(hovered, spec.range, value);
        response.changed = drag_changed || wheel_changed;
        response
    }

    /// Update active/pressed/released state transitions for a knob.
    fn update_knob_active_state(
        &mut self,
        id: WidgetId,
        hovered: bool,
        response: &mut KnobResponse,
        current_value: f32,
    ) {
        if hovered && self.claim_mouse_pressed() {
            self.state.active = Some(id);
            self.state.drag_start = Some(self.input.pointer_pos);
            self.state.drag_value = current_value;
            response.active = true;
        }
        if self.state.active == Some(id) && self.input.mouse_released {
            self.state.active = None;
            self.state.drag_start = None;
            response.active = false;
        }
    }

    /// Apply pointer-drag delta when this knob is active.
    fn apply_knob_drag(&mut self, id: WidgetId, range: KnobRange, value: &mut f32) -> bool {
        if self.state.active != Some(id) || !self.input.mouse_down {
            return false;
        }
        let Some(drag_start) = self.state.drag_start else {
            return false;
        };
        let drag_delta_y = (self.input.pointer_pos.y - drag_start.y) as f32;
        let value_delta = -drag_delta_y * 0.005 * range.span();
        let new_value = range.clamp(self.state.drag_value + value_delta);
        if (*value - new_value).abs() <= f32::EPSILON {
            return false;
        }
        *value = new_value;
        true
    }

    /// Apply wheel step updates when hovered.
    fn apply_knob_wheel(&mut self, hovered: bool, range: KnobRange, value: &mut f32) -> bool {
        if !hovered || self.input.wheel_delta == 0.0 {
            return false;
        }
        let step = 0.02 * range.span();
        let new_value = range.clamp(*value - step * self.input.wheel_delta.signum());
        if (*value - new_value).abs() <= f32::EPSILON {
            return false;
        }
        *value = new_value;
        true
    }
}
