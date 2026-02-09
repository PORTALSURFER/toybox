impl<'a> Ui<'a> {
    /// Resolve slider layout geometry and draw the optional label.
    pub(crate) fn resolve_slider_layout(
        &mut self,
        label: &str,
        width: i32,
        height: i32,
    ) -> SliderLayoutResolved {
        let height = height.max(1);
        let control_size = Size {
            width: width.max(1) as u32,
            height: height as u32,
        };
        let block_size = self.slider_block_size(label, control_size);
        let rect_origin = self.draw_slider_label(label, control_size);
        let rect = Rect {
            origin: rect_origin,
            size: control_size,
        };
        self.track_rect_internal(rect);
        SliderLayoutResolved {
            block_size,
            rect,
            height,
        }
    }

    /// Draw slider label and return the control rectangle origin.
    pub(crate) fn draw_slider_label(&mut self, label: &str, control_size: Size) -> Point {
        let base = self.layout.cursor;
        if label.is_empty() {
            return base;
        }
        let _ = self.draw_text_single_line_clamped(
            base,
            label,
            control_size.width,
            self.theme.text,
            true,
        );
        Point {
            x: base.x,
            y: base.y + 8 * self.theme.text_scale as i32,
        }
    }

    /// Advance layout cursor after drawing a slider block.
    pub(crate) fn advance_slider_cursor(&mut self, rect: Rect, block_size: Size) {
        self.layout.cursor.y = rect.origin.y + block_size.height as i32 + self.layout.spacing;
    }
}
