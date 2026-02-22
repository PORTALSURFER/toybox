/// Resolved layout geometry for a toggle draw pass.
#[derive(Clone, Copy)]
struct ToggleLayoutResolved {
    /// Total vertical block consumed by toggle + optional label.
    block_size: Size,
    /// Toggle control rectangle.
    rect: Rect,
}

/// Request payload for rendering a toggle inside a fixed rectangle.
pub(crate) struct ToggleRectRenderRequest<'a> {
    /// Stable widget id.
    pub(crate) id: WidgetId,
    /// Label rendered above the toggle body.
    pub(crate) label: &'a str,
    /// Mutable toggle model value.
    pub(crate) value: &'a mut bool,
    /// Declared control footprint.
    pub(crate) control_size: Size,
    /// Bounds used for clipping and placement.
    pub(crate) rect: Rect,
    /// Explicit text scale override for the label.
    pub(crate) text_scale: u32,
    /// Optional color variants for role-driven styling.
    pub(crate) color_variants: Option<ControlColorVariants>,
    /// Disable pointer interaction and render disabled visuals.
    pub(crate) disabled: bool,
    /// Render focus affordances.
    pub(crate) focused: bool,
}

impl<'a> ToggleRectRenderRequest<'a> {
    /// Build a toggle-in-rect request with default text scale.
    pub(crate) fn new(
        id: WidgetId,
        label: &'a str,
        value: &'a mut bool,
        control_size: Size,
        rect: Rect,
    ) -> Self {
        Self {
            id,
            label,
            value,
            control_size,
            rect,
            text_scale: 1,
            color_variants: None,
            disabled: false,
            focused: false,
        }
    }

    /// Override text scale for toggle label rendering.
    pub(crate) fn with_text_scale(mut self, text_scale: u32) -> Self {
        self.text_scale = text_scale.max(1);
        self
    }

    /// Override state variants used for role-driven toggle rendering.
    pub(crate) fn with_color_variants(mut self, variants: ControlColorVariants) -> Self {
        self.color_variants = Some(variants);
        self
    }

    /// Disable pointer interaction and render disabled styling.
    pub(crate) fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Render focus affordances for this toggle.
    pub(crate) fn with_focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

/// Request payload for rendering a button inside a fixed rectangle.
pub(crate) struct ButtonRectRenderRequest<'a> {
    /// Stable widget id.
    pub(crate) id: WidgetId,
    /// Button label text.
    pub(crate) label: &'a str,
    /// Declared control footprint.
    pub(crate) control_size: Size,
    /// Bounds used for clipping and placement.
    pub(crate) rect: Rect,
    /// Explicit text scale override for the label.
    pub(crate) text_scale: u32,
    /// Optional color variants for role-driven styling.
    pub(crate) color_variants: Option<ControlColorVariants>,
    /// Disable pointer interaction and render disabled visuals.
    pub(crate) disabled: bool,
    /// Render focus affordances.
    pub(crate) focused: bool,
}

impl<'a> ButtonRectRenderRequest<'a> {
    /// Build a button-in-rect request with default text scale.
    pub(crate) fn new(
        id: WidgetId,
        label: &'a str,
        control_size: Size,
        rect: Rect,
    ) -> Self {
        Self {
            id,
            label,
            control_size,
            rect,
            text_scale: 1,
            color_variants: None,
            disabled: false,
            focused: false,
        }
    }

    /// Override text scale for button label rendering.
    pub(crate) fn with_text_scale(mut self, text_scale: u32) -> Self {
        self.text_scale = text_scale.max(1);
        self
    }

    /// Override state variants used for role-driven button rendering.
    pub(crate) fn with_color_variants(mut self, variants: ControlColorVariants) -> Self {
        self.color_variants = Some(variants);
        self
    }

    /// Disable pointer interaction and render disabled styling.
    pub(crate) fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Render focus affordances for this button.
    pub(crate) fn with_focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}
