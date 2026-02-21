impl<'a> Ui<'a> {
    /// Resolve hover/open state and apply click-to-toggle behavior.
    pub(crate) fn resolve_dropdown_state(&mut self, id: WidgetId, rect: Rect) -> DropdownResponse {
        let hovered = self.pointer_inside_clipped_rect(rect);
        if hovered {
            self.state.hot = Some(id);
        }

        let mut response = DropdownResponse {
            hovered,
            open: self.state.open_dropdown == Some(id),
            changed: false,
        };
        if hovered && self.claim_mouse_pressed_for_dropdown(id) {
            if response.open {
                self.state.clear_open_dropdown();
                response.open = false;
            } else {
                self.state.open_dropdown = Some(id);
                self.state.open_dropdown_scroll_px = 0;
                response.open = true;
            }
        }
        response
    }

    /// Evaluate option hover/selection while the dropdown menu is open.
    pub(crate) fn evaluate_open_dropdown_menu(
        &mut self,
        id: WidgetId,
        layout: DropdownLayout,
        options: &[&str],
        selected: &mut usize,
        hovered_control: bool,
    ) -> DropdownMenuInteraction {
        let mut geometry = self.resolve_dropdown_menu_geometry(
            layout,
            options.len(),
            self.state.open_dropdown_scroll_px,
        );
        let menu_hovered = self.pointer_inside_rect(geometry.menu_rect);
        let scroll_px = self.apply_dropdown_scroll(geometry, menu_hovered);
        if scroll_px != geometry.scroll_px {
            geometry = self.resolve_dropdown_menu_geometry(layout, options.len(), scroll_px);
        }
        let pressed = self.claim_mouse_pressed_for_dropdown(id);
        let hovered_index = self.find_hovered_dropdown_option(geometry);
        let selection_change = self.apply_dropdown_selection(pressed, hovered_index, selected);
        let open = self.resolve_dropdown_open_after_press(
            pressed,
            hovered_control,
            menu_hovered,
            selection_change,
        );

        DropdownMenuInteraction {
            open,
            changed: selection_change,
            hovered_index,
            geometry,
        }
    }

    /// Find hovered option index for the current pointer position.
    pub(crate) fn find_hovered_dropdown_option(
        &self,
        geometry: DropdownMenuGeometry,
    ) -> Option<usize> {
        let mut hovered_index = None;
        for index in 0..geometry.option_count {
            let option_rect = self.dropdown_option_rect(geometry, index, geometry.scroll_px);
            if self.pointer_inside_rect(option_rect) {
                hovered_index = Some(index);
            }
        }
        hovered_index
    }

    /// Apply wheel scrolling while the pointer is inside the open menu.
    fn apply_dropdown_scroll(&mut self, geometry: DropdownMenuGeometry, menu_hovered: bool) -> i32 {
        if !menu_hovered || self.input.wheel_delta == 0.0 || geometry.max_scroll_px == 0 {
            return geometry.scroll_px;
        }
        let step = (geometry.control_height / 2).max(8);
        let delta = if self.input.wheel_delta > 0.0 {
            -step
        } else {
            step
        };
        let scroll_px = (geometry.scroll_px + delta).clamp(0, geometry.max_scroll_px);
        self.state.open_dropdown_scroll_px = scroll_px;
        scroll_px
    }

    /// Apply hovered-option selection on press and return whether value changed.
    pub(crate) fn apply_dropdown_selection(
        &mut self,
        pressed: bool,
        hovered_index: Option<usize>,
        selected: &mut usize,
    ) -> bool {
        let Some(index) = hovered_index else {
            return false;
        };
        if !pressed {
            return false;
        }
        let changed = *selected != index;
        *selected = index;
        self.state.clear_open_dropdown();
        changed
    }

    /// Resolve menu-open state after handling pointer press rules.
    pub(crate) fn resolve_dropdown_open_after_press(
        &mut self,
        pressed: bool,
        hovered_control: bool,
        menu_hovered: bool,
        selection_change: bool,
    ) -> bool {
        if selection_change {
            return false;
        }
        if pressed && !hovered_control && !menu_hovered {
            self.state.clear_open_dropdown();
            return false;
        }
        true
    }
}
