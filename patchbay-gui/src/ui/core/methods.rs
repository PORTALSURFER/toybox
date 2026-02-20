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
        }
    }

    /// Enable or disable vector text emission for this frame.
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    pub(crate) fn set_vector_text_enabled(&mut self, enabled: bool) {
        self.vector_text_enabled = enabled;
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
        self.clipped_rect(rect)
            .map(|clipped| clipped.contains(self.input.pointer_pos))
            .unwrap_or(false)
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
