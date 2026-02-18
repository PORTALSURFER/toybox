impl<'a> Ui<'a> {
    /// Resolve knob geometry using the current layout cursor and knob size.
    fn resolve_knob_geometry_for_cursor(&self, labels: KnobLabels<'_>) -> KnobGeometry {
        let knob_size = self.layout.knob_size.max(1);
        self.resolve_knob_geometry_at_origin(labels, self.layout.cursor, knob_size)
    }

    /// Resolve knob geometry for a specific origin and knob side length.
    fn resolve_knob_geometry_at_origin(
        &self,
        labels: KnobLabels<'_>,
        origin: Point,
        knob_size: i32,
    ) -> KnobGeometry {
        let block_size = self.knob_block_size(labels.name, labels.value);
        let label_height = knob_label_height(self.theme.text_scale) as i32;
        let label_gap = knob_label_gap(self.theme.text_scale) as i32;
        let block_rect = Rect { origin, size: block_size };
        let knob_x_offset = ((block_size.width as i32 - knob_size) / 2).max(0);
        let knob_origin = Point {
            x: origin.x + knob_x_offset,
            y: origin.y + label_height + label_gap,
        };
        let knob_rect = Rect {
            origin: knob_origin,
            size: Size {
                width: knob_size as u32,
                height: knob_size as u32,
            },
        };
        let hit_rect = Rect {
            origin: Point {
                x: knob_rect.origin.x - KNOB_BLOCK_SIDE_PADDING,
                y: knob_rect.origin.y - KNOB_BLOCK_SIDE_PADDING,
            },
            size: Size {
                width: (knob_size + KNOB_BLOCK_SIDE_PADDING * 2).max(1) as u32,
                height: (knob_size + KNOB_BLOCK_SIDE_PADDING * 2).max(1) as u32,
            },
        };
        let center = Point {
            x: knob_rect.origin.x + knob_size / 2,
            y: knob_rect.origin.y + knob_size / 2,
        };
        let radius = (knob_size / 2 - 4).max(1);
        KnobGeometry {
            block_rect,
            knob_rect,
            hit_rect,
            center,
            radius,
            knob_size,
            label_gap,
        }
    }

    /// Render helper rectangles used for layout bounds and debug visualization.
    fn draw_knob_bounds(&mut self, geometry: KnobGeometry) {
        self.stroke_rect_clipped(geometry.block_rect, 1, self.theme.knob_indicator);
        self.track_rect_internal(geometry.block_rect);
    }

    /// Advance layout after rendering one knob block.
    fn advance_layout_after_knob(&mut self, geometry: KnobGeometry) {
        let block_height = geometry.block_rect.size.height as i32;
        self.layout.cursor.y += block_height + self.layout.spacing;
    }

    /// Resolve a clamped knob diameter for rectangle-scoped rendering.
    fn resolve_knob_size_for_rect(&self, rect: Rect, desired_diameter: u32) -> i32 {
        let label_height = knob_label_height(self.theme.text_scale) as i32;
        let label_gap = knob_label_gap(self.theme.text_scale) as i32;
        let side_padding = KNOB_BLOCK_SIDE_PADDING.max(0);
        let available_height = (rect.size.height as i32 - label_height * 2 - label_gap * 2).max(1);
        let available_width = (rect.size.width as i32 - side_padding * 2).max(1);
        (desired_diameter.max(1) as i32)
            .min(available_width)
            .min(available_height)
            .max(1)
    }
}
