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

        if response.open {
            let menu = self.evaluate_open_dropdown_menu(layout, options, selected, response.hovered);
            response.open = menu.open;
            response.changed = menu.changed;
            if response.open {
                self.push_dropdown_overlay(options, menu.hovered_index, menu.geometry);
            }
            if menu.pressed {
                self.consume_mouse_pressed();
            }
        }

        self.advance_dropdown_layout_cursor(layout);
        response
    }
}
