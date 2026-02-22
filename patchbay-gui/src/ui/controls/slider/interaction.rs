impl<'a> Ui<'a> {
    /// Resolve slider hover/active state for this frame.
    pub(crate) fn begin_slider_interaction(&mut self, id: WidgetId, rect: Rect) -> SliderResponse {
        let hovered = self.pointer_inside_clipped_rect(rect);
        if hovered {
            self.state.hot = Some(id);
        }

        let mut response = SliderResponse {
            hovered,
            active: self.state.active == Some(id),
            changed: false,
        };
        if hovered && self.claim_mouse_pressed() {
            self.state.active = Some(id);
            response.active = true;
        }
        if self.state.active == Some(id) && (self.input.mouse_released || !self.input.mouse_down) {
            self.state.active = None;
            response.active = false;
        }
        response
    }

    /// Apply mouse-drag value updates while this slider is active.
    pub(crate) fn apply_slider_drag(
        &mut self,
        id: WidgetId,
        rect: Rect,
        range: (f32, f32),
        value: &mut f32,
    ) -> bool {
        if self.state.active != Some(id) || !self.input.mouse_down {
            return false;
        }
        let span = (range.1 - range.0).max(1.0e-6);
        let x = (self.input.pointer_pos.x - rect.origin.x) as f32;
        let t = (x / rect.size.width.max(1) as f32).clamp(0.0, 1.0);
        self.apply_slider_value(value, range, range.0 + t * span)
    }

    /// Apply wheel-based value updates when the slider is hovered.
    pub(crate) fn apply_slider_wheel(
        &mut self,
        hovered: bool,
        range: (f32, f32),
        value: &mut f32,
    ) -> bool {
        if !hovered || self.input.wheel_delta == 0.0 {
            return false;
        }
        let span = (range.1 - range.0).max(1.0e-6);
        let step = 0.02 * span;
        let next = (*value + step * self.input.wheel_delta.signum()).clamp(range.0, range.1);
        self.apply_slider_value(value, range, next)
    }

    /// Write a slider value if it changed by more than epsilon.
    pub(crate) fn apply_slider_value(&self, value: &mut f32, range: (f32, f32), next: f32) -> bool {
        let clamped = next.clamp(range.0, range.1);
        if (*value - clamped).abs() <= f32::EPSILON {
            return false;
        }
        *value = clamped;
        true
    }

    /// Reset slider value when the control receives a primary-button double click.
    pub(crate) fn apply_slider_double_click_reset(
        &mut self,
        id: WidgetId,
        range: (f32, f32),
        default_value: f32,
        value: &mut f32,
    ) -> bool {
        if self.input.mouse_pressed {
            let _ = self.claim_mouse_pressed();
        }
        if self.state.active == Some(id) {
            self.state.active = None;
        }
        self.apply_slider_value(value, range, default_value)
    }

    /// Resolve slider track fill color from interaction state.
    pub(crate) fn resolve_slider_fill_color(
        &self,
        _variants: Option<ControlColorVariants>,
        disabled: bool,
        response: SliderResponse,
    ) -> Color {
        if disabled {
            return self.theme.knob_fill;
        }
        if response.active {
            self.theme.knob_active
        } else if response.hovered {
            self.theme.knob_hover
        } else {
            self.theme.knob_fill
        }
    }
}
