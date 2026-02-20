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
        self.dropdown_with_visual_style(
            id,
            label,
            options,
            selected,
            Size { width: width.max(0) as u32, height: height.max(0) as u32 },
            DropdownVisualStyle::default(),
        )
    }

    /// Draw a dropdown selector with optional style overrides.
    pub(crate) fn dropdown_with_visual_style(
        &mut self,
        id: WidgetId,
        label: &str,
        options: &[&str],
        selected: &mut usize,
        control_size: Size,
        visual_style: DropdownVisualStyle,
    ) -> DropdownResponse {
        let width = control_size.width.min(i32::MAX as u32) as i32;
        let height = control_size.height.min(i32::MAX as u32) as i32;
        let layout = self.resolve_dropdown_layout(label, width, height);
        let mut response = self.resolve_dropdown_state(id, layout.rect);
        if response.open {
            self.state.mark_open_dropdown_seen(id);
        }
        self.draw_dropdown_control(layout, options, *selected, &response, visual_style);

        if response.open {
            let menu = self.evaluate_open_dropdown_menu(id, layout, options, selected, response.hovered);
            response.open = menu.open;
            response.changed = menu.changed;
            if response.open {
                self.push_dropdown_overlay(
                    options,
                    menu.hovered_index,
                    *selected,
                    menu.geometry,
                    visual_style,
                );
            }
        }

        self.advance_dropdown_layout_cursor(layout);
        response
    }
}
