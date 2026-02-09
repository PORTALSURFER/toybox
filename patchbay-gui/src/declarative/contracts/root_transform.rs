/// Root-level transform that maps design-space coordinates to the host surface.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RootTransform {
    /// X-axis scale from design space to surface space.
    pub scale_x: f32,
    /// Y-axis scale from design space to surface space.
    pub scale_y: f32,
    /// X-axis surface offset in pixels after scaling.
    pub offset_x: f32,
    /// Y-axis surface offset in pixels after scaling.
    pub offset_y: f32,
    /// Design-space content rectangle before transform.
    pub content_rect_design: Rect,
    /// Surface-space content rectangle after transform.
    pub content_rect_surface: Rect,
}

impl RootTransform {
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    /// Map a point from surface space back into design space.
    pub(crate) fn surface_to_design(&self, point: Point) -> Point {
        let inv_x = if self.scale_x.abs() <= f32::EPSILON {
            1.0
        } else {
            1.0 / self.scale_x
        };
        let inv_y = if self.scale_y.abs() <= f32::EPSILON {
            1.0
        } else {
            1.0 / self.scale_y
        };
        Point {
            x: ((point.x as f32 - self.offset_x) * inv_x).round() as i32,
            y: ((point.y as f32 - self.offset_y) * inv_y).round() as i32,
        }
    }
}
