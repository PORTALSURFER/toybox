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
        key: &str,
        spec: GridSpec,
        origin: Point,
        mut f: F,
    ) -> GridResponse
    where
        F: FnMut(&mut Ui<'_>, &mut GridContext),
    {
        let cache_key = WidgetId::from_label(key);
        let cached_rows = self
            .state
            .layout
            .get(cache_key)
            .map(|size| cached_grid_rows(size, spec));
        let mut ctx = GridContext::new(origin, spec);
        f(self, &mut ctx);
        let rows = resolve_grid_rows(spec, ctx.max_index, cached_rows);
        let columns = spec.columns.max(1);
        let bounds_rect = Rect {
            origin,
            size: grid_bounds_size(spec, rows, columns),
        };
        self.track_rect_internal(bounds_rect);
        self.state.layout.set(cache_key, bounds_rect.size);

        GridResponse {
            bounds_rect,
            rows,
            columns,
        }
    }
}

/// Resolve rendered row count for a keyed grid traversal.
fn resolve_grid_rows(spec: GridSpec, max_index: i32, cached_rows: Option<i32>) -> i32 {
    if let Some(rows) = spec.rows {
        return rows.max(0);
    }
    if max_index >= 0 {
        return (max_index / spec.columns.max(1)) + 1;
    }
    cached_rows.unwrap_or(0).max(0)
}

/// Derive previous grid row count from a cached bounds size.
fn cached_grid_rows(size: Size, spec: GridSpec) -> i32 {
    let columns = spec.columns.max(1);
    let grid_height = size.height as i32;
    let cell_h = spec.cell_size.height as i32;
    let span = cell_h + spec.gap;
    if grid_height <= 0 || columns <= 0 || span <= 0 {
        return 0;
    }
    ((grid_height + spec.gap).max(0) / span).max(0)
}

/// Resolve grid bounds size from row/column counts and track geometry.
fn grid_bounds_size(spec: GridSpec, rows: i32, columns: i32) -> Size {
    if rows <= 0 || columns <= 0 {
        return Size {
            width: 0,
            height: 0,
        };
    }
    let width = columns * spec.cell_size.width as i32 + (columns - 1) * spec.gap;
    let height = rows * spec.cell_size.height as i32 + (rows - 1) * spec.gap;
    Size {
        width: width.max(0) as u32,
        height: height.max(0) as u32,
    }
}
