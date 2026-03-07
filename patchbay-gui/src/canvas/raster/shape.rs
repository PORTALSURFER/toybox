impl Canvas {
    /// Draw a filled rectangle.
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        let x0 = max(rect.origin.x, 0) as u32;
        let y0 = max(rect.origin.y, 0) as u32;
        let x1 = min(
            rect.origin.x + rect.size.width as i32,
            self.size.width as i32,
        ) as u32;
        let y1 = min(
            rect.origin.y + rect.size.height as i32,
            self.size.height as i32,
        ) as u32;

        for y in y0..y1 {
            for x in x0..x1 {
                self.blend_pixel(x, y, color);
            }
        }
    }

    /// Draw a rectangle outline with the given thickness.
    pub fn stroke_rect(&mut self, rect: Rect, thickness: u32, color: Color) {
        if thickness == 0 {
            return;
        }
        let t = thickness as i32;
        self.fill_rect(
            Rect {
                origin: rect.origin,
                size: Size {
                    width: rect.size.width,
                    height: thickness,
                },
            },
            color,
        );
        self.fill_rect(
            Rect {
                origin: Point {
                    x: rect.origin.x,
                    y: rect.origin.y + rect.size.height as i32 - t,
                },
                size: Size {
                    width: rect.size.width,
                    height: thickness,
                },
            },
            color,
        );
        self.fill_rect(
            Rect {
                origin: rect.origin,
                size: Size {
                    width: thickness,
                    height: rect.size.height,
                },
            },
            color,
        );
        self.fill_rect(
            Rect {
                origin: Point {
                    x: rect.origin.x + rect.size.width as i32 - t,
                    y: rect.origin.y,
                },
                size: Size {
                    width: thickness,
                    height: rect.size.height,
                },
            },
            color,
        );
    }

    /// Draw a filled circle.
    pub fn fill_circle(&mut self, center: Point, radius: i32, color: Color) {
        if radius <= 0 {
            return;
        }
        let r2 = radius * radius;
        let y0 = max(center.y - radius, 0);
        let y1 = min(center.y + radius, self.size.height as i32 - 1);
        let x0 = max(center.x - radius, 0);
        let x1 = min(center.x + radius, self.size.width as i32 - 1);

        for y in y0..=y1 {
            for x in x0..=x1 {
                let dx = x - center.x;
                let dy = y - center.y;
                if dx * dx + dy * dy <= r2 {
                    self.blend_pixel(x as u32, y as u32, color);
                }
            }
        }
    }

    /// Draw a ring with the given thickness.
    pub fn stroke_circle(&mut self, center: Point, radius: i32, thickness: i32, color: Color) {
        if radius <= 0 || thickness <= 0 {
            return;
        }
        let outer = radius * radius;
        let inner = (radius - thickness).max(0);
        let inner2 = inner * inner;
        let y0 = max(center.y - radius, 0);
        let y1 = min(center.y + radius, self.size.height as i32 - 1);
        let x0 = max(center.x - radius, 0);
        let x1 = min(center.x + radius, self.size.width as i32 - 1);

        for y in y0..=y1 {
            for x in x0..=x1 {
                let dx = x - center.x;
                let dy = y - center.y;
                let dist2 = dx * dx + dy * dy;
                if dist2 <= outer && dist2 >= inner2 {
                    self.blend_pixel(x as u32, y as u32, color);
                }
            }
        }
    }

    /// Draw a line using a basic Bresenham algorithm.
    pub fn draw_line(&mut self, start: Point, end: Point, color: Color) {
        let mut x0 = start.x;
        let mut y0 = start.y;
        let x1 = end.x;
        let y1 = end.y;
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            if x0 >= 0 && y0 >= 0 && (x0 as u32) < self.size.width && (y0 as u32) < self.size.height
            {
                self.blend_pixel(x0 as u32, y0 as u32, color);
            }
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    /// Fill a simple polygon using a scanline raster pass.
    pub fn fill_polygon(&mut self, points: &[Point], color: Color) {
        if points.len() < 3 {
            return;
        }

        let min_y = max(
            points.iter().map(|point| point.y).min().unwrap_or_default(),
            0,
        );
        let max_y = min(
            points.iter().map(|point| point.y).max().unwrap_or_default(),
            self.size.height.saturating_sub(1) as i32,
        );
        if min_y > max_y {
            return;
        }

        let mut intersections = Vec::with_capacity(points.len());
        for y in min_y..=max_y {
            intersections.clear();
            let scan_y = y as f32 + 0.5;
            for (start, end) in points
                .iter()
                .copied()
                .zip(points.iter().copied().cycle().skip(1))
                .take(points.len())
            {
                let y0 = start.y as f32;
                let y1 = end.y as f32;
                if (y0 <= scan_y && y1 > scan_y) || (y1 <= scan_y && y0 > scan_y) {
                    let t = (scan_y - y0) / (y1 - y0);
                    intersections.push(start.x as f32 + (end.x - start.x) as f32 * t);
                }
            }

            intersections.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            for pair in intersections.chunks_exact(2) {
                let x0 = max(pair[0].ceil() as i32, 0);
                let x1 = min(
                    pair[1].floor() as i32,
                    self.size.width.saturating_sub(1) as i32,
                );
                for x in x0..=x1 {
                    self.blend_pixel(x as u32, y as u32, color);
                }
            }
        }
    }
}
