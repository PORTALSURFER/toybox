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
        let layout = self.resolve_dropdown_layout(label, width, height);
        let mut response = self.resolve_dropdown_state(id, layout.rect);
        self.draw_dropdown_control(layout, options, *selected, &response);

        #[cfg(target_os = "windows")]
        {
            if let Some(result) = self.state.take_dropdown_popup_result_for(id) {
                match result {
                    DropdownPopupResult::Selected { index, .. } => {
                        response.changed = *selected != index;
                        *selected = index;
                    }
                    DropdownPopupResult::Closed { .. } => {}
                }
                response.open = false;
                self.state.clear_open_dropdown();
            }

            if response.open {
                let geometry = self.resolve_dropdown_menu_geometry(layout, options.len());
                let surface_rect = self.state.design_rect_to_surface(layout.rect);
                self.state.queue_dropdown_popup_request(DropdownPopupRequest {
                    dropdown_id: id,
                    control_rect_surface: surface_rect,
                    options: options.iter().map(|option| (*option).to_string()).collect(),
                    selected: *selected,
                    open_up: geometry.open_up,
                });
                if self.mouse_pressed() {
                    self.consume_mouse_pressed();
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        if response.open {
            let menu = self.evaluate_open_dropdown_menu(layout, options, selected, response.hovered);
            response.open = menu.open;
            response.changed = menu.changed;
            if response.open {
                let clip_rect = Rect {
                    origin: Point { x: 0, y: 0 },
                    size: self.canvas.size(),
                };
                self.push_dropdown_overlay(layout.rect, options, menu.hovered_index, menu.open_up, clip_rect);
            }
            if menu.pressed {
                self.consume_mouse_pressed();
            }
        }

        self.advance_dropdown_layout_cursor(layout);
        response
    }
}
