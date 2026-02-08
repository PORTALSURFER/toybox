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
        f: F,
    ) -> RootFrameResponse
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
                origin,
                background: style.background.unwrap_or(self.theme.knob_fill),
                outline: style.outline.unwrap_or(self.theme.knob_outline),
            },
            ExplicitSizePolicy::PreferExplicit,
            ContainerFrameEffects {
                advance_layout_cursor: false,
                update_root_frame_size: true,
            },
            f,
        );
        RootFrameResponse {
            outer_rect: result.outer_rect,
            content_rect: result.content_rect,
            measured_size: result.measured_size,
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
