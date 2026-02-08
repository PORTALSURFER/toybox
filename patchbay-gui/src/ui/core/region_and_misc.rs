impl<'a> Ui<'a> {

    /// Create an interactive region for custom drawing.
    ///
    /// Use this to capture hover/drag interactions over arbitrary rectangles.
    /// The `key` must be stable across frames.
    pub fn region_with_key(&mut self, key: &str, rect: Rect) -> RegionResponse {
        let id = WidgetId::from_label(key);
        let hovered = self.pointer_inside_clipped_rect(rect);
        let local_pointer = local_pointer_in_rect(self.input.pointer_pos, rect);
        let raw_local_pointer = raw_local_pointer_in_rect(self.input.pointer_pos, rect);
        if hovered {
            self.state.hot = Some(id);
        }

        let mut response = RegionResponse {
            hovered,
            local_pointer,
            raw_local_pointer,
            alt_down: self.input.alt_down,
            active: self.state.active == Some(id),
            pressed: false,
            released: false,
            dragged: false,
            secondary_clicked: false,
            double_clicked: false,
        };

        if hovered && self.mouse_pressed() {
            self.state.active = Some(id);
            self.state.drag_start = Some(self.input.pointer_pos);
            response.active = true;
            response.pressed = true;
        }

        if self.state.active == Some(id) && self.input.mouse_released {
            self.state.active = None;
            self.state.drag_start = None;
            response.active = false;
            response.released = true;
        }

        if self.state.active == Some(id) && self.input.mouse_down {
            response.dragged = true;
        }

        if hovered && self.input.mouse_secondary_pressed {
            response.secondary_clicked = true;
        }

        if hovered && self.input.mouse_double_clicked {
            response.double_clicked = true;
        }

        response
    }

    /// Clear the background with the theme color.
    pub fn clear(&mut self) {
        self.canvas.clear(self.theme.background);
    }

    /// Draw a non-interactive indicator cell.
    ///
    /// This is useful for sequencer step lights or other simple state displays.
    pub fn indicator(&mut self, rect: Rect, active: bool) {
        let fill = if active {
            self.theme.knob_indicator
        } else {
            self.theme.knob_fill
        };
        self.fill_rect_clipped(rect, fill);
        self.stroke_rect_clipped(rect, 1, self.theme.knob_outline);
        self.track_rect_internal(rect);
    }
}
