/// Base point draw radius in design pixels.
const NODE_DRAW_RADIUS: i32 = 4;
/// Base playhead core radius in design pixels.
const PLAYHEAD_DOT_CORE_RADIUS: i32 = 4;
/// Base playhead ring radius in design pixels.
const PLAYHEAD_DOT_RING_RADIUS: i32 = 6;

impl<'a> Ui<'a> {
    /// Render one curve-editor visual frame.
    fn render_curve_editor_visuals(
        &mut self,
        model: &crate::declarative::CurveModel,
        rect: Rect,
        state: CurveEditorVisualState,
        style: crate::declarative::CurveEditorStyle,
        playhead_x: Option<f32>,
    ) {
        self.curve_fill_rect(rect, style.background);
        self.curve_stroke_rect(rect, 1.0, style.border);
        self.render_curve_grid(rect, style);
        self.render_curve_polyline(model, rect, state, style);
        self.render_curve_preview(rect, state, style);
        self.render_curve_points(model, rect, state, style);
        self.render_curve_playhead(model, rect, style, playhead_x);
        self.track_rect_internal(rect);
    }

    /// Draw background grid lines.
    fn render_curve_grid(&mut self, rect: Rect, style: crate::declarative::CurveEditorStyle) {
        for step in 1..16 {
            let x = rect.origin.x + ((rect.size.width as i32 - 1) * step) / 16;
            self.curve_stroke_line(
                Point { x, y: rect.origin.y },
                Point {
                    x,
                    y: rect.origin.y + rect.size.height as i32 - 1,
                },
                1.0,
                style.grid_vertical,
            );
        }
        for step in 1..4 {
            let y = rect.origin.y + ((rect.size.height as i32 - 1) * step) / 4;
            self.curve_stroke_line(
                Point { x: rect.origin.x, y },
                Point {
                    x: rect.origin.x + rect.size.width as i32 - 1,
                    y,
                },
                1.0,
                style.grid_horizontal,
            );
        }
    }

    /// Draw sampled curve segments.
    fn render_curve_polyline(
        &mut self,
        model: &crate::declarative::CurveModel,
        rect: Rect,
        state: CurveEditorVisualState,
        style: crate::declarative::CurveEditorStyle,
    ) {
        for segment_index in 0..model.segments.len() {
            let left = model.points[segment_index];
            let right = model.points[(segment_index + 1).min(model.points.len().saturating_sub(1))];
            let left_x = local_from_curve_point(crate::declarative::CurvePoint { x: left.x, y: 0.0 }, rect).x;
            let right_x =
                local_from_curve_point(crate::declarative::CurvePoint { x: right.x, y: 0.0 }, rect).x;
            let segment_width = (right_x - left_x).abs().max(2);
            let steps = segment_width.clamp(2, 128) as usize;
            let mut points = Vec::with_capacity(steps + 1);
            for step in 0..=steps {
                let t = step as f32 / steps as f32;
                let x = left.x + (right.x - left.x) * t;
                let y = sample_curve_model(model, x);
                let local = local_from_curve_point(crate::declarative::CurvePoint { x, y }, rect);
                points.push(Point {
                    x: rect.origin.x + local.x,
                    y: rect.origin.y + local.y,
                });
            }
            let highlighted = state.preview_point.is_none() && state.hovered_segment == Some(segment_index);
            let line_color = if highlighted {
                style.line_highlight
            } else {
                style.line
            };
            self.curve_stroke_polyline(&points, 1.3, line_color);
            if highlighted
                && matches!(style.highlight_mode, crate::declarative::CurveHighlightMode::BrightCircle)
            {
                let mid = points[points.len() / 2];
                self.curve_fill_circle(mid, scaled_curve_i32(5, rect) as f32, style.line_highlight);
            }
        }
    }

    /// Draw preview insertion point.
    fn render_curve_preview(
        &mut self,
        rect: Rect,
        state: CurveEditorVisualState,
        style: crate::declarative::CurveEditorStyle,
    ) {
        let Some(preview) = state.preview_point else {
            return;
        };
        let local = local_from_curve_point(preview, rect);
        let center = Point {
            x: rect.origin.x + local.x,
            y: rect.origin.y + local.y,
        };
        self.curve_fill_circle(center, scaled_curve_i32(NODE_DRAW_RADIUS + 1, rect) as f32, style.preview_fill);
        self.curve_stroke_circle(
            center,
            scaled_curve_i32(NODE_DRAW_RADIUS + 2, rect) as f32,
            1.0,
            style.preview_stroke,
        );
    }

    /// Draw points with selected/hover styling.
    fn render_curve_points(
        &mut self,
        model: &crate::declarative::CurveModel,
        rect: Rect,
        state: CurveEditorVisualState,
        style: crate::declarative::CurveEditorStyle,
    ) {
        for (index, point) in model.points.iter().copied().enumerate() {
            let local = local_from_curve_point(point, rect);
            let center = Point {
                x: rect.origin.x + local.x,
                y: rect.origin.y + local.y,
            };
            let selected = state.selected_point == Some(index);
            let hovered = state.hovered_point == Some(index);
            let fill = if selected {
                style.node_selected_fill
            } else if hovered {
                style.node_hover_fill
            } else {
                style.node_fill
            };
            let stroke = if selected {
                style.node_selected_stroke
            } else if hovered {
                style.node_hover_stroke
            } else {
                style.node_stroke
            };
            let radius = if selected || hovered {
                scaled_curve_i32(NODE_DRAW_RADIUS + 1, rect) as f32
            } else {
                scaled_curve_i32(NODE_DRAW_RADIUS, rect) as f32
            };
            self.curve_fill_circle(center, radius, fill);
            self.curve_stroke_circle(
                center,
                scaled_curve_i32(NODE_DRAW_RADIUS, rect) as f32,
                1.0,
                stroke,
            );
            if (selected || hovered)
                && matches!(style.highlight_mode, crate::declarative::CurveHighlightMode::BrightCircle)
            {
                self.curve_stroke_circle(
                    center,
                    scaled_curve_i32(NODE_DRAW_RADIUS + 3, rect) as f32,
                    1.0,
                    style.line_highlight,
                );
            }
        }
    }

    /// Draw optional playhead marker.
    fn render_curve_playhead(
        &mut self,
        model: &crate::declarative::CurveModel,
        rect: Rect,
        style: crate::declarative::CurveEditorStyle,
        playhead_x: Option<f32>,
    ) {
        let Some(playhead_x) = playhead_x else {
            return;
        };
        let x = playhead_x.clamp(0.0, 1.0);
        let y = sample_curve_model(model, x);
        let local = local_from_curve_point(crate::declarative::CurvePoint { x, y }, rect);
        let center = Point {
            x: rect.origin.x + local.x,
            y: rect.origin.y + local.y,
        };
        self.curve_fill_circle(center, scaled_curve_i32(PLAYHEAD_DOT_CORE_RADIUS, rect) as f32, style.playhead_core);
        self.curve_stroke_circle(
            center,
            scaled_curve_i32(PLAYHEAD_DOT_RING_RADIUS, rect) as f32,
            1.0,
            style.playhead_stroke,
        );
    }

    /// Draw a filled rectangle using vector path when enabled, else CPU canvas.
    fn curve_fill_rect(&mut self, rect: Rect, color: Color) {
        if self.vector_shapes_enabled {
            self.vector_commands
                .push(VectorCommand::RectFill(RectVisual { rect, color }));
            return;
        }
        if let Some(clipped) = self.clipped_rect(rect) {
            self.canvas.fill_rect(clipped, color);
        }
    }

    /// Draw a stroked rectangle using vector path when enabled, else CPU canvas.
    fn curve_stroke_rect(&mut self, rect: Rect, thickness: f32, color: Color) {
        if self.vector_shapes_enabled {
            self.vector_commands.push(VectorCommand::RectStroke(RectStrokeVisual {
                rect,
                thickness,
                color,
            }));
            return;
        }
        if let Some(clipped) = self.clipped_rect(rect) {
            self.canvas
                .stroke_rect(clipped, thickness.round().max(1.0) as u32, color);
        }
    }

    /// Draw a line using vector path when enabled, else CPU canvas.
    fn curve_stroke_line(&mut self, start: Point, end: Point, thickness: f32, color: Color) {
        if self.vector_shapes_enabled {
            self.vector_commands.push(VectorCommand::Line(LineVisual {
                start,
                end,
                thickness,
                color,
            }));
            return;
        }
        self.canvas.draw_line(start, end, color);
    }

    /// Draw a polyline using vector path when enabled, else CPU canvas.
    fn curve_stroke_polyline(&mut self, points: &[Point], thickness: f32, color: Color) {
        if points.len() < 2 {
            return;
        }
        if self.vector_shapes_enabled {
            self.vector_commands
                .push(VectorCommand::Polyline(PolylineVisual {
                    points: points.to_vec(),
                    thickness,
                    color,
                }));
            return;
        }
        for pair in points.windows(2) {
            if let [start, end] = pair {
                self.canvas.draw_line(*start, *end, color);
            }
        }
    }

    /// Draw a filled circle using vector path when enabled, else CPU canvas.
    fn curve_fill_circle(&mut self, center: Point, radius: f32, color: Color) {
        if self.vector_shapes_enabled {
            self.vector_commands
                .push(VectorCommand::CircleFill(CircleVisual {
                    center,
                    radius,
                    color,
                }));
            return;
        }
        self.canvas.fill_circle(center, radius.round().max(1.0) as i32, color);
    }

    /// Draw a stroked circle using vector path when enabled, else CPU canvas.
    fn curve_stroke_circle(&mut self, center: Point, radius: f32, thickness: f32, color: Color) {
        if self.vector_shapes_enabled {
            self.vector_commands
                .push(VectorCommand::CircleStroke(CircleStrokeVisual {
                    center,
                    radius,
                    thickness,
                    color,
                }));
            return;
        }
        self.canvas.stroke_circle(
            center,
            radius.round().max(1.0) as i32,
            thickness.round().max(1.0) as i32,
            color,
        );
    }
}
