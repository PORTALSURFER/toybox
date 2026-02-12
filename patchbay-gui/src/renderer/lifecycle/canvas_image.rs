impl Renderer {
    /// Ensure the canvas upload texture matches the current canvas size.
    pub(super) fn ensure_canvas_texture(&mut self, size: Size) {
        if self.canvas_size == size {
            return;
        }
        let old_image = self.canvas_image.clone();
        self.canvas_texture = Self::create_canvas_texture(&self.device.device, size);
        self.canvas_image = self
            .vello_renderer
            .register_texture(self.canvas_texture.clone());
        self.vello_renderer.unregister_texture(old_image);
        self.canvas_size = size;
    }

    /// Create the texture used to upload and sample CPU canvas pixels.
    fn create_canvas_texture(device: &wgpu::Device, size: Size) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("patchbay-gui-canvas-texture"),
            size: wgpu::Extent3d {
                width: size.width.max(1),
                height: size.height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
    }
}
