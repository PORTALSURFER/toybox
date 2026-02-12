impl<'a> Ui<'a> {
    /// Resolve layout geometry for a toggle and draw its optional label.
    fn resolve_toggle_layout(&mut self, label: &str, width: i32, height: i32) -> ToggleLayoutResolved {
        let width = width.max(1);
        let height = height.max(1);
        let control_size = Size {
            width: width as u32,
            height: height as u32,
        };
        let block_size = self.toggle_block_size(label, control_size);
        let origin = self.draw_toggle_label(label, control_size);
        let rect = Rect {
            origin,
            size: control_size,
        };
        self.track_rect_internal(rect);
        ToggleLayoutResolved { block_size, rect }
    }

    /// Draw the toggle label and return the control origin point.
    fn draw_toggle_label(&mut self, label: &str, control_size: Size) -> Point {
        let base = self.layout.cursor;
        if label.is_empty() {
            return base;
        }
        let _ = self.draw_text_single_line_clamped(base, label, control_size.width, self.theme.text, true);
        Point {
            x: base.x,
            y: base.y + (8 * self.theme.text_scale as i32),
        }
    }

    /// Resolve the button rectangle from current layout cursor and requested size.
    fn resolve_button_rect(&mut self, width: i32, height: i32) -> Rect {
        let control_size = Size {
            width: width.max(1) as u32,
            height: height.max(1) as u32,
        };
        let rect = Rect {
            origin: self.layout.cursor,
            size: control_size,
        };
        self.track_rect_internal(rect);
        rect
    }

    /// Advance the layout cursor after rendering a toggle.
    fn advance_toggle_cursor(&mut self, block_size: Size, rect_y: i32) {
        self.layout.cursor.y = rect_y + block_size.height as i32 + self.layout.spacing;
    }

    /// Advance the vertical layout cursor after drawing a button.
    fn advance_button_cursor(&mut self, rect: Rect) {
        self.layout.cursor.y = rect.origin.y + rect.size.height as i32 + self.layout.spacing;
    }
}
