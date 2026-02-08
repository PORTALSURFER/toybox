impl<'a> Ui<'a> {

    /// Draw a dropdown selector with the given label and options.
    pub fn dropdown(
        &mut self,
        id: WidgetId,
        label: &str,
        options: &[&str],
        selected: &mut usize,
        width: i32,
        height: i32,
    ) -> DropdownResponse {
        let width = width.max(1);
        let height = height.max(1);
        let control_size = Size {
            width: width as u32,
            height: height as u32,
        };
        let block_size = self.dropdown_block_size(label, control_size);
        let label_height = 8 * self.theme.text_scale as i32;
        let base = self.layout.cursor;
        let mut rect_origin = base;
        if !label.is_empty() {
            let _ = self.draw_text_single_line_clamped(
                base,
                label,
                control_size.width,
                self.theme.text,
                true,
            );
            rect_origin.y += label_height;
        }

        let rect = Rect {
            origin: rect_origin,
            size: control_size,
        };
        self.track_rect_internal(rect);
        let hovered = self.pointer_inside_clipped_rect(rect);
        if hovered {
            self.state.hot = Some(id);
        }
        let mut response = DropdownResponse {
            hovered,
            open: self.state.open_dropdown == Some(id),
            changed: false,
        };

        if hovered && self.mouse_pressed() {
            if response.open {
                self.state.open_dropdown = None;
                response.open = false;
            } else {
                self.state.open_dropdown = Some(id);
                response.open = true;
            }
        }

        let fill = if response.open {
            self.theme.knob_active
        } else if hovered {
            self.theme.knob_hover
        } else {
            self.theme.knob_fill
        };
        self.fill_rect_clipped(rect, fill);
        self.stroke_rect_clipped(rect, 1, self.theme.knob_outline);
        let current = options.get(*selected).copied().unwrap_or("-");
        let text_pos = Point {
            x: rect.origin.x + 4,
            y: rect.origin.y + (height - (7 * self.theme.text_scale as i32)) / 2,
        };
        let _ = self.draw_text_single_line_clamped(
            text_pos,
            current,
            rect.size.width.saturating_sub(8),
            self.theme.text,
            false,
        );

        if response.open {
            let pressed = self.mouse_pressed();
            let mut any_hovered = false;
            let mut hovered_index = None;
            let menu_height = height * options.len() as i32;
            let canvas_height = self.canvas.size().height as i32;
            let open_up = rect.origin.y + height + menu_height > canvas_height
                && rect.origin.y >= menu_height;
            for (index, _option) in options.iter().enumerate() {
                let option_rect = Rect {
                    origin: Point {
                        x: rect.origin.x,
                        y: if open_up {
                            rect.origin.y - height * (index as i32 + 1)
                        } else {
                            rect.origin.y + height * (index as i32 + 1)
                        },
                    },
                    size: rect.size,
                };
                let option_hovered = self.pointer_inside_clipped_rect(option_rect);
                if option_hovered {
                    any_hovered = true;
                    hovered_index = Some(index);
                }
                if option_hovered && pressed {
                    *selected = index;
                    response.changed = true;
                    self.state.open_dropdown = None;
                    response.open = false;
                }
            }

            if pressed && !hovered && !any_hovered {
                self.state.open_dropdown = None;
                response.open = false;
            }

            if response.open {
                let clip_rect = self.clipped_rect(rect).unwrap_or(rect);
                self.push_dropdown_overlay(rect, options, hovered_index, open_up, clip_rect);
            }

            if pressed {
                self.consume_mouse_pressed();
            }
        }

        self.layout.cursor.y = rect.origin.y + block_size.height as i32 + self.layout.spacing;
        response
    }
}
