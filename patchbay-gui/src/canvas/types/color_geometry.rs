/// Packed RGBA color in sRGB space.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
    /// Alpha channel.
    pub a: u8,
}

impl Color {
    /// Create an opaque color from RGB values.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Create a color from RGBA values.
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

/// 2D pixel coordinate.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    /// X coordinate in pixels.
    pub x: i32,
    /// Y coordinate in pixels.
    pub y: i32,
}

/// 2D subpixel coordinate used by vector-rendered geometry.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PointF {
    /// X coordinate in pixels.
    pub x: f32,
    /// Y coordinate in pixels.
    pub y: f32,
}

/// 2D size in pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Size {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

/// Rectangle in pixel coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    /// Top-left corner in pixels.
    pub origin: Point,
    /// Size in pixels.
    pub size: Size,
}

impl Rect {
    /// Returns true if the point lies inside the rectangle.
    pub fn contains(&self, point: Point) -> bool {
        let x0 = self.origin.x;
        let y0 = self.origin.y;
        let x1 = x0 + self.size.width as i32;
        let y1 = y0 + self.size.height as i32;
        point.x >= x0 && point.x < x1 && point.y >= y0 && point.y < y1
    }
}
