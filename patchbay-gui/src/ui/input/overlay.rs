impl<'a> Ui<'a> {

    /// Queue a dropdown overlay for deferred draw order.
    fn push_dropdown_overlay(
        &mut self,
        base_rect: Rect,
        options: &[&str],
        hovered: Option<usize>,
        open_up: bool,
        clip_rect: Rect,
    ) {
        self.state.overlays.push(DropdownOverlay {
            base_rect,
            options: options.iter().map(|option| (*option).to_string()).collect(),
            hovered,
            open_up,
            clip_rect,
        });
    }

    /// Draw any deferred overlays (dropdown menus).
    pub fn draw_overlays(&mut self) {
        let overlays = self.state.overlays.clone();
        for overlay in overlays.iter() {
            let rect = overlay.base_rect;
            let height = rect.size.height as i32;
            for (index, option) in overlay.options.iter().enumerate() {
                let option_rect = Rect {
                    origin: Point {
                        x: rect.origin.x,
                        y: if overlay.open_up {
                            rect.origin.y - height * (index as i32 + 1)
                        } else {
                            rect.origin.y + height * (index as i32 + 1)
                        },
                    },
                    size: rect.size,
                };
                let Some(visible_rect) = rect_intersection(option_rect, overlay.clip_rect) else {
                    continue;
                };
                let option_fill = if overlay.hovered == Some(index) {
                    self.theme.knob_hover
                } else {
                    self.theme.knob_fill
                };
                self.canvas.fill_rect(visible_rect, option_fill);
                self.canvas
                    .stroke_rect(visible_rect, 1, self.theme.knob_outline);
                let option_text = Point {
                    x: visible_rect.origin.x + 4,
                    y: visible_rect.origin.y + (height - (7 * self.theme.text_scale as i32)) / 2,
                };
                let fitted = fit_text_single_line_ellipsis(
                    option,
                    visible_rect.size.width.saturating_sub(8),
                    self.theme.text_scale,
                );
                self.draw_text_internal(
                    option_text,
                    &fitted,
                    self.theme.text,
                    self.theme.text_scale,
                );
            }
        }
    }

    /// Clear any deferred overlay drawings for the next frame.
    pub fn clear_overlays(&mut self) {
        self.state.overlays.clear();
    }

    /// Reset per-frame input consumption flags.
    pub fn reset_input_consumption(&mut self) {
        self.state.consume_mouse_pressed = false;
    }

    /// Return true when a primary-button press is still available this frame.
    fn mouse_pressed(&self) -> bool {
        self.input.mouse_pressed && !self.state.consume_mouse_pressed
    }

    /// Consume the frame's primary-button press.
    fn consume_mouse_pressed(&mut self) {
        self.state.consume_mouse_pressed = true;
    }
}
