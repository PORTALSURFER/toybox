impl Canvas {
    /// Draw an arc with the given thickness between two angles (in radians).
    ///
    /// Angles are measured in standard mathematical coordinates, where 0 is to
    /// the right and positive values rotate counter-clockwise. If `end_angle`
    /// is less than `start_angle`, the arc wraps across 2π.
    pub fn stroke_arc(
        &mut self,
        center: Point,
        radius: i32,
        thickness: i32,
        start_angle: f32,
        end_angle: f32,
        color: Color,
    ) {
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
        let start = normalize_angle(start_angle);
        let end = normalize_angle(end_angle);

        for y in y0..=y1 {
            for x in x0..=x1 {
                let dx = x - center.x;
                let dy = y - center.y;
                let dist2 = dx * dx + dy * dy;
                if dist2 <= outer && dist2 >= inner2 {
                    // Convert screen-space Y (downward-positive) to mathematical
                    // coordinates so angles remain counter-clockwise.
                    let angle = normalize_angle((-(dy as f32)).atan2(dx as f32));
                    if angle_in_range(angle, start, end) {
                        self.blend_pixel(x as u32, y as u32, color);
                    }
                }
            }
        }
    }
}

/// Normalize an angle to the `[0, 2π)` range.
fn normalize_angle(angle: f32) -> f32 {
    let mut normalized = angle % std::f32::consts::TAU;
    if normalized < 0.0 {
        normalized += std::f32::consts::TAU;
    }
    normalized
}

/// Return true when `angle` lies in the inclusive start/end arc interval.
fn angle_in_range(angle: f32, start: f32, end: f32) -> bool {
    if start <= end {
        angle >= start && angle <= end
    } else {
        angle >= start || angle <= end
    }
}
