impl Canvas {
    /// Blit an RGBA pixel buffer into the given rectangle.
    ///
    /// The source buffer must contain `width * height * 4` bytes in RGBA order.
    /// The image is scaled to the destination rectangle using nearest-neighbor
    /// sampling.
    pub fn blit_rgba(&mut self, rect: Rect, pixels: &[u8], width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        let expected = (width as usize) * (height as usize) * 4;
        if pixels.len() < expected {
            return;
        }
        let dest_w = rect.size.width.max(1) as i32;
        let dest_h = rect.size.height.max(1) as i32;
        for dy in 0..dest_h {
            let src_y = (dy as u32 * height) / rect.size.height.max(1);
            for dx in 0..dest_w {
                let src_x = (dx as u32 * width) / rect.size.width.max(1);
                let idx = ((src_y * width + src_x) * 4) as usize;
                let color = Color::rgba(
                    pixels[idx],
                    pixels[idx + 1],
                    pixels[idx + 2],
                    pixels[idx + 3],
                );
                let x = rect.origin.x + dx;
                let y = rect.origin.y + dy;
                if x >= 0 && y >= 0 && (x as u32) < self.size.width && (y as u32) < self.size.height
                {
                    self.blend_pixel(x as u32, y as u32, color);
                }
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
                        let x = cursor.x + col * scale;
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
}
