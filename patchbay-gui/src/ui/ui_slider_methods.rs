impl<'a> Ui<'a> {

    /// Draw a horizontal slider with the given label and value.
    ///
    /// The provided `id` must be stable across frames. If the label changes
    /// while dragging, use [`Ui::slider_with_key`] to provide a separate stable
    /// identifier.
    pub fn slider(
        &mut self,
        id: WidgetId,
        label: &str,
        value: &mut f32,
        range: (f32, f32),
        width: i32,
        height: i32,
    ) -> SliderResponse {
        let width = width.max(1);
        let height = height.max(1);
        let control_size = Size {
            width: width as u32,
            height: height as u32,
        };
        let block_size = self.slider_block_size(label, control_size);
        let label_height = 8 * self.theme.text_scale as i32;
        let base = self.layout.cursor;
        let mut rect_origin = base;
        if !label.is_empty() {
            let _ = self.draw_text_single_line_clamped(
                base,
                label,
                control_size.width,
                self.theme.text,
                true,
            );
            rect_origin.y += label_height;
        }

        let rect = Rect {
            origin: rect_origin,
            size: control_size,
        };
        self.track_rect_internal(rect);
        let hovered = self.pointer_inside_clipped_rect(rect);
        if hovered {
            self.state.hot = Some(id);
        }

        let mut response = SliderResponse {
            hovered,
            active: self.state.active == Some(id),
            changed: false,
        };

        if hovered && self.mouse_pressed() {
            self.state.active = Some(id);
            response.active = true;
        }

        if self.state.active == Some(id) && self.input.mouse_released {
            self.state.active = None;
            response.active = false;
        }

        if self.state.active == Some(id) && self.input.mouse_down {
            let span = (range.1 - range.0).max(1.0e-6);
            let x = (self.input.pointer_pos.x - rect.origin.x) as f32;
            let t = (x / rect.size.width.max(1) as f32).clamp(0.0, 1.0);
            let new_value = range.0 + t * span;
            if (*value - new_value).abs() > f32::EPSILON {
                *value = new_value;
                response.changed = true;
            }
        }

        if hovered && self.input.wheel_delta != 0.0 {
            let span = (range.1 - range.0).max(1.0e-6);
            let step = 0.02 * span;
            let new_value =
                (*value + step * self.input.wheel_delta.signum()).clamp(range.0, range.1);
            if (*value - new_value).abs() > f32::EPSILON {
                *value = new_value;
                response.changed = true;
            }
        }

        let span = (range.1 - range.0).max(1.0e-6);
        let t = ((*value - range.0) / span).clamp(0.0, 1.0);
        let track_height = (height / 4).max(4);
        let track_y = rect.origin.y + (height - track_height) / 2;
        let track_rect = Rect {
            origin: Point {
                x: rect.origin.x,
                y: track_y,
            },
            size: Size {
                width: rect.size.width,
                height: track_height as u32,
            },
        };
        let fill_width = ((rect.size.width as f32) * t).round() as u32;
        let fill_rect = Rect {
            origin: track_rect.origin,
            size: Size {
                width: fill_width,
                height: track_rect.size.height,
            },
        };
        let fill = if response.active {
            self.theme.knob_active
        } else if hovered {
            self.theme.knob_hover
        } else {
            self.theme.knob_fill
        };
        self.fill_rect_clipped(track_rect, fill);
        self.stroke_rect_clipped(track_rect, 1, self.theme.knob_outline);
        self.fill_rect_clipped(fill_rect, self.theme.knob_indicator);

        let handle_x = rect.origin.x + (rect.size.width as f32 * t) as i32;
        let handle_center = Point {
            x: handle_x,
            y: rect.origin.y + height / 2,
        };
        let handle_radius = (height / 2).max(3);
        self.canvas
            .fill_circle(handle_center, handle_radius, self.theme.knob_indicator);

        self.layout.cursor.y = rect.origin.y + block_size.height as i32 + self.layout.spacing;
        response
    }

    /// Draw a horizontal slider with a stable key and a dynamic label.
    pub fn slider_with_key(
        &mut self,
        key: &str,
        label: &str,
        value: &mut f32,
        range: (f32, f32),
        width: i32,
        height: i32,
    ) -> SliderResponse {
        let id = WidgetId::from_label(key);
        self.slider(id, label, value, range, width, height)
    }

    /// Render a slider in a fixed rectangle without affecting surrounding layout.
    pub(crate) fn slider_in_rect(
        &mut self,
        id: WidgetId,
        label: &str,
        value: &mut f32,
        range: (f32, f32),
        control_size: Size,
        rect: Rect,
    ) -> SliderResponse {
        let previous = *self.layout;
        self.layout.cursor = rect.origin;
        let height = control_size.height.max(1) as i32;
        let response = {
            let mut response = SliderResponse::default();
            self.with_clip(rect, |ui| {
                response = ui.slider(
                    id,
                    label,
                    value,
                    range,
                    rect.size.width.max(1) as i32,
                    height,
                );
            });
            response
        };
        *self.layout = previous;
        response
    }

    /// Render a slider in a fixed rectangle with an explicit text scale.
    pub(crate) fn slider_in_rect_scaled(
        &mut self,
        id: WidgetId,
        label: &str,
        value: &mut f32,
        range: (f32, f32),
        control_size: Size,
        rect: Rect,
        text_scale: u32,
    ) -> SliderResponse {
        let previous = self.theme.text_scale;
        self.theme.text_scale = text_scale.max(1);
        let response = self.slider_in_rect(id, label, value, range, control_size, rect);
        self.theme.text_scale = previous;
        response
    }
}
