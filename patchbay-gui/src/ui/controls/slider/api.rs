/// Request payload for rendering a slider inside a fixed rectangle.
#[derive(Clone, Copy)]
pub(crate) struct SliderRectRenderRequest<'a> {
    /// Stable widget id.
    id: WidgetId,
    /// Label displayed above the slider track.
    label: &'a str,
    /// Inclusive value range.
    range: (f32, f32),
    /// Control footprint for slider visuals.
    control_size: Size,
    /// Bounds used for clipping and placement.
    rect: Rect,
    /// Explicit text scale override for the label.
    text_scale: u32,
}

impl<'a> SliderRectRenderRequest<'a> {
    /// Build a slider-in-rect request with default text scale.
    pub(crate) fn new(
        id: WidgetId,
        label: &'a str,
        range: (f32, f32),
        control_size: Size,
        rect: Rect,
    ) -> Self {
        Self {
            id,
            label,
            range,
            control_size,
            rect,
            text_scale: 1,
        }
    }

    /// Override text scale for slider label rendering.
    pub(crate) fn with_text_scale(mut self, text_scale: u32) -> Self {
        self.text_scale = text_scale.max(1);
        self
    }
}

/// Resolved layout geometry for one slider draw call.
#[derive(Clone, Copy)]
struct SliderLayoutResolved {
    /// Total vertical block consumed by slider + optional label.
    block_size: Size,
    /// Slider control rectangle.
    rect: Rect,
    /// Slider control height in pixels.
    height: i32,
}

/// Precomputed drawing geometry for slider visuals.
#[derive(Clone, Copy)]
struct SliderVisualState {
    /// Slider track rectangle.
    track_rect: Rect,
    /// Filled portion of the track.
    fill_rect: Rect,
    /// Handle center point.
    handle_center: Point,
    /// Handle radius in pixels.
    handle_radius: i32,
    /// Track fill color based on interaction state.
    fill: Color,
}

/// Public slider configuration shared across keyed and non-keyed slider APIs.
#[derive(Clone, Copy)]
pub struct SliderConfig {
    /// Inclusive slider value range.
    pub range: (f32, f32),
    /// Slider control footprint.
    pub size: Size,
}

impl<'a> Ui<'a> {
    /// Draw a horizontal slider with the given label and value.
    ///
    /// The provided `id` must be stable across frames. If the label changes
    /// while dragging, use [`Ui::slider_with_key`] to provide a separate stable
    /// identifier.
    pub fn slider(
        &mut self,
        id: WidgetId,
        label: &str,
        value: &mut f32,
        config: SliderConfig,
    ) -> SliderResponse {
        let width = config.size.width.max(1) as i32;
        let height = config.size.height.max(1) as i32;
        let layout = self.resolve_slider_layout(label, width, height);
        let mut response = self.begin_slider_interaction(id, layout.rect);

        response.changed |= self.apply_slider_drag(id, layout.rect, config.range, value);
        response.changed |= self.apply_slider_wheel(response.hovered, config.range, value);

        let visuals = self.resolve_slider_visuals(
            layout.rect,
            layout.height,
            config.range,
            *value,
            response,
        );
        self.draw_slider_visuals(visuals);
        self.advance_slider_cursor(layout.rect, layout.block_size);
        response
    }

    /// Resolve slider layout geometry and draw the optional label.
    fn resolve_slider_layout(&mut self, label: &str, width: i32, height: i32) -> SliderLayoutResolved {
        let height = height.max(1);
        let control_size = Size {
            width: width.max(1) as u32,
            height: height as u32,
        };
        let block_size = self.slider_block_size(label, control_size);
        let rect_origin = self.draw_slider_label(label, control_size);
        let rect = Rect {
            origin: rect_origin,
            size: control_size,
        };
        self.track_rect_internal(rect);
        SliderLayoutResolved {
            block_size,
            rect,
            height,
        }
    }

    /// Draw slider label and return the control rectangle origin.
    fn draw_slider_label(&mut self, label: &str, control_size: Size) -> Point {
        let base = self.layout.cursor;
        if label.is_empty() {
            return base;
        }
        let _ = self.draw_text_single_line_clamped(
            base,
            label,
            control_size.width,
            self.theme.text,
            true,
        );
        Point {
            x: base.x,
            y: base.y + 8 * self.theme.text_scale as i32,
        }
    }

    /// Resolve slider hover/active state for this frame.
    fn begin_slider_interaction(&mut self, id: WidgetId, rect: Rect) -> SliderResponse {
        let hovered = self.pointer_inside_clipped_rect(rect);
        if hovered {
            self.state.hot = Some(id);
        }

        let mut response = SliderResponse {
            hovered,
            active: self.state.active == Some(id),
            changed: false,
        };
        if hovered && self.mouse_pressed() {
            self.state.active = Some(id);
            response.active = true;
        }
        if self.state.active == Some(id) && self.input.mouse_released {
            self.state.active = None;
            response.active = false;
        }
        response
    }

    /// Apply mouse-drag value updates while this slider is active.
    fn apply_slider_drag(
        &mut self,
        id: WidgetId,
        rect: Rect,
        range: (f32, f32),
        value: &mut f32,
    ) -> bool {
        if self.state.active != Some(id) || !self.input.mouse_down {
            return false;
        }
        let span = (range.1 - range.0).max(1.0e-6);
        let x = (self.input.pointer_pos.x - rect.origin.x) as f32;
        let t = (x / rect.size.width.max(1) as f32).clamp(0.0, 1.0);
        self.apply_slider_value(value, range, range.0 + t * span)
    }

    /// Apply wheel-based value updates when the slider is hovered.
    fn apply_slider_wheel(&mut self, hovered: bool, range: (f32, f32), value: &mut f32) -> bool {
        if !hovered || self.input.wheel_delta == 0.0 {
            return false;
        }
        let span = (range.1 - range.0).max(1.0e-6);
        let step = 0.02 * span;
        let next = (*value + step * self.input.wheel_delta.signum()).clamp(range.0, range.1);
        self.apply_slider_value(value, range, next)
    }

    /// Write a slider value if it changed by more than epsilon.
    fn apply_slider_value(&self, value: &mut f32, range: (f32, f32), next: f32) -> bool {
        let clamped = next.clamp(range.0, range.1);
        if (*value - clamped).abs() <= f32::EPSILON {
            return false;
        }
        *value = clamped;
        true
    }

    /// Resolve all geometry/colors needed to draw slider visuals.
    fn resolve_slider_visuals(
        &self,
        rect: Rect,
        height: i32,
        range: (f32, f32),
        value: f32,
        response: SliderResponse,
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
            fill: self.resolve_slider_fill_color(response),
        }
    }

    /// Resolve slider track fill color from interaction state.
    fn resolve_slider_fill_color(&self, response: SliderResponse) -> Color {
        if response.active {
            self.theme.knob_active
        } else if response.hovered {
            self.theme.knob_hover
        } else {
            self.theme.knob_fill
        }
    }

    /// Draw slider visuals from precomputed geometry.
    fn draw_slider_visuals(&mut self, visuals: SliderVisualState) {
        self.fill_rect_clipped(visuals.track_rect, visuals.fill);
        self.stroke_rect_clipped(visuals.track_rect, 1, self.theme.knob_outline);
        self.fill_rect_clipped(visuals.fill_rect, self.theme.knob_indicator);
        self.canvas.fill_circle(
            visuals.handle_center,
            visuals.handle_radius,
            self.theme.knob_indicator,
        );
    }

    /// Advance layout cursor after drawing a slider block.
    fn advance_slider_cursor(&mut self, rect: Rect, block_size: Size) {
        self.layout.cursor.y = rect.origin.y + block_size.height as i32 + self.layout.spacing;
    }

    /// Draw a horizontal slider with a stable key and a dynamic label.
    pub fn slider_with_key(
        &mut self,
        key: &str,
        label: &str,
        value: &mut f32,
        config: SliderConfig,
    ) -> SliderResponse {
        let id = WidgetId::from_label(key);
        self.slider(id, label, value, config)
    }

    /// Render a slider in a fixed rectangle without affecting surrounding layout.
    pub(crate) fn slider_in_rect(
        &mut self,
        value: &mut f32,
        request: SliderRectRenderRequest<'_>,
    ) -> SliderResponse {
        let previous = *self.layout;
        self.layout.cursor = request.rect.origin;
        let previous_text_scale = self.theme.text_scale;
        self.theme.text_scale = request.text_scale;
        let response = {
            let mut response = SliderResponse::default();
            self.with_clip(request.rect, |ui| {
                response = ui.slider(
                    request.id,
                    request.label,
                    value,
                    SliderConfig {
                        range: request.range,
                        size: Size {
                            width: request.rect.size.width.max(1),
                            height: request.control_size.height.max(1),
                        },
                    },
                );
            });
            response
        };
        self.theme.text_scale = previous_text_scale;
        *self.layout = previous;
        response
    }

    /// Render a slider in a fixed rectangle with an explicit text scale.
    pub(crate) fn slider_in_rect_scaled(
        &mut self,
        value: &mut f32,
        request: SliderRectRenderRequest<'_>,
    ) -> SliderResponse {
        self.slider_in_rect(value, request)
    }
}
