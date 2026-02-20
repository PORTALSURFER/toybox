impl<'a> Ui<'a> {
    /// Draw the closed-state dropdown control and selected option label.
    pub(crate) fn draw_dropdown_control(
        &mut self,
        layout: DropdownLayout,
        options: &[&str],
        selected: usize,
        response: &DropdownResponse,
        visual_style: DropdownVisualStyle,
    ) {
        let fill = if response.open {
            visual_style.fill.unwrap_or(self.theme.knob_active)
        } else if response.hovered {
            visual_style.hover_fill.unwrap_or(self.theme.knob_hover)
        } else {
            visual_style.fill.unwrap_or(self.theme.knob_fill)
        };
        let outline = visual_style.outline.unwrap_or(self.theme.knob_outline);
        let text_color = visual_style.text.unwrap_or(self.theme.text);
        self.fill_rect_clipped(layout.rect, fill);
        self.stroke_rect_clipped(layout.rect, 1, outline);

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
            text_color,
            false,
        );
    }
}
