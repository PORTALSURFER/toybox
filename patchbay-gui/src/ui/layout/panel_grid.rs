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
        f: F,
    ) -> PanelResponse
    where
        F: FnMut(&mut Ui<'_>, Rect),
    {
        let result = render_container_frame(
            self,
            ContainerFrameSpec {
                key,
                title: style.title,
                padding: style.padding,
                header_height: style.header_height,
                requested_size: size,
                origin: self.layout.cursor,
                background: style.background.unwrap_or(self.theme.knob_fill),
                outline: style.outline.unwrap_or(self.theme.knob_outline),
            },
            ExplicitSizePolicy::PreserveExplicitMinimum,
            ContainerFrameEffects {
                advance_layout_cursor: true,
                update_root_frame_size: false,
            },
            f,
        );
        PanelResponse {
            outer_rect: result.outer_rect,
            content_rect: result.content_rect,
            measured_size: result.measured_size,
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
