impl<'a> Ui<'a> {

    /// Render a dropdown in a fixed rectangle without affecting surrounding layout.
    pub(crate) fn dropdown_in_rect(
        &mut self,
        id: WidgetId,
        label: &str,
        options: &[&str],
        selected: &mut usize,
        control_size: Size,
        rect: Rect,
    ) -> DropdownResponse {
        let previous = *self.layout;
        self.layout.cursor = rect.origin;
        let height = control_size.height.max(1) as i32;
        let response = {
            let mut response = DropdownResponse::default();
            self.with_clip(rect, |ui| {
                response = ui.dropdown(
                    id,
                    label,
                    options,
                    selected,
                    rect.size.width.max(1) as i32,
                    height,
                );
            });
            response
        };
        *self.layout = previous;
        response
    }

    /// Render a dropdown in a fixed rectangle with an explicit text scale.
    pub(crate) fn dropdown_in_rect_scaled(
        &mut self,
        id: WidgetId,
        label: &str,
        options: &[&str],
        selected: &mut usize,
        control_size: Size,
        rect: Rect,
        text_scale: u32,
    ) -> DropdownResponse {
        let previous = self.theme.text_scale;
        self.theme.text_scale = text_scale.max(1);
        let response = self.dropdown_in_rect(id, label, options, selected, control_size, rect);
        self.theme.text_scale = previous;
        response
    }
}
