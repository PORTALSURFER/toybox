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
        let layout = self.resolve_toggle_layout(label, width, height);
        let mut response = self.begin_toggle_interaction(id, layout.rect);
        response.changed = self.apply_toggle_press(response.hovered, value);
        let fill = self.resolve_toggle_fill(*value, response.hovered);
        self.draw_toggle_visuals(layout.rect, fill, *value);
        self.advance_toggle_cursor(layout.block_size, layout.rect.origin.y);
        response
    }

    /// Resolve layout geometry for a toggle and draw its optional label.
    fn resolve_toggle_layout(&mut self, label: &str, width: i32, height: i32) -> ToggleLayoutResolved {
        let width = width.max(1);
        let height = height.max(1);
        let control_size = Size {
            width: width as u32,
            height: height as u32,
        };
        let block_size = self.toggle_block_size(label, control_size);
        let origin = self.draw_toggle_label(label, control_size);
        let rect = Rect {
            origin,
            size: control_size,
        };
        self.track_rect_internal(rect);
        ToggleLayoutResolved { block_size, rect }
    }

    /// Draw the toggle label and return the control origin point.
    fn draw_toggle_label(&mut self, label: &str, control_size: Size) -> Point {
        let base = self.layout.cursor;
        if label.is_empty() {
            return base;
        }
        let _ = self.draw_text_single_line_clamped(
            base,
            label,
            control_size.width,
            self.theme.text,
            true,
        );
        Point {
            x: base.x,
            y: base.y + (8 * self.theme.text_scale as i32),
        }
    }

    /// Resolve hover/hot interaction state for toggle controls.
    fn begin_toggle_interaction(&mut self, id: WidgetId, rect: Rect) -> ToggleResponse {
        let hovered = self.pointer_inside_clipped_rect(rect);
        if hovered {
            self.state.hot = Some(id);
        }
        ToggleResponse {
            hovered,
            changed: false,
        }
    }

    /// Apply press interaction and update the toggle value.
    fn apply_toggle_press(&mut self, hovered: bool, value: &mut bool) -> bool {
        if !(hovered && self.mouse_pressed()) {
            return false;
        }
        *value = !*value;
        true
    }

    /// Resolve toggle fill color from value and hover state.
    fn resolve_toggle_fill(&self, value: bool, hovered: bool) -> Color {
        if value {
            self.theme.knob_indicator
        } else if hovered {
            self.theme.knob_hover
        } else {
            self.theme.knob_fill
        }
    }

    /// Draw toggle body and thumb.
    fn draw_toggle_visuals(&mut self, rect: Rect, fill: Color, value: bool) {
        self.fill_rect_clipped(rect, fill);
        self.stroke_rect_clipped(rect, 1, self.theme.knob_outline);
        let width = rect.size.width as i32;
        let height = rect.size.height as i32;
        let thumb_radius = (height / 2).max(3);
        let thumb_x = if value {
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
    }

    /// Advance the layout cursor after rendering a toggle.
    fn advance_toggle_cursor(&mut self, block_size: Size, rect_y: i32) {
        self.layout.cursor.y = rect_y + block_size.height as i32 + self.layout.spacing;
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
        let rect = self.resolve_button_rect(width, height);
        let response = self.resolve_button_interaction(id, rect);
        self.draw_button_visuals(rect, label, response.hovered);
        self.advance_button_cursor(rect);
        response
    }

    /// Resolve the button rectangle from current layout cursor and requested size.
    fn resolve_button_rect(&mut self, width: i32, height: i32) -> Rect {
        let control_size = Size {
            width: width.max(1) as u32,
            height: height.max(1) as u32,
        };
        let rect = Rect {
            origin: self.layout.cursor,
            size: control_size,
        };
        self.track_rect_internal(rect);
        rect
    }

    /// Resolve hover/click state for a button interaction.
    fn resolve_button_interaction(&mut self, id: WidgetId, rect: Rect) -> ButtonResponse {
        let hovered = self.pointer_inside_clipped_rect(rect);
        if hovered {
            self.state.hot = Some(id);
        }
        ButtonResponse {
            hovered,
            clicked: hovered && self.mouse_pressed(),
        }
    }

    /// Draw button body, outline, and clamped label text.
    fn draw_button_visuals(&mut self, rect: Rect, label: &str, hovered: bool) {
        let fill = if hovered {
            self.theme.knob_hover
        } else {
            self.theme.knob_fill
        };
        self.fill_rect_clipped(rect, fill);
        self.stroke_rect_clipped(rect, 1, self.theme.knob_outline);
        let text_pos = Point {
            x: rect.origin.x + 4,
            y: rect.origin.y + (rect.size.height as i32 - (7 * self.theme.text_scale as i32)) / 2,
        };
        let _ = self.draw_text_single_line_clamped(
            text_pos,
            label,
            rect.size.width.saturating_sub(8),
            self.theme.text,
            false,
        );
    }

    /// Advance the vertical layout cursor after drawing a button.
    fn advance_button_cursor(&mut self, rect: Rect) {
        self.layout.cursor.y = rect.origin.y + rect.size.height as i32 + self.layout.spacing;
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

/// Resolved layout geometry for a toggle draw pass.
#[derive(Clone, Copy)]
struct ToggleLayoutResolved {
    /// Total vertical block consumed by toggle + optional label.
    block_size: Size,
    /// Toggle control rectangle.
    rect: Rect,
}
