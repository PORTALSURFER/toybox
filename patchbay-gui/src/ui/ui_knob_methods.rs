impl<'a> Ui<'a> {

    /// Draw a knob with the given name label and value.
    ///
    /// The provided `id` must be stable across frames. If the label changes
    /// while dragging, use [`Ui::knob_with_key`] to provide a separate stable
    /// identifier.
    pub fn knob(
        &mut self,
        id: WidgetId,
        label: &str,
        value: &mut f32,
        range: (f32, f32),
    ) -> KnobResponse {
        let value_label = Self::format_knob_value(*value);
        self.knob_with_labels(id, label, &value_label, value, range)
    }

    /// Draw a knob with a name label above and a value label below.
    ///
    /// Knob labels are hard-clamped to the knob width (without ellipsis) and
    /// horizontally centered. Name labels are normalized to uppercase, while
    /// value labels are lowercased when they contain alphabetic text.
    pub fn knob_with_labels(
        &mut self,
        id: WidgetId,
        name_label: &str,
        value_label: &str,
        value: &mut f32,
        range: (f32, f32),
    ) -> KnobResponse {
        let knob_size = self.layout.knob_size.max(1);
        let block_size = self.knob_block_size(name_label, value_label);
        let label_height = 8 * self.theme.text_scale as i32;
        let label_gap = 4 * self.theme.text_scale as i32;
        let block_rect = Rect {
            origin: self.layout.cursor,
            size: block_size,
        };
        self.stroke_rect_clipped(block_rect, 1, self.theme.knob_outline);
        self.track_rect_internal(block_rect);
        let knob_x_offset = ((block_size.width as i32 - knob_size) / 2).max(0);
        let knob_origin = Point {
            x: self.layout.cursor.x + knob_x_offset,
            y: self.layout.cursor.y + label_height + label_gap,
        };
        let knob_rect = Rect {
            origin: knob_origin,
            size: Size {
                width: knob_size as u32,
                height: knob_size as u32,
            },
        };
        let hit_rect = Rect {
            origin: Point {
                x: knob_rect.origin.x - KNOB_BLOCK_SIDE_PADDING,
                y: knob_rect.origin.y - KNOB_BLOCK_SIDE_PADDING,
            },
            size: Size {
                width: (knob_size + KNOB_BLOCK_SIDE_PADDING * 2).max(1) as u32,
                height: (knob_size + KNOB_BLOCK_SIDE_PADDING * 2).max(1) as u32,
            },
        };
        self.stroke_rect_clipped(hit_rect, 1, self.theme.knob_outline);
        self.track_rect_internal(hit_rect);
        let center = Point {
            x: knob_rect.origin.x + knob_size / 2,
            y: knob_rect.origin.y + knob_size / 2,
        };
        let radius = (knob_size / 2 - 4).max(1);
        let hovered = self.pointer_inside_clipped_rect(hit_rect);
        if hovered {
            self.state.hot = Some(id);
        }

        let mut response = KnobResponse {
            hovered,
            active: self.state.active == Some(id),
            changed: false,
        };

        if hovered && self.mouse_pressed() {
            self.state.active = Some(id);
            self.state.drag_start = Some(self.input.pointer_pos);
            self.state.drag_value = *value;
            response.active = true;
        }

        if self.state.active == Some(id) && self.input.mouse_released {
            self.state.active = None;
            self.state.drag_start = None;
            response.active = false;
        }

        if self.state.active == Some(id)
            && self.input.mouse_down
            && let Some(start) = self.state.drag_start
        {
            let dy = (self.input.pointer_pos.y - start.y) as f32;
            let delta = -dy * 0.005 * (range.1 - range.0);
            let new_value = (self.state.drag_value + delta).clamp(range.0, range.1);
            if (*value - new_value).abs() > f32::EPSILON {
                *value = new_value;
                response.changed = true;
            }
        }

        if hovered && self.input.wheel_delta != 0.0 {
            let step = 0.02 * (range.1 - range.0);
            let new_value =
                (*value - step * self.input.wheel_delta.signum()).clamp(range.0, range.1);
            if (*value - new_value).abs() > f32::EPSILON {
                *value = new_value;
                response.changed = true;
            }
        }

        let t = ((*value - range.0) / (range.1 - range.0).max(1.0e-6)).clamp(0.0, 1.0);
        let arc_start = 7.0 * std::f32::consts::PI / 4.0;
        let arc_end = 5.0 * std::f32::consts::PI / 4.0;
        let arc_span = if arc_end < arc_start {
            arc_end + std::f32::consts::TAU - arc_start
        } else {
            arc_end - arc_start
        };
        // Keep the existing rotation path but map high values to the right side.
        let angle = arc_start + (1.0 - t) * arc_span;
        let fill = if response.active {
            self.theme.knob_active
        } else if hovered {
            self.theme.knob_hover
        } else {
            self.theme.knob_fill
        };

        let arc_radius = radius + 6;
        let arc_thickness = 3;
        self.vector_commands.push(VectorCommand::Knob(KnobVisual {
            center,
            radius,
            arc_radius,
            arc_thickness,
            arc_start,
            arc_end,
            value_angle: angle,
            fill,
            outline: self.theme.knob_outline,
            indicator: self.theme.knob_indicator,
        }));

        let name_pos = Point {
            x: self.layout.cursor.x,
            y: self.layout.cursor.y,
        };
        let normalized_name = normalize_knob_name_label(name_label);
        if !normalized_name.is_empty() {
            let _ = self.draw_text_single_line_hard_clamped_centered_on_x(
                name_pos,
                center.x,
                &normalized_name,
                block_size.width,
                self.theme.text,
                true,
            );
        }

        let value_pos = Point {
            x: self.layout.cursor.x,
            y: knob_rect.origin.y + knob_size + label_gap,
        };
        let normalized_value = normalize_knob_value_label(value_label);
        if !normalized_value.is_empty() {
            let _ = self.draw_text_single_line_hard_clamped_centered_on_x(
                value_pos,
                center.x,
                &normalized_value,
                block_size.width,
                self.theme.text,
                true,
            );
        }

        let block_height = block_size.height as i32;
        self.layout.cursor.y += block_height + self.layout.spacing;
        response
    }

    /// Draw a knob with a stable key and a potentially dynamic name label.
    ///
    /// The `key` should be a stable identifier across frames (for example,
    /// `"attack"`). The `label` can change to include formatted values without
    /// breaking drag tracking.
    ///
    /// # Example
    /// ```ignore
    /// # use patchbay_gui::{Canvas, InputState, Layout, Theme, Ui, UiState};
    /// let mut canvas = Canvas::new(200, 200);
    /// let mut layout = Layout::default();
    /// let theme = Theme::default();
    /// let mut ui_state = UiState::default();
    /// let input = InputState::default();
    /// let mut value = 0.5;
    /// let mut ui = Ui::new(&mut canvas, &input, &mut ui_state, &mut layout, &theme);
    /// ui.knob_with_key("attack", "Attack 0.50s", &mut value, (0.0, 1.0));
    /// ```
    pub fn knob_with_key(
        &mut self,
        key: &str,
        label: &str,
        value: &mut f32,
        range: (f32, f32),
    ) -> KnobResponse {
        let id = WidgetId::from_label(key);
        self.knob(id, label, value, range)
    }

    /// Draw a knob with explicit name and value labels and a stable key.
    pub fn knob_with_key_labels(
        &mut self,
        key: &str,
        name_label: &str,
        value_label: &str,
        value: &mut f32,
        range: (f32, f32),
    ) -> KnobResponse {
        let id = WidgetId::from_label(key);
        self.knob_with_labels(id, name_label, value_label, value, range)
    }

    /// Render a knob in a fixed rectangle without affecting surrounding layout.
    pub(crate) fn knob_with_labels_in_rect(
        &mut self,
        id: WidgetId,
        name_label: &str,
        value_label: &str,
        value: &mut f32,
        range: (f32, f32),
        desired_diameter: u32,
        rect: Rect,
    ) -> KnobResponse {
        let previous = *self.layout;
        let label_height = knob_label_height(self.theme.text_scale) as i32;
        let label_gap = knob_label_gap(self.theme.text_scale) as i32;
        let side_padding = KNOB_BLOCK_SIDE_PADDING.max(0);
        // Keep knob rendering bounded to the resolved declarative diameter so
        // measured and rendered footprints remain consistent.
        let available_height = (rect.size.height as i32 - label_height * 2 - label_gap * 2).max(1);
        let available_width = (rect.size.width as i32 - side_padding * 2).max(1);
        let knob_size = (desired_diameter.max(1) as i32)
            .min(available_width)
            .min(available_height)
            .max(1);
        self.layout.cursor = rect.origin;
        self.layout.knob_size = knob_size;
        let response = {
            let mut response = KnobResponse::default();
            self.with_clip(rect, |ui| {
                response = ui.knob_with_labels(id, name_label, value_label, value, range);
            });
            response
        };
        *self.layout = previous;
        response
    }

    /// Render a knob in a fixed rectangle with an explicit text scale.
    pub(crate) fn knob_with_labels_in_rect_scaled(
        &mut self,
        id: WidgetId,
        name_label: &str,
        value_label: &str,
        value: &mut f32,
        range: (f32, f32),
        desired_diameter: u32,
        rect: Rect,
        text_scale: u32,
    ) -> KnobResponse {
        let previous = self.theme.text_scale;
        self.theme.text_scale = text_scale.max(1);
        let response = self.knob_with_labels_in_rect(
            id,
            name_label,
            value_label,
            value,
            range,
            desired_diameter,
            rect,
        );
        self.theme.text_scale = previous;
        response
    }
}
