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
        config: SliderConfig,
    ) -> SliderResponse {
        self.slider_with_default(id, label, value, config, slider_default_value(config.range))
    }

    /// Draw a horizontal slider with an explicit double-click reset default value.
    pub fn slider_with_default(
        &mut self,
        id: WidgetId,
        label: &str,
        value: &mut f32,
        config: SliderConfig,
        default_value: f32,
    ) -> SliderResponse {
        let width = config.size.width.max(1) as i32;
        let height = config.size.height.max(1) as i32;
        let layout = self.resolve_slider_layout(label, width, height);
        let hovered = self.pointer_inside_clipped_rect(layout.rect);
        if hovered {
            self.state.hot = Some(id);
        }
        if hovered && self.input.mouse_double_clicked {
            let changed = self.apply_slider_double_click_reset(id, config.range, default_value, value);
            let response = SliderResponse {
                changed,
                hovered,
                active: false,
            };
            let visuals =
                self.resolve_slider_visuals(layout.rect, layout.height, config.range, *value, response);
            self.draw_slider_visuals(visuals);
            self.advance_slider_cursor(layout.rect, layout.block_size);
            return response;
        }
        let mut response = self.begin_slider_interaction(id, layout.rect);

        response.changed |= self.apply_slider_drag(id, layout.rect, config.range, value);
        response.changed |= self.apply_slider_wheel(response.hovered, config.range, value);

        let visuals = self.resolve_slider_visuals(layout.rect, layout.height, config.range, *value, response);
        self.draw_slider_visuals(visuals);
        self.advance_slider_cursor(layout.rect, layout.block_size);
        response
    }

    /// Draw a horizontal slider with a stable key and a dynamic label.
    pub fn slider_with_key(
        &mut self,
        key: &str,
        label: &str,
        value: &mut f32,
        config: SliderConfig,
    ) -> SliderResponse {
        let id = WidgetId::from_label(key);
        self.slider(id, label, value, config)
    }

    /// Draw a horizontal slider with a stable key and explicit reset default.
    pub fn slider_with_key_default(
        &mut self,
        key: &str,
        label: &str,
        value: &mut f32,
        config: SliderConfig,
        default_value: f32,
    ) -> SliderResponse {
        let id = WidgetId::from_label(key);
        self.slider_with_default(id, label, value, config, default_value)
    }

    /// Render a slider in a fixed rectangle without affecting surrounding layout.
    pub(crate) fn slider_in_rect(
        &mut self,
        value: &mut f32,
        request: SliderRectRenderRequest<'_>,
    ) -> SliderResponse {
        let previous = *self.layout;
        self.layout.cursor = request.rect.origin;
        let previous_text_scale = self.theme.text_scale;
        self.theme.text_scale = request.text_scale;
        let response = {
            let mut response = SliderResponse::default();
            self.with_clip(request.rect, |ui| {
                response = ui.slider_with_default(
                    request.id,
                    request.label,
                    value,
                    SliderConfig {
                        range: request.range,
                        size: Size {
                            width: request.rect.size.width.max(1),
                            height: request.rect.size.height.max(1),
                        },
                    },
                    request.default_value,
                );
            });
            response
        };
        self.theme.text_scale = previous_text_scale;
        *self.layout = previous;
        response
    }

    /// Render a slider in a fixed rectangle with an explicit text scale.
    pub(crate) fn slider_in_rect_scaled(
        &mut self,
        value: &mut f32,
        request: SliderRectRenderRequest<'_>,
    ) -> SliderResponse {
        self.slider_in_rect(value, request)
    }
}
