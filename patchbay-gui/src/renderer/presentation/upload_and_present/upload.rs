impl Renderer {
    /// Upload the latest canvas pixels to the GPU texture used by Vello.
    pub fn upload(&mut self, size: Size, pixels: &[u8]) {
        let size = Size {
            width: size.width.max(1),
            height: size.height.max(1),
        };
        self.ensure_canvas_texture(size);

        let bytes_per_pixel = 4u32;
        let bytes_per_row = bytes_per_pixel * size.width;
        let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u32;
        let padded_bytes_per_row = ((bytes_per_row + alignment - 1) / alignment) * alignment;

        if padded_bytes_per_row == bytes_per_row {
            self.write_canvas_texture(size, pixels, bytes_per_row);
            return;
        }

        let required = (padded_bytes_per_row * size.height) as usize;
        Self::pad_rows_rgba(
            pixels,
            size.width,
            size.height,
            padded_bytes_per_row,
            &mut self.upload_scratch,
            required,
        );

        self.write_canvas_texture(size, &self.upload_scratch, padded_bytes_per_row);
    }

    /// Write pixels into the canvas texture using the given row stride.
    fn write_canvas_texture(&self, size: Size, pixels: &[u8], bytes_per_row: u32) {
        self.device.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.canvas_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(size.height),
            },
            wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Pad tightly packed RGBA rows to WGPU's row alignment requirement.
    pub(super) fn pad_rows_rgba(
        pixels: &[u8],
        width: u32,
        height: u32,
        padded_bytes_per_row: u32,
        scratch: &mut Vec<u8>,
        required: usize,
    ) {
        scratch.resize(required, 0);
        scratch.fill(0);
        let src_row = (width as usize) * 4;
        let dst_row = padded_bytes_per_row as usize;
        for row in 0..height as usize {
            let src_offset = row * src_row;
            let dst_offset = row * dst_row;
            scratch[dst_offset..dst_offset + src_row]
                .copy_from_slice(&pixels[src_offset..src_offset + src_row]);
        }
    }
}
