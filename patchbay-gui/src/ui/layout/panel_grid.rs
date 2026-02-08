impl<'a> Ui<'a> {

    /// Draw a panel container with an optional title and padding.
    ///
    /// The panel can auto-size to fit its contents. When `size` is `None`, the
    /// panel uses the last measured size for the key and updates it after the
    /// closure runs. Auto-sized panels advance the layout cursor using the
    /// newly measured height so following widgets line up with the rendered
    /// content.
    pub fn panel_with_key<F>(
        &mut self,
        key: &str,
        style: PanelStyle<'_>,
        size: Option<Size>,
        mut f: F,
    ) -> PanelResponse
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
        let origin = self.layout.cursor;
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
            Some(explicit) => Size {
                width: explicit.width.max(measured_size.width),
                height: explicit.height.max(measured_size.height),
            },
            None => measured_size,
        };

        self.state.layout.set(id, measured_size);
        self.track_rect_internal(outer_rect);
        let advance_height = measured_size.height;
        self.layout.cursor.y = origin.y + advance_height as i32 + self.layout.spacing;

        PanelResponse {
            outer_rect,
            content_rect,
            measured_size,
        }
    }

    /// Draw a grid container and provide a helper for addressing cells.
    pub fn grid_with_key<F>(
        &mut self,
        _key: &str,
        spec: GridSpec,
        origin: Point,
        mut f: F,
    ) -> GridResponse
    where
        F: FnMut(&mut Ui<'_>, &mut GridContext),
    {
        let mut ctx = GridContext::new(origin, spec);
        f(self, &mut ctx);

        let rows = spec.rows.unwrap_or_else(|| {
            if ctx.max_index < 0 {
                0
            } else {
                (ctx.max_index / spec.columns.max(1)) + 1
            }
        });
        let columns = spec.columns.max(1);
        let width = if rows == 0 || columns == 0 {
            0
        } else {
            columns * spec.cell_size.width as i32 + (columns - 1) * spec.gap
        };
        let height = if rows == 0 || columns == 0 {
            0
        } else {
            rows * spec.cell_size.height as i32 + (rows - 1) * spec.gap
        };
        let bounds_rect = Rect {
            origin,
            size: Size {
                width: width.max(0) as u32,
                height: height.max(0) as u32,
            },
        };
        self.track_rect_internal(bounds_rect);

        GridResponse {
            bounds_rect,
            rows,
            columns,
        }
    }
}
