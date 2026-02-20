impl<'a> Ui<'a> {
    /// Draw the knob vector command and both associated labels.
    fn draw_knob_visuals(
        &mut self,
        spec: KnobRenderSpec<'_>,
        geometry: KnobGeometry,
        response: KnobResponse,
        value: f32,
    ) {
        self.push_knob_vector_command(geometry, response, value, spec.range);
        self.draw_knob_name_label(spec.labels, geometry);
        self.draw_knob_value_label(spec.labels, geometry);
    }

    /// Queue the vector-scene knob primitive.
    fn push_knob_vector_command(
        &mut self,
        geometry: KnobGeometry,
        response: KnobResponse,
        value: f32,
        range: KnobRange,
    ) {
        let value_angle = Self::resolve_knob_value_angle(value, range);
        let fill = self.resolve_knob_fill(response);
        self.vector_commands.push(VectorCommand::Knob(KnobVisual {
            center: geometry.center,
            radius: geometry.radius,
            arc_radius: geometry.radius + 2,
            arc_thickness: 1.5,
            arc_start: 7.0 * std::f32::consts::PI / 4.0,
            arc_end: 5.0 * std::f32::consts::PI / 4.0,
            value_angle,
            fill,
            outline: self.theme.knob_outline,
            indicator: self.theme.knob_indicator,
        }));
    }

    /// Return the angular indicator position for a knob value.
    fn resolve_knob_value_angle(value: f32, range: KnobRange) -> f32 {
        let t = range.normalized(value);
        let arc_start = 7.0 * std::f32::consts::PI / 4.0;
        let arc_end = 5.0 * std::f32::consts::PI / 4.0;
        let arc_span = if arc_end < arc_start {
            arc_end + std::f32::consts::TAU - arc_start
        } else {
            arc_end - arc_start
        };
        arc_start + (1.0 - t) * arc_span
    }

    /// Resolve dial fill color from current interaction response.
    fn resolve_knob_fill(&self, response: KnobResponse) -> Color {
        if response.active {
            self.theme.knob_active
        } else if response.hovered {
            self.theme.knob_hover
        } else {
            self.theme.knob_fill
        }
    }

    /// Draw the normalized name label above the knob.
    fn draw_knob_name_label(&mut self, labels: KnobLabels<'_>, geometry: KnobGeometry) {
        let text = labels.normalized_name();
        if text.is_empty() {
            return;
        }
        let position = Point {
            x: geometry.block_rect.origin.x,
            y: geometry.block_rect.origin.y,
        };
        let _ = self.draw_text_single_line_hard_clamped_centered_on_char_size(
            position,
            geometry.center.x,
            &text,
            geometry.block_rect.size.width,
            self.theme.text,
            true,
        );
    }

    /// Draw the normalized value label below the knob.
    fn draw_knob_value_label(&mut self, labels: KnobLabels<'_>, geometry: KnobGeometry) {
        let text = labels.normalized_value();
        if text.is_empty() {
            return;
        }
        let position = Point {
            x: geometry.block_rect.origin.x,
            y: geometry.knob_rect.origin.y + geometry.knob_size + geometry.label_gap,
        };
        let _ = self.draw_text_single_line_hard_clamped_centered_on_char_size(
            position,
            geometry.center.x,
            &text,
            geometry.block_rect.size.width,
            self.theme.text,
            true,
        );
    }
}
