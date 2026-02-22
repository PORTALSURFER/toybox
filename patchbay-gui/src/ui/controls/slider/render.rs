impl<'a> Ui<'a> {
    /// Resolve all geometry/colors needed to draw slider visuals.
    pub(crate) fn resolve_slider_visuals(
        &self,
        rect: Rect,
        height: i32,
        range: (f32, f32),
        value: f32,
        response: SliderResponse,
        style: ControlVisualState,
    ) -> SliderVisualState {
        let span = (range.1 - range.0).max(1.0e-6);
        let t = ((value - range.0) / span).clamp(0.0, 1.0);
        let track_height = (height / 4).max(4) as u32;
        let track_rect = Rect {
            origin: Point {
                x: rect.origin.x,
                y: rect.origin.y + (height - track_height as i32) / 2,
            },
            size: Size {
                width: rect.size.width,
                height: track_height,
            },
        };
        let fill_rect = Rect {
            origin: track_rect.origin,
            size: Size {
                width: ((rect.size.width as f32) * t).round() as u32,
                height: track_rect.size.height,
            },
        };
        SliderVisualState {
            track_rect,
            fill_rect,
            handle_center: Point {
                x: rect.origin.x + (rect.size.width as f32 * t) as i32,
                y: rect.origin.y + height / 2,
            },
            handle_radius: (height / 2).max(3),
            track_fill: self.resolve_slider_fill_color(style.variants, style.disabled, response),
            value_fill: self.theme.knob_indicator,
            handle_fill: self.theme.knob_indicator,
            outline: if style.focused {
                self.theme.knob_active
            } else {
                self.theme.knob_outline
            },
        }
    }

    /// Draw slider visuals from precomputed geometry.
    pub(crate) fn draw_slider_visuals(&mut self, visuals: SliderVisualState) {
        self.fill_rect_clipped(visuals.track_rect, visuals.track_fill);
        self.stroke_rect_clipped(visuals.track_rect, 1, visuals.outline);
        self.fill_rect_clipped(visuals.fill_rect, visuals.value_fill);
        self.canvas.fill_circle(
            visuals.handle_center,
            visuals.handle_radius,
            visuals.handle_fill,
        );
    }
}
