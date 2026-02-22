impl<'a> Ui<'a> {
    /// Create a UI frame tied to the given canvas and input snapshot.
    pub fn new(
        canvas: &'a mut Canvas,
        input: &'a InputState,
        state: &'a mut UiState,
        layout: &'a mut Layout,
        theme: &'a Theme,
    ) -> Self {
        Self {
            canvas,
            input,
            state,
            layout,
            theme: theme.clone(),
            layout_stack: Vec::new(),
            clip_stack: Vec::new(),
            bounds_stack: Vec::new(),
            vector_commands: Vec::new(),
            vector_text_enabled: false,
            vector_shapes_enabled: false,
        }
    }

    /// Enable or disable vector text emission for this frame.
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    pub(crate) fn set_vector_text_enabled(&mut self, enabled: bool) {
        self.vector_text_enabled = enabled;
    }

    /// Enable or disable vector shape emission for this frame.
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    pub(crate) fn set_vector_shapes_enabled(&mut self, enabled: bool) {
        self.vector_shapes_enabled = enabled;
    }

    /// Drain queued vector commands for renderer submission.
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    pub(crate) fn take_vector_commands(&mut self) -> Vec<VectorCommand> {
        std::mem::take(&mut self.vector_commands)
    }

    /// Access the current layout cursor.
    pub fn cursor(&self) -> Point {
        self.layout.cursor
    }

    /// Set the layout cursor position.
    pub fn set_cursor(&mut self, cursor: Point) {
        self.layout.cursor = cursor;
    }

    /// Advance the cursor vertically.
    pub fn advance_y(&mut self, amount: i32) {
        self.layout.cursor.y += amount;
    }

    /// Draw text through either the vector renderer or bitmap fallback path.
    fn draw_text_internal(&mut self, position: Point, text: &str, color: Color, scale: u32) {
        let scale = scale.max(1);
        if self.vector_text_enabled {
            self.vector_commands.push(VectorCommand::Text {
                origin: position,
                clip_rect: self.current_clip_rect(),
                text: text.to_string(),
                color,
                scale,
            });
            return;
        }
        self.canvas.draw_text(position, text, color, scale);
    }

    /// Fill a rectangle constrained by the current clip stack.
    fn fill_rect_clipped(&mut self, rect: Rect, color: Color) {
        if let Some(clipped) = self.clipped_rect(rect) {
            self.canvas.fill_rect(clipped, color);
        }
    }

    /// Stroke a rectangle constrained by the current clip stack.
    fn stroke_rect_clipped(&mut self, rect: Rect, thickness: u32, color: Color) {
        if let Some(clipped) = self.clipped_rect(rect) {
            self.canvas.stroke_rect(clipped, thickness, color);
        }
    }

    /// Fill a rectangle using vector antialiasing when available.
    ///
    /// Falls back to CPU raster fill and always respects the current clip stack.
    pub(crate) fn fill_rect_visual(&mut self, rect: Rect, color: Color) {
        let Some(clipped) = self.clipped_rect(rect) else {
            return;
        };
        if self.vector_shapes_enabled {
            self.vector_commands
                .push(VectorCommand::RectFill(RectVisual { rect: clipped, color }));
            return;
        }
        self.canvas.fill_rect(clipped, color);
    }

    /// Stroke a rectangle using vector antialiasing when available.
    ///
    /// Falls back to CPU raster stroke and always respects the current clip stack.
    pub(crate) fn stroke_rect_visual(
        &mut self,
        rect: Rect,
        thickness: f32,
        color: Color,
    ) {
        let Some(clipped) = self.clipped_rect(rect) else {
            return;
        };
        let thickness = thickness.max(1.0);
        if self.vector_shapes_enabled {
            self.vector_commands.push(VectorCommand::RectStroke(RectStrokeVisual {
                rect: clipped,
                thickness,
                color,
            }));
            return;
        }
        self.canvas
            .stroke_rect(clipped, thickness.round().max(1.0) as u32, color);
    }

    /// Access the input snapshot for this frame.
    pub fn input(&self) -> &InputState {
        self.input
    }

    /// Return the current effective clip rectangle.
    fn current_clip_rect(&self) -> Option<Rect> {
        self.clip_stack.last().copied()
    }

    /// Intersect a rectangle with the current clip bounds.
    pub(crate) fn clipped_rect(&self, rect: Rect) -> Option<Rect> {
        if let Some(clip) = self.current_clip_rect() {
            rect_intersection(clip, rect)
        } else {
            Some(rect)
        }
    }

    /// Return true when the pointer lies inside the currently visible portion
    /// of `rect`.
    fn pointer_inside_clipped_rect(&self, rect: Rect) -> bool {
        if !self.input.pointer_in_window {
            return false;
        }
        self.clipped_rect(rect)
            .map(|clipped| clipped.contains(self.input.pointer_pos))
            .unwrap_or(false)
    }

    /// Return true when the pointer lies inside `rect`.
    ///
    /// Unlike `pointer_inside_clipped_rect`, this does not apply clip-stack
    /// intersection and is intended for overlay surfaces such as open dropdown
    /// menus that intentionally render outside control clip bounds.
    fn pointer_inside_rect(&self, rect: Rect) -> bool {
        self.input.pointer_in_window && rect.contains(self.input.pointer_pos)
    }

    /// Run a closure with an additional rectangular clip constraint.
    pub(crate) fn with_clip<F>(&mut self, rect: Rect, mut f: F)
    where
        F: FnMut(&mut Ui<'_>),
    {
        let next_clip = if let Some(parent) = self.current_clip_rect() {
            rect_intersection(parent, rect)
        } else {
            Some(rect)
        };
        let Some(next_clip) = next_clip else {
            return;
        };
        self.clip_stack.push(next_clip);
        f(self);
        self.clip_stack.pop();
    }

    /// Return the key pressed this frame, if any.
    pub fn key_pressed(&self) -> Option<char> {
        self.input.key_pressed
    }

    /// Access the canvas for custom drawing.
    pub fn canvas(&mut self) -> &mut Canvas {
        self.canvas
    }

    /// Draw a line with vector antialiasing when available.
    ///
    /// Falls back to CPU raster line drawing when vector shapes are disabled.
    pub(crate) fn draw_line_visual(
        &mut self,
        start: Point,
        end: Point,
        thickness: f32,
        color: Color,
    ) {
        if self.vector_shapes_enabled {
            self.vector_commands.push(VectorCommand::Line(LineVisual {
                start: PointF {
                    x: start.x as f32,
                    y: start.y as f32,
                },
                end: PointF {
                    x: end.x as f32,
                    y: end.y as f32,
                },
                thickness: thickness.max(1.0),
                color,
            }));
            return;
        }
        self.canvas.draw_line(start, end, color);
    }

    /// Return or initialize editable textbox runtime state for one stable key.
    pub(crate) fn begin_text_edit_runtime(
        &mut self,
        edit_key: &str,
        text_char_count: usize,
    ) -> TextEditRuntimeState {
        let id = WidgetId::from_label(edit_key);
        let entry = self
            .state
            .text_edit_runtime
            .entry(id)
            .or_insert(TextEditRuntimeState {
                cursor: text_char_count,
                anchor: text_char_count,
                pointer_selecting: false,
                cursor_pulse_frame: 0,
            });
        entry.cursor = entry.cursor.min(text_char_count);
        entry.anchor = entry.anchor.min(text_char_count);
        *entry
    }

    /// Persist editable textbox runtime state for one stable key.
    pub(crate) fn set_text_edit_runtime(
        &mut self,
        edit_key: &str,
        runtime: TextEditRuntimeState,
    ) {
        let id = WidgetId::from_label(edit_key);
        self.state.text_edit_runtime.insert(id, runtime);
    }

    /// Remove editable textbox runtime state for one stable key.
    pub(crate) fn clear_text_edit_runtime(&mut self, edit_key: &str) {
        let id = WidgetId::from_label(edit_key);
        self.state.text_edit_runtime.remove(&id);
    }

    /// Access the current theme settings.
    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Access the layout for custom sizing.
    pub fn layout_mut(&mut self) -> &mut Layout {
        self.layout
    }

    /// Push a new empty bounds union for nested layout tracking.
    fn push_bounds(&mut self) {
        self.bounds_stack.push(None);
    }

    /// Pop the current bounds union.
    fn pop_bounds(&mut self) -> Option<Rect> {
        self.bounds_stack.pop().flatten()
    }

    /// Merge a rendered rectangle into the current bounds union.
    fn track_rect_internal(&mut self, rect: Rect) {
        if let Some(entry) = self.bounds_stack.last_mut() {
            *entry = Some(match *entry {
                Some(existing) => rect_union(existing, rect),
                None => rect,
            });
        }
    }

    /// Track a rectangle so container sizing can include custom drawing.
    pub fn track_rect(&mut self, rect: Rect) {
        self.track_rect_internal(rect);
    }

    /// Stroke a debug border rectangle and include it in tracked bounds.
    pub(crate) fn debug_stroke_rect(&mut self, rect: Rect, thickness: u32, color: Color) {
        self.canvas.stroke_rect(rect, thickness, color);
        self.track_rect_internal(rect);
    }
}
