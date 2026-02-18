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
        let spec = KnobRenderSpec::from_args(id, name_label, value_label, range);
        self.render_knob_from_spec(spec, value)
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
        value: &mut f32,
        request: KnobRectRenderRequest<'_>,
    ) -> KnobResponse {
        let spec = KnobRenderSpec {
            id: request.id,
            labels: request.labels,
            range: KnobRange::from_tuple(request.range),
        };
        let rect_spec = KnobRectSpec::new(request.rect, request.desired_diameter);
        let previous_text_scale = self.theme.text_scale;
        self.theme.text_scale = request.text_scale;
        let response = self.render_knob_in_rect_from_spec(spec, value, rect_spec);
        self.theme.text_scale = previous_text_scale;
        response
    }

    /// Render a knob in a fixed rectangle with an explicit text scale.
    pub(crate) fn knob_with_labels_in_rect_scaled(
        &mut self,
        value: &mut f32,
        request: KnobRectRenderRequest<'_>,
    ) -> KnobResponse {
        self.knob_with_labels_in_rect(value, request)
    }

    /// Render a knob from normalized spec inputs at the current cursor.
    fn render_knob_from_spec(&mut self, spec: KnobRenderSpec<'_>, value: &mut f32) -> KnobResponse {
        let geometry = self.resolve_knob_geometry_for_cursor(spec.labels);
        self.draw_knob_bounds(geometry);
        let response = self.resolve_knob_interaction(spec, geometry, value);
        self.draw_knob_visuals(spec, geometry, response, *value);
        self.advance_layout_after_knob(geometry);
        response
    }

    /// Render a knob from normalized spec inputs inside a fixed clip rectangle.
    fn render_knob_in_rect_from_spec(
        &mut self,
        spec: KnobRenderSpec<'_>,
        value: &mut f32,
        rect_spec: KnobRectSpec,
    ) -> KnobResponse {
        let previous_layout = *self.layout;
        self.layout.knob_size = self.resolve_knob_size_for_rect(
            rect_spec.rect,
            rect_spec.desired_diameter,
        );
        let block_size = knob_block_size_for_diameter(
            self.layout.knob_size.max(1) as u32,
            self.theme.text_scale,
        );
        let offset_x = ((rect_spec.rect.size.width as i32 - block_size.width as i32) / 2).max(0);
        let offset_y = ((rect_spec.rect.size.height as i32 - block_size.height as i32) / 2).max(0);
        self.layout.cursor = Point {
            x: rect_spec.rect.origin.x + offset_x,
            y: rect_spec.rect.origin.y + offset_y,
        };
        let mut response = KnobResponse::default();
        self.with_clip(rect_spec.rect, |ui| {
            response = ui.render_knob_from_spec(spec, value);
        });
        *self.layout = previous_layout;
        response
    }
}
