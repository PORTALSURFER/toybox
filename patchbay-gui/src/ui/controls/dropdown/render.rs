impl<'a> Ui<'a> {
    /// Draw the closed-state dropdown control and selected option label.
    pub(crate) fn draw_dropdown_control(
        &mut self,
        layout: DropdownLayout,
        options: &[&str],
        selected: usize,
        response: &DropdownResponse,
    ) {
        let fill = if response.open {
            self.theme.knob_active
        } else if response.hovered {
            self.theme.knob_hover
        } else {
            self.theme.knob_fill
        };
        self.fill_rect_clipped(layout.rect, fill);
        self.stroke_rect_clipped(layout.rect, 1, self.theme.knob_outline);

        let current = options.get(selected).copied().unwrap_or("-");
        let text_pos = Point {
            x: layout.rect.origin.x + 4,
            y: layout.rect.origin.y
                + (layout.control_height - (7 * self.theme.text_scale as i32)) / 2,
        };
        let _ = self.draw_text_single_line_clamped(
            text_pos,
            current,
            layout.rect.size.width.saturating_sub(8),
            self.theme.text,
            false,
        );
    }
}
