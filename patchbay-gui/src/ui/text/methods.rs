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
        let clip_rect = Rect {
            origin,
            size: Size {
                width: max_width,
                height: text_line_height(self.theme.text_scale.max(1)),
            },
        };
        self.with_clip(clip_rect, |ui| {
            ui.draw_text_internal(origin, &fitted, color, ui.theme.text_scale);
        });
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
        let line_height = text_line_height(self.theme.text_scale.max(1));
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
        let clip_rect = Rect {
            origin,
            size: Size {
                width: max_width,
                height: max_height,
            },
        };
        self.with_clip(clip_rect, |ui| {
            ui.draw_text_internal(origin, &fitted, color, ui.theme.text_scale);
        });
        let size = text_size(&fitted, self.theme.text_scale);
        if track_bounds {
            self.track_rect_internal(Rect { origin, size });
        }
        size
    }

    /// Draw centered single-line hard-clamped text using character width only.
    fn draw_text_single_line_hard_clamped_centered_on_char_size(
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
        let (span_left, char_size) = glyph_ink_span(&fitted, self.theme.text_scale);
        let centered_origin = Point {
            x: if span_left == 0 {
                centered_text_origin_on_x(origin.x, max_width, char_size, center_x)
            } else {
                centered_text_origin_on_span(origin.x, max_width, span_left, char_size, center_x)
            },
            y: origin.y,
        };
        if self.vector_text_enabled {
            self.vector_commands.push(VectorCommand::CenteredText {
                left_bound: origin.x,
                max_width,
                target_center_x: center_x,
                origin_y: origin.y,
                text: fitted.clone(),
                color,
                scale: self.theme.text_scale,
            });
        } else {
            self.draw_text_internal(centered_origin, &fitted, color, self.theme.text_scale);
        }
        if track_bounds {
            self.track_rect_internal(Rect {
                origin: centered_origin,
                size: Size {
                    width: char_size,
                    height: size.height,
                },
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
        let line_height = text_line_height(self.theme.text_scale.max(1));
        self.draw_text_internal(pos, text, self.theme.text, self.theme.text_scale);
        let size = text_size(text, self.theme.text_scale);
        self.track_rect_internal(Rect { origin: pos, size });
        self.layout
            .cursor
            .y = self
            .layout
            .cursor
            .y
            .saturating_add(i32::try_from(line_height).unwrap_or(i32::MAX))
            .saturating_add(self.layout.spacing);
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

/// Return a stable, bounded line height for monospaced text.
fn text_line_height(text_scale: u32) -> u32 {
    let height = 8u64.saturating_mul(u64::from(text_scale));
    u32::try_from(height).unwrap_or(u32::MAX)
}
