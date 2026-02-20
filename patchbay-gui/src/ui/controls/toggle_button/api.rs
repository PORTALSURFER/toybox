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
        _control_size: Size,
        rect: Rect,
    ) -> ToggleResponse {
        let previous = *self.layout;
        self.layout.cursor = rect.origin;
        let height = rect.size.height.max(1) as i32;
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
