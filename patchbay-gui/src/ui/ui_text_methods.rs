impl<'a> Ui<'a> {

    /// Draw a label at the given position.
    pub fn text(&mut self, position: Point, text: &str) {
        self.draw_text_internal(position, text, self.theme.text, self.theme.text_scale);
        let size = text_size(text, self.theme.text_scale);
        self.track_rect_internal(Rect {
            origin: position,
            size,
        });
    }

    /// Draw a label at the given position with a custom color.
    pub fn text_with_color(&mut self, position: Point, text: &str, color: Color) {
        self.draw_text_internal(position, text, color, self.theme.text_scale);
        let size = text_size(text, self.theme.text_scale);
        self.track_rect_internal(Rect {
            origin: position,
            size,
        });
    }

    /// Draw text with an explicit scale value.
    pub(crate) fn text_scaled_with_color(
        &mut self,
        position: Point,
        text: &str,
        color: Color,
        scale: u32,
    ) {
        self.draw_text_internal(position, text, color, scale);
    }

    /// Draw a single-line label with ellipsis truncation.
    fn draw_text_single_line_clamped(
        &mut self,
        origin: Point,
        text: &str,
        max_width: u32,
        color: Color,
        track_bounds: bool,
    ) -> Size {
        let fitted = fit_text_single_line_ellipsis(text, max_width, self.theme.text_scale);
        if fitted.is_empty() {
            return Size {
                width: 0,
                height: 0,
            };
        }
        self.draw_text_internal(origin, &fitted, color, self.theme.text_scale);
        let size = text_size(&fitted, self.theme.text_scale);
        if track_bounds {
            self.track_rect_internal(Rect { origin, size });
        }
        size
    }

    /// Draw a single-line label hard-clamped to width and height bounds.
    fn draw_text_single_line_hard_clamped(
        &mut self,
        origin: Point,
        text: &str,
        max_width: u32,
        max_height: u32,
        color: Color,
        track_bounds: bool,
    ) -> Size {
        let line_height = 8 * self.theme.text_scale.max(1);
        if max_height < line_height {
            return Size {
                width: 0,
                height: 0,
            };
        }

        let fitted = fit_text_single_line_hard_clamp(text, max_width, self.theme.text_scale);
        if fitted.is_empty() {
            return Size {
                width: 0,
                height: 0,
            };
        }
        self.draw_text_internal(origin, &fitted, color, self.theme.text_scale);
        let size = text_size(&fitted, self.theme.text_scale);
        if track_bounds {
            self.track_rect_internal(Rect { origin, size });
        }
        size
    }

    /// Draw centered single-line text hard-clamped to width bounds.
    fn draw_text_single_line_hard_clamped_centered_on_x(
        &mut self,
        origin: Point,
        center_x: i32,
        text: &str,
        max_width: u32,
        color: Color,
        track_bounds: bool,
    ) -> Size {
        let fitted = fit_text_single_line_hard_clamp(text, max_width, self.theme.text_scale);
        if fitted.is_empty() {
            return Size {
                width: 0,
                height: 0,
            };
        }
        let size = text_size(&fitted, self.theme.text_scale);
        let centered_origin = Point {
            x: centered_text_origin_on_x(origin.x, max_width, size.width, center_x),
            y: origin.y,
        };
        self.draw_text_internal(centered_origin, &fitted, color, self.theme.text_scale);
        if track_bounds {
            self.track_rect_internal(Rect {
                origin: centered_origin,
                size,
            });
        }
        size
    }

    /// Draw bounded single-line text in a rect with hard clipping semantics.
    pub(crate) fn text_single_line_hard_clamped_in_rect(
        &mut self,
        rect: Rect,
        text: &str,
        color: Color,
    ) -> Size {
        self.draw_text_single_line_hard_clamped(
            rect.origin,
            text,
            rect.size.width,
            rect.size.height,
            color,
            true,
        )
    }

    /// Draw bounded single-line text in a rect with an explicit text scale.
    pub(crate) fn text_single_line_hard_clamped_in_rect_scaled(
        &mut self,
        rect: Rect,
        text: &str,
        color: Color,
        text_scale: u32,
    ) -> Size {
        let previous = self.theme.text_scale;
        self.theme.text_scale = text_scale.max(1);
        let rendered = self.text_single_line_hard_clamped_in_rect(rect, text, color);
        self.theme.text_scale = previous;
        rendered
    }

    /// Draw a label at the current cursor and advance the cursor.
    pub fn label(&mut self, text: &str) {
        let pos = self.layout.cursor;
        let line_height = 8 * self.theme.text_scale as i32;
        self.draw_text_internal(pos, text, self.theme.text, self.theme.text_scale);
        let size = text_size(text, self.theme.text_scale);
        self.track_rect_internal(Rect { origin: pos, size });
        self.layout.cursor.y += line_height + self.layout.spacing;
    }

    /// Format a knob value using compact decimal precision.
    fn format_knob_value(value: f32) -> String {
        let mut text = format!("{value:.2}");
        if text.contains('.') {
            while text.ends_with('0') {
                text.pop();
            }
            if text.ends_with('.') {
                text.pop();
            }
        }
        if text == "-0" {
            text = "0".to_string();
        }
        text
    }
}
