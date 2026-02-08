impl<'a> Ui<'a> {

    /// Draw a toggle switch with the given label and value.
    pub fn toggle(
        &mut self,
        id: WidgetId,
        label: &str,
        value: &mut bool,
        width: i32,
        height: i32,
    ) -> ToggleResponse {
        let width = width.max(1);
        let height = height.max(1);
        let control_size = Size {
            width: width as u32,
            height: height as u32,
        };
        let block_size = self.toggle_block_size(label, control_size);
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
        let mut response = ToggleResponse {
            hovered,
            changed: false,
        };
        if hovered && self.mouse_pressed() {
            *value = !*value;
            response.changed = true;
        }
        let fill = if *value {
            self.theme.knob_indicator
        } else if hovered {
            self.theme.knob_hover
        } else {
            self.theme.knob_fill
        };
        self.fill_rect_clipped(rect, fill);
        self.stroke_rect_clipped(rect, 1, self.theme.knob_outline);

        let thumb_radius = (height / 2).max(3);
        let thumb_x = if *value {
            rect.origin.x + width - thumb_radius
        } else {
            rect.origin.x + thumb_radius
        };
        let thumb_center = Point {
            x: thumb_x,
            y: rect.origin.y + height / 2,
        };
        self.canvas
            .fill_circle(thumb_center, thumb_radius, self.theme.knob_outline);

        self.layout.cursor.y = rect.origin.y + block_size.height as i32 + self.layout.spacing;
        response
    }

    /// Draw a toggle switch with a stable key and a dynamic label.
    pub fn toggle_with_key(
        &mut self,
        key: &str,
        label: &str,
        value: &mut bool,
        width: i32,
        height: i32,
    ) -> ToggleResponse {
        let id = WidgetId::from_label(key);
        self.toggle(id, label, value, width, height)
    }

    /// Draw a button with the given label.
    pub fn button(&mut self, id: WidgetId, label: &str, width: i32, height: i32) -> ButtonResponse {
        let width = width.max(1);
        let height = height.max(1);
        let control_size = Size {
            width: width as u32,
            height: height as u32,
        };
        let rect = Rect {
            origin: self.layout.cursor,
            size: control_size,
        };
        self.track_rect_internal(rect);
        let hovered = self.pointer_inside_clipped_rect(rect);
        if hovered {
            self.state.hot = Some(id);
        }
        let mut response = ButtonResponse {
            hovered,
            clicked: false,
        };
        if hovered && self.mouse_pressed() {
            response.clicked = true;
        }
        let fill = if hovered {
            self.theme.knob_hover
        } else {
            self.theme.knob_fill
        };
        self.fill_rect_clipped(rect, fill);
        self.stroke_rect_clipped(rect, 1, self.theme.knob_outline);
        let text_pos = Point {
            x: rect.origin.x + 4,
            y: rect.origin.y + (height - (7 * self.theme.text_scale as i32)) / 2,
        };
        let _ = self.draw_text_single_line_clamped(
            text_pos,
            label,
            rect.size.width.saturating_sub(8),
            self.theme.text,
            false,
        );

        self.layout.cursor.y = rect.origin.y + height + self.layout.spacing;
        response
    }

    /// Draw a button with a stable key and a dynamic label.
    pub fn button_with_key(
        &mut self,
        key: &str,
        label: &str,
        width: i32,
        height: i32,
    ) -> ButtonResponse {
        let id = WidgetId::from_label(key);
        self.button(id, label, width, height)
    }

    /// Render a toggle in a fixed rectangle without affecting surrounding layout.
    pub(crate) fn toggle_in_rect(
        &mut self,
        id: WidgetId,
        label: &str,
        value: &mut bool,
        control_size: Size,
        rect: Rect,
    ) -> ToggleResponse {
        let previous = *self.layout;
        self.layout.cursor = rect.origin;
        let height = control_size.height.max(1) as i32;
        let response = {
            let mut response = ToggleResponse::default();
            self.with_clip(rect, |ui| {
                response = ui.toggle(id, label, value, rect.size.width.max(1) as i32, height);
            });
            response
        };
        *self.layout = previous;
        response
    }

    /// Render a toggle in a fixed rectangle with an explicit text scale.
    pub(crate) fn toggle_in_rect_scaled(
        &mut self,
        id: WidgetId,
        label: &str,
        value: &mut bool,
        control_size: Size,
        rect: Rect,
        text_scale: u32,
    ) -> ToggleResponse {
        let previous = self.theme.text_scale;
        self.theme.text_scale = text_scale.max(1);
        let response = self.toggle_in_rect(id, label, value, control_size, rect);
        self.theme.text_scale = previous;
        response
    }

    /// Render a button in a fixed rectangle without affecting surrounding layout.
    pub(crate) fn button_in_rect(
        &mut self,
        id: WidgetId,
        label: &str,
        _control_size: Size,
        rect: Rect,
    ) -> ButtonResponse {
        let previous = *self.layout;
        self.layout.cursor = rect.origin;
        let response = {
            let mut response = ButtonResponse::default();
            self.with_clip(rect, |ui| {
                response = ui.button(
                    id,
                    label,
                    rect.size.width.max(1) as i32,
                    rect.size.height.max(1) as i32,
                );
            });
            response
        };
        *self.layout = previous;
        response
    }

    /// Render a button in a fixed rectangle with an explicit text scale.
    pub(crate) fn button_in_rect_scaled(
        &mut self,
        id: WidgetId,
        label: &str,
        control_size: Size,
        rect: Rect,
        text_scale: u32,
    ) -> ButtonResponse {
        let previous = self.theme.text_scale;
        self.theme.text_scale = text_scale.max(1);
        let response = self.button_in_rect(id, label, control_size, rect);
        self.theme.text_scale = previous;
        response
    }
}
