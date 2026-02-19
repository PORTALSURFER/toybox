impl<'a> Ui<'a> {
    /// Resolve control placement and draw optional label text.
    pub(crate) fn resolve_dropdown_layout(
        &mut self,
        label: &str,
        width: i32,
        height: i32,
    ) -> DropdownLayout {
        let control_height = height.max(1);
        let control_size = Size {
            width: width.max(1) as u32,
            height: control_height as u32,
        };
        let block_size = self.dropdown_block_size(label, control_size);
        let rect = Rect {
            origin: self.draw_dropdown_label(label, control_size),
            size: control_size,
        };
        self.track_rect_internal(rect);

        DropdownLayout {
            block_size,
            rect,
            control_height,
        }
    }

    /// Draw dropdown label and return the control rectangle origin.
    pub(crate) fn draw_dropdown_label(&mut self, label: &str, control_size: Size) -> Point {
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

    /// Resolve menu placement relative to control bounds.
    pub(crate) fn resolve_dropdown_menu_geometry(
        &self,
        layout: DropdownLayout,
        option_count: usize,
    ) -> DropdownMenuGeometry {
        let menu_height = layout.control_height * option_count as i32;
        let canvas_height = self.canvas.size().height as i32;
        let open_up = layout.rect.origin.y + layout.control_height + menu_height > canvas_height
            && layout.rect.origin.y >= menu_height;
        DropdownMenuGeometry {
            rect: layout.rect,
            control_height: layout.control_height,
            open_up,
        }
    }

    /// Resolve one option row rectangle for a given index.
    #[cfg_attr(target_os = "windows", allow(dead_code))]
    pub(crate) fn dropdown_option_rect(
        &self,
        geometry: DropdownMenuGeometry,
        index: usize,
    ) -> Rect {
        let row_offset = geometry.control_height * (index as i32 + 1);
        let y = if geometry.open_up {
            geometry.rect.origin.y - row_offset
        } else {
            geometry.rect.origin.y + row_offset
        };
        Rect {
            origin: Point {
                x: geometry.rect.origin.x,
                y,
            },
            size: geometry.rect.size,
        }
    }

    /// Advance the block layout cursor after dropdown rendering.
    pub(crate) fn advance_dropdown_layout_cursor(&mut self, layout: DropdownLayout) {
        self.layout.cursor.y = layout.rect.origin.y + layout.block_size.height as i32 + self.layout.spacing;
    }
}
