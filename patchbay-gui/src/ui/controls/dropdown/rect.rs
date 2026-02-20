/// Request payload for rendering a dropdown inside a fixed rectangle.
#[derive(Clone, Copy)]
pub(crate) struct DropdownRectRenderRequest<'a> {
    /// Stable widget id.
    id: WidgetId,
    /// Label displayed above the dropdown control.
    label: &'a str,
    /// Option labels shown by the dropdown.
    options: &'a [&'a str],
    /// Bounds used for clipping and placement.
    rect: Rect,
    /// Explicit text scale override for the dropdown label.
    text_scale: u32,
    /// Optional visual overrides for dropdown rendering.
    visual_style: DropdownVisualStyle,
}

impl<'a> DropdownRectRenderRequest<'a> {
    /// Build a dropdown-in-rect request with default text scale.
    pub(crate) fn new(
        id: WidgetId,
        label: &'a str,
        options: &'a [&'a str],
        rect: Rect,
    ) -> Self {
        Self {
            id,
            label,
            options,
            rect,
            text_scale: 1,
            visual_style: DropdownVisualStyle::default(),
        }
    }

    /// Override text scale for dropdown label rendering.
    pub(crate) fn with_text_scale(mut self, text_scale: u32) -> Self {
        self.text_scale = text_scale.max(1);
        self
    }

    /// Override dropdown background fill color.
    pub(crate) fn with_background_color(mut self, color: Color) -> Self {
        self.visual_style.fill = Some(color);
        self.visual_style.hover_fill = Some(color);
        self.visual_style.active_fill = Some(color);
        self
    }

    /// Override dropdown background fill color for hovered rows/states only.
    pub(crate) fn with_hover_background_color(mut self, color: Color) -> Self {
        self.visual_style.hover_fill = Some(color);
        self
    }

    /// Override dropdown control fill color while open.
    pub(crate) fn with_active_background_color(mut self, color: Color) -> Self {
        self.visual_style.active_fill = Some(color);
        self
    }

    /// Override dropdown outline color.
    pub(crate) fn with_outline_color(mut self, color: Color) -> Self {
        self.visual_style.outline = Some(color);
        self
    }

    /// Override dropdown text color.
    pub(crate) fn with_text_color(mut self, color: Color) -> Self {
        self.visual_style.text = Some(color);
        self
    }

    /// Override selected option row fill color in the open menu.
    pub(crate) fn with_selected_option_background_color(
        mut self,
        color: Color,
    ) -> Self {
        self.visual_style.selected_option_fill = Some(color);
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
        let height = request.rect.size.height.max(1) as i32;
        let previous_text_scale = self.theme.text_scale;
        self.theme.text_scale = request.text_scale;
        let response = self.dropdown_with_visual_style(
            request.id,
            request.label,
            request.options,
            selected,
            Size {
                width: request.rect.size.width.max(1),
                height: height.max(1) as u32,
            },
            request.visual_style,
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
