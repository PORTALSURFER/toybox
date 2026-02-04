//! CPU-side drawing surface used by the GUI renderer.

use std::cmp::{max, min};

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

/// CPU-side RGBA canvas with simple drawing helpers.
pub struct Canvas {
    size: Size,
    pixels: Vec<u8>,
}

impl Canvas {
    /// Create a new canvas with the given dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        let size = Size { width, height };
        let mut canvas = Self {
            size,
            pixels: vec![0; (width as usize) * (height as usize) * 4],
        };
        canvas.clear(Color::rgba(0, 0, 0, 255));
        canvas
    }

    /// Returns the current canvas size.
    pub fn size(&self) -> Size {
        self.size
    }

    /// Returns the raw RGBA pixel buffer.
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// Resize the canvas, discarding previous contents.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.size = Size { width, height };
        self.pixels
            .resize((width as usize) * (height as usize) * 4, 0);
    }

    /// Fill the entire canvas with a color.
    pub fn clear(&mut self, color: Color) {
        for chunk in self.pixels.chunks_exact_mut(4) {
            chunk[0] = color.r;
            chunk[1] = color.g;
            chunk[2] = color.b;
            chunk[3] = color.a;
        }
    }

    /// Draw a filled rectangle.
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        let x0 = max(rect.origin.x, 0) as u32;
        let y0 = max(rect.origin.y, 0) as u32;
        let x1 = min(rect.origin.x + rect.size.width as i32, self.size.width as i32) as u32;
        let y1 = min(rect.origin.y + rect.size.height as i32, self.size.height as i32) as u32;

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
        let r2 = (radius * radius) as i32;
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
                    let angle = normalize_angle((dy as f32).atan2(dx as f32));
                    if angle_in_range(angle, start, end) {
                        self.blend_pixel(x as u32, y as u32, color);
                    }
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
            if x0 >= 0 && y0 >= 0 && (x0 as u32) < self.size.width && (y0 as u32) < self.size.height {
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

    /// Draw a text string using the builtin bitmap font.
    pub fn draw_text(&mut self, origin: Point, text: &str, color: Color, scale: u32) {
        let scale = scale.max(1) as i32;
        let mut cursor = origin;
        for ch in text.chars() {
            if ch == '\n' {
                cursor.x = origin.x;
                cursor.y += 8 * scale;
                continue;
            }
            let glyph = BitmapFont::glyph(ch);
            for (row, bits) in glyph.iter().enumerate() {
                for col in 0..5 {
                    if (bits >> (4 - col)) & 1 == 1 {
                        let x = cursor.x + col as i32 * scale;
                        let y = cursor.y + row as i32 * scale;
                        for dy in 0..scale {
                            for dx in 0..scale {
                                let px = x + dx;
                                let py = y + dy;
                                if px >= 0
                                    && py >= 0
                                    && (px as u32) < self.size.width
                                    && (py as u32) < self.size.height
                                {
                                    self.blend_pixel(px as u32, py as u32, color);
                                }
                            }
                        }
                    }
                }
            }
            cursor.x += 6 * scale;
        }
    }

    fn blend_pixel(&mut self, x: u32, y: u32, color: Color) {
        let idx = ((y as usize) * (self.size.width as usize) + (x as usize)) * 4;
        let dst = &mut self.pixels[idx..idx + 4];
        let src_a = color.a as u32;
        let inv_a = 255 - src_a;
        dst[0] = ((color.r as u32 * src_a + dst[0] as u32 * inv_a) / 255) as u8;
        dst[1] = ((color.g as u32 * src_a + dst[1] as u32 * inv_a) / 255) as u8;
        dst[2] = ((color.b as u32 * src_a + dst[2] as u32 * inv_a) / 255) as u8;
        dst[3] = min(255, (dst[3] as u32 + src_a) as u32) as u8;
    }
}

fn normalize_angle(angle: f32) -> f32 {
    let mut normalized = angle % std::f32::consts::TAU;
    if normalized < 0.0 {
        normalized += std::f32::consts::TAU;
    }
    normalized
}

fn angle_in_range(angle: f32, start: f32, end: f32) -> bool {
    if start <= end {
        angle >= start && angle <= end
    } else {
        angle >= start || angle <= end
    }
}

struct BitmapFont;

impl BitmapFont {
    fn glyph(ch: char) -> [u8; 7] {
        match ch {
            '0' => [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
            '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
            '2' => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111],
            '3' => [0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110],
            '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
            '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
            '6' => [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
            '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
            '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
            '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
            'A' | 'a' => [0b00100, 0b01010, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
            'B' | 'b' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
            'C' | 'c' => [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
            'D' | 'd' => [0b11100, 0b10010, 0b10001, 0b10001, 0b10001, 0b10010, 0b11100],
            'E' | 'e' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
            'F' | 'f' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
            'G' | 'g' => [0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110],
            'H' | 'h' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
            'I' | 'i' => [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
            'J' | 'j' => [0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100],
            'K' | 'k' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
            'L' | 'l' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
            'M' | 'm' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
            'N' | 'n' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
            'O' | 'o' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
            'P' | 'p' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
            'Q' | 'q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
            'R' | 'r' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
            'S' | 's' => [0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110],
            'T' | 't' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
            'U' | 'u' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
            'V' | 'v' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
            'W' | 'w' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010],
            'X' | 'x' => [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
            'Y' | 'y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
            'Z' | 'z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
            '-' => [0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000],
            '_' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111],
            '.' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100],
            ':' => [0b00000, 0b01100, 0b01100, 0b00000, 0b01100, 0b01100, 0b00000],
            '/' => [0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b00000, 0b00000],
            ' ' => [0; 7],
            _ => [0b00000, 0b00100, 0b00000, 0b00100, 0b00000, 0b00000, 0b00100],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel_at(canvas: &Canvas, x: u32, y: u32) -> [u8; 4] {
        let width = canvas.size.width as usize;
        let index = (y as usize * width + x as usize) * 4;
        [
            canvas.pixels[index],
            canvas.pixels[index + 1],
            canvas.pixels[index + 2],
            canvas.pixels[index + 3],
        ]
    }

    #[test]
    fn rect_contains_point() {
        let rect = Rect {
            origin: Point { x: 10, y: 20 },
            size: Size {
                width: 100,
                height: 50,
            },
        };
        assert!(rect.contains(Point { x: 10, y: 20 }));
        assert!(rect.contains(Point { x: 109, y: 69 }));
        assert!(!rect.contains(Point { x: 110, y: 70 }));
    }

    #[test]
    fn draw_text_advances_cursor() {
        let mut canvas = Canvas::new(64, 64);
        canvas.draw_text(Point { x: 0, y: 0 }, "AB", Color::rgb(255, 255, 255), 1);
        assert!(canvas.pixels().iter().any(|value| *value != 0));
    }

    #[test]
    fn stroke_arc_renders_top_semicircle() {
        let mut canvas = Canvas::new(21, 21);
        let center = Point { x: 10, y: 10 };
        let color = Color::rgb(200, 100, 50);
        canvas.stroke_arc(center, 8, 2, 0.0, std::f32::consts::PI, color);

        let top = pixel_at(&canvas, 10, 2);
        let bottom = pixel_at(&canvas, 10, 18);

        assert_eq!(top, [color.r, color.g, color.b, color.a]);
        assert_ne!(bottom, [color.r, color.g, color.b, color.a]);
    }
}
