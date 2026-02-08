/// CPU-side RGBA canvas with simple drawing helpers.
pub struct Canvas {
    /// Current canvas dimensions.
    size: Size,
    /// Packed RGBA pixel data in row-major order.
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

    /// Alpha-blend a single source color into the destination pixel.
    fn blend_pixel(&mut self, x: u32, y: u32, color: Color) {
        let idx = ((y as usize) * (self.size.width as usize) + (x as usize)) * 4;
        let dst = &mut self.pixels[idx..idx + 4];
        let src_a = color.a as u32;
        let inv_a = 255 - src_a;
        dst[0] = ((color.r as u32 * src_a + dst[0] as u32 * inv_a) / 255) as u8;
        dst[1] = ((color.g as u32 * src_a + dst[1] as u32 * inv_a) / 255) as u8;
        dst[2] = ((color.b as u32 * src_a + dst[2] as u32 * inv_a) / 255) as u8;
        dst[3] = min(255, dst[3] as u32 + src_a) as u8;
    }
}
