/// Request payload for rendering a dropdown inside a fixed rectangle.
#[derive(Clone, Copy)]
pub(crate) struct DropdownRectRenderRequest<'a> {
    /// Stable widget id.
    id: WidgetId,
    /// Label displayed above the dropdown control.
    label: &'a str,
    /// Option labels shown by the dropdown.
    options: &'a [&'a str],
    /// Control footprint for dropdown visuals.
    control_size: Size,
    /// Bounds used for clipping and placement.
    rect: Rect,
    /// Explicit text scale override for the dropdown label.
    text_scale: u32,
}

impl<'a> DropdownRectRenderRequest<'a> {
    /// Build a dropdown-in-rect request with default text scale.
    pub(crate) fn new(
        id: WidgetId,
        label: &'a str,
        options: &'a [&'a str],
        control_size: Size,
        rect: Rect,
    ) -> Self {
        Self {
            id,
            label,
            options,
            control_size,
            rect,
            text_scale: 1,
        }
    }

    /// Override text scale for dropdown label rendering.
    pub(crate) fn with_text_scale(mut self, text_scale: u32) -> Self {
        self.text_scale = text_scale.max(1);
        self
    }
}

impl<'a> Ui<'a> {

    /// Render a dropdown in a fixed rectangle without affecting surrounding layout.
    pub(crate) fn dropdown_in_rect(
        &mut self,
        selected: &mut usize,
        request: DropdownRectRenderRequest<'_>,
    ) -> DropdownResponse {
        let previous = *self.layout;
        self.layout.cursor = request.rect.origin;
        let height = request.control_size.height.max(1) as i32;
        let previous_text_scale = self.theme.text_scale;
        self.theme.text_scale = request.text_scale;
        let response = self.dropdown(
            request.id,
            request.label,
            request.options,
            selected,
            request.rect.size.width.max(1) as i32,
            height,
        );
        self.theme.text_scale = previous_text_scale;
        *self.layout = previous;
        response
    }

    /// Render a dropdown in a fixed rectangle with an explicit text scale.
    pub(crate) fn dropdown_in_rect_scaled(
        &mut self,
        selected: &mut usize,
        request: DropdownRectRenderRequest<'_>,
    ) -> DropdownResponse {
        self.dropdown_in_rect(selected, request)
    }
}
