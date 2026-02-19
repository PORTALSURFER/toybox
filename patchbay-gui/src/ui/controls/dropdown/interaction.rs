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
        if hovered && self.mouse_pressed() {
            if response.open {
                self.state.open_dropdown = None;
                response.open = false;
            } else {
                self.state.open_dropdown = Some(id);
                response.open = true;
            }
        }
        response
    }

    /// Evaluate option hover/selection while the dropdown menu is open.
    #[cfg_attr(target_os = "windows", allow(dead_code))]
    pub(crate) fn evaluate_open_dropdown_menu(
        &mut self,
        layout: DropdownLayout,
        options: &[&str],
        selected: &mut usize,
        hovered_control: bool,
    ) -> DropdownMenuInteraction {
        let geometry = self.resolve_dropdown_menu_geometry(layout, options.len());
        let pressed = self.mouse_pressed();
        let (hovered_index, any_hovered) =
            self.find_hovered_dropdown_option(geometry, options.len());
        let selection_change = self.apply_dropdown_selection(pressed, hovered_index, selected);
        let open = self.resolve_dropdown_open_after_press(
            pressed,
            hovered_control,
            any_hovered,
            selection_change,
        );

        DropdownMenuInteraction {
            open,
            changed: selection_change,
            hovered_index,
            open_up: geometry.open_up,
            pressed,
        }
    }

    /// Find hovered option index for the current pointer position.
    #[cfg_attr(target_os = "windows", allow(dead_code))]
    pub(crate) fn find_hovered_dropdown_option(
        &self,
        geometry: DropdownMenuGeometry,
        option_count: usize,
    ) -> (Option<usize>, bool) {
        let mut hovered_index = None;
        let mut any_hovered = false;
        for index in 0..option_count {
            let option_rect = self.dropdown_option_rect(geometry, index);
            if option_rect.contains(self.input.pointer_pos) {
                any_hovered = true;
                hovered_index = Some(index);
            }
        }
        (hovered_index, any_hovered)
    }

    /// Apply hovered-option selection on press and return whether value changed.
    #[cfg_attr(target_os = "windows", allow(dead_code))]
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
        *selected = index;
        self.state.open_dropdown = None;
        true
    }

    /// Resolve menu-open state after handling pointer press rules.
    #[cfg_attr(target_os = "windows", allow(dead_code))]
    pub(crate) fn resolve_dropdown_open_after_press(
        &mut self,
        pressed: bool,
        hovered_control: bool,
        any_hovered: bool,
        selection_change: bool,
    ) -> bool {
        if selection_change {
            return false;
        }
        if pressed && !hovered_control && !any_hovered {
            self.state.open_dropdown = None;
            return false;
        }
        true
    }
}
