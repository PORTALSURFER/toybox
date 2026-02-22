impl<'a> Ui<'a> {
    /// Resolve toggle fill color from value and hover state.
    fn resolve_toggle_fill(
        &self,
        value: bool,
        hovered: bool,
        variants: Option<ControlColorVariants>,
        disabled: bool,
    ) -> Color {
        if let Some(variants) = variants {
            if disabled {
                return variants.disabled;
            }
            if value {
                return variants.active;
            }
            if hovered {
                return variants.hover;
            }
            return variants.base;
        }
        if value {
            self.theme.knob_indicator
        } else if hovered {
            self.theme.knob_hover
        } else {
            self.theme.knob_fill
        }
    }

    /// Draw toggle body and thumb.
    fn draw_toggle_visuals(
        &mut self,
        rect: Rect,
        fill: Color,
        value: bool,
        variants: Option<ControlColorVariants>,
        focused: bool,
    ) {
        self.fill_rect_clipped(rect, fill);
        let outline = if focused {
            variants
                .map(|variants| variants.focus_ring)
                .unwrap_or(self.theme.knob_active)
        } else {
            self.theme.knob_outline
        };
        self.stroke_rect_clipped(rect, 1, outline);
        let width = rect.size.width as i32;
        let height = rect.size.height as i32;
        let thumb_radius = (height / 2).max(3);
        let thumb_x = if value {
            rect.origin.x + width - thumb_radius
        } else {
            rect.origin.x + thumb_radius
        };
        let thumb_center = Point {
            x: thumb_x,
            y: rect.origin.y + height / 2,
        };
        let thumb_fill = variants
            .map(|variants| variants.base)
            .unwrap_or(self.theme.knob_outline);
        self.canvas.fill_circle(thumb_center, thumb_radius, thumb_fill);
    }

    /// Draw button body, outline, and clamped label text.
    fn draw_button_visuals(
        &mut self,
        rect: Rect,
        label: &str,
        hovered: bool,
        active: bool,
        style: ControlVisualState,
    ) {
        let fill = if let Some(variants) = style.variants {
            if style.disabled {
                variants.disabled
            } else if active {
                variants.active
            } else if hovered {
                variants.hover
            } else {
                variants.base
            }
        } else if hovered {
            self.theme.knob_hover
        } else {
            self.theme.knob_fill
        };
        self.fill_rect_clipped(rect, fill);
        let outline = if style.focused {
            style
                .variants
                .map(|variants| variants.focus_ring)
                .unwrap_or(self.theme.knob_active)
        } else {
            self.theme.knob_outline
        };
        self.stroke_rect_clipped(rect, 1, outline);
        let center_x =
            rect.origin.x.saturating_add(i32::try_from(rect.size.width / 2).unwrap_or(i32::MAX));
        let text_pos = Point {
            x: rect.origin.x,
            y: rect.origin.y + (rect.size.height as i32 - (7 * self.theme.text_scale as i32)) / 2,
        };
        let text_color = style
            .variants
            .map(|variants| {
                if style.disabled {
                    variants.disabled
                } else {
                    variants.base
                }
            })
            .unwrap_or(self.theme.text);
        let _ = self.draw_text_single_line_hard_clamped_centered_on_char_size(
            text_pos,
            center_x,
            label,
            rect.size.width,
            text_color,
            false,
        );
    }
}
