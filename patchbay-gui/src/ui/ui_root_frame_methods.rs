impl<'a> Ui<'a> {

    /// Draw a root frame container sized to its contents.
    ///
    /// Root frames are the top-level container for a window. The measured size
    /// is stored for auto-resizing the native window each frame. When `size`
    /// is provided, it is treated as the pre-measured content size.
    pub fn root_frame_with_key<F>(
        &mut self,
        key: &str,
        style: RootFrameStyle<'_>,
        size: Option<Size>,
        mut f: F,
    ) -> RootFrameResponse
    where
        F: FnMut(&mut Ui<'_>, Rect),
    {
        self.root_frame_with_key_at(key, style, size, Point { x: 0, y: 0 }, |ui, rect| {
            f(ui, rect);
        })
    }

    /// Draw a root frame at an explicit origin.
    ///
    /// The measured root size is still reported for host auto-resize.
    pub fn root_frame_with_key_at<F>(
        &mut self,
        key: &str,
        style: RootFrameStyle<'_>,
        size: Option<Size>,
        origin: Point,
        mut f: F,
    ) -> RootFrameResponse
    where
        F: FnMut(&mut Ui<'_>, Rect),
    {
        let id = WidgetId::from_label(key);
        let header_height = style.header_height.unwrap_or_else(|| {
            if style.title.is_some() {
                (8 * self.theme.text_scale as i32 + 4).max(0)
            } else {
                0
            }
        });
        let padding = style.padding.max(0);
        let fallback = Size {
            width: (padding * 2 + 160).max(0) as u32,
            height: (padding * 2 + header_height + 80).max(0) as u32,
        };
        let requested_size = size;
        let cached = self.state.layout.get(id);
        let size = requested_size.or(cached).unwrap_or(fallback);
        let outer_rect = Rect { origin, size };
        let background = style.background.unwrap_or(self.theme.knob_fill);
        let outline = style.outline.unwrap_or(self.theme.knob_outline);

        self.fill_rect_clipped(outer_rect, background);
        self.stroke_rect_clipped(outer_rect, 1, outline);

        if let Some(title) = style.title {
            let title_pos = Point {
                x: origin.x + padding,
                y: origin.y + padding,
            };
            self.draw_text_internal(title_pos, title, self.theme.text, self.theme.text_scale);
            let title_size = text_size(title, self.theme.text_scale);
            self.track_rect_internal(Rect {
                origin: title_pos,
                size: title_size,
            });
        }

        let content_origin = Point {
            x: origin.x + padding,
            y: origin.y + padding + header_height,
        };
        let content_rect = Rect {
            origin: content_origin,
            size: Size {
                width: size.width.saturating_sub((padding * 2) as u32),
                height: size
                    .height
                    .saturating_sub((padding * 2 + header_height) as u32),
            },
        };

        self.push_bounds();
        self.with_layout(content_origin, |ui| f(ui, content_rect));
        let measured_bounds = self.pop_bounds();

        let measured_size = if let Some(bounds) = measured_bounds {
            let max_x = bounds.origin.x + bounds.size.width as i32;
            let max_y = bounds.origin.y + bounds.size.height as i32;
            let content_width = (max_x - content_origin.x).max(0) as u32;
            let content_height = (max_y - content_origin.y).max(0) as u32;
            Size {
                width: content_width + (padding * 2) as u32,
                height: content_height + (padding * 2 + header_height) as u32,
            }
        } else {
            Size {
                width: (padding * 2) as u32,
                height: (padding * 2 + header_height) as u32,
            }
        };
        let measured_size = match requested_size {
            Some(explicit) => explicit,
            None => measured_size,
        };

        self.state.layout.set(id, measured_size);
        self.track_rect_internal(outer_rect);
        self.state.set_root_frame_size(measured_size);

        RootFrameResponse {
            outer_rect,
            content_rect,
            measured_size,
        }
    }

    /// Draw a root frame with a stable default key.
    pub fn root_frame<F>(&mut self, style: RootFrameStyle<'_>, f: F) -> RootFrameResponse
    where
        F: FnMut(&mut Ui<'_>, Rect),
    {
        self.root_frame_with_key("__root_frame__", style, None, f)
    }
}
