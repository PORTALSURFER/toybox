impl<'a> Ui<'a> {
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
        self.canvas.fill_circle(thumb_center, thumb_radius, self.theme.knob_outline);
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
}
