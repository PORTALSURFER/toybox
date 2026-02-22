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
        self.toggle_styled(
            id,
            label,
            value,
            width,
            height,
            ControlVisualState::default(),
        )
    }

    /// Draw a toggle switch with optional role-driven style overrides.
    fn toggle_styled(
        &mut self,
        id: WidgetId,
        label: &str,
        value: &mut bool,
        width: i32,
        height: i32,
        style: ControlVisualState,
    ) -> ToggleResponse {
        let layout = self.resolve_toggle_layout(label, width, height);
        let mut response = if style.disabled {
            ToggleResponse::default()
        } else {
            self.begin_toggle_interaction(id, layout.rect)
        };
        if !style.disabled {
            response.changed = self.apply_toggle_press(response.hovered, value);
        }
        let fill = self.resolve_toggle_fill(*value, response.hovered, style.variants, style.disabled);
        self.draw_toggle_visuals(layout.rect, fill, *value, style.variants, style.focused);
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
        self.button_styled(id, label, width, height, ControlVisualState::default())
    }

    /// Draw a button with optional role-driven style overrides.
    fn button_styled(
        &mut self,
        id: WidgetId,
        label: &str,
        width: i32,
        height: i32,
        style: ControlVisualState,
    ) -> ButtonResponse {
        let rect = self.resolve_button_rect(width, height);
        let response = if style.disabled {
            ButtonResponse::default()
        } else {
            self.resolve_button_interaction(id, rect)
        };
        let active = !style.disabled && response.hovered && self.input.mouse_down;
        self.draw_button_visuals(
            rect,
            label,
            response.hovered,
            active,
            style,
        );
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

    /// Render a toggle in a fixed rectangle with style and state overrides.
    pub(crate) fn toggle_in_rect_styled(
        &mut self,
        request: ToggleRectRenderRequest<'_>,
    ) -> ToggleResponse {
        let previous = *self.layout;
        self.layout.cursor = request.rect.origin;
        let height = request.rect.size.height.max(1) as i32;
        let width = request.rect.size.width.max(1) as i32;
        let previous_text_scale = self.theme.text_scale;
        self.theme.text_scale = request.text_scale.max(1);
        let response = {
            let mut response = ToggleResponse::default();
            self.with_clip(request.rect, |ui| {
                let _ = request.control_size;
                response = ui.toggle_styled(
                    request.id,
                    request.label,
                    request.value,
                    width,
                    height,
                    ControlVisualState {
                        variants: request.color_variants,
                        disabled: request.disabled,
                        focused: request.focused,
                    },
                );
            });
            response
        };
        self.theme.text_scale = previous_text_scale;
        *self.layout = previous;
        response
    }

    /// Render a button in a fixed rectangle with style and state overrides.
    pub(crate) fn button_in_rect_styled(
        &mut self,
        request: ButtonRectRenderRequest<'_>,
    ) -> ButtonResponse {
        let previous = *self.layout;
        self.layout.cursor = request.rect.origin;
        let previous_text_scale = self.theme.text_scale;
        self.theme.text_scale = request.text_scale.max(1);
        let response = {
            let mut response = ButtonResponse::default();
            self.with_clip(request.rect, |ui| {
                let _ = request.control_size;
                response = ui.button_styled(
                    request.id,
                    request.label,
                    request.rect.size.width.max(1) as i32,
                    request.rect.size.height.max(1) as i32,
                    ControlVisualState {
                        variants: request.color_variants,
                        disabled: request.disabled,
                        focused: request.focused,
                    },
                );
            });
            response
        };
        self.theme.text_scale = previous_text_scale;
        *self.layout = previous;
        response
    }
}
