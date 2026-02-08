impl<'a> Ui<'a> {

    /// Draw a dropdown selector with a stable key and a dynamic label.
    pub fn dropdown_with_key(
        &mut self,
        key: &str,
        label: &str,
        options: &[&str],
        selected: &mut usize,
        width: i32,
        height: i32,
    ) -> DropdownResponse {
        let id = WidgetId::from_label(key);
        self.dropdown(id, label, options, selected, width, height)
    }
}
