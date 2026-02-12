impl<'a> Ui<'a> {
    /// Resolve hover/hot interaction state for toggle controls.
    fn begin_toggle_interaction(&mut self, id: WidgetId, rect: Rect) -> ToggleResponse {
        let hovered = self.pointer_inside_clipped_rect(rect);
        if hovered {
            self.state.hot = Some(id);
        }
        ToggleResponse {
            hovered,
            changed: false,
        }
    }

    /// Apply press interaction and update the toggle value.
    fn apply_toggle_press(&mut self, hovered: bool, value: &mut bool) -> bool {
        if !(hovered && self.mouse_pressed()) {
            return false;
        }
        *value = !*value;
        true
    }

    /// Resolve hover/click state for a button interaction.
    fn resolve_button_interaction(&mut self, id: WidgetId, rect: Rect) -> ButtonResponse {
        let hovered = self.pointer_inside_clipped_rect(rect);
        if hovered {
            self.state.hot = Some(id);
        }
        ButtonResponse {
            hovered,
            clicked: hovered && self.mouse_pressed(),
        }
    }
}
