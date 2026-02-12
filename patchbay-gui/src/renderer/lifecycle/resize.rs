impl Renderer {
    /// Resize the surface and backing Vello render target.
    pub fn resize(&mut self, size: Size) {
        self.config.width = size.width.max(1);
        self.config.height = size.height.max(1);
        self.surface.configure(&self.device.device, &self.config);
        let (texture, view) = Self::create_render_target(
            &self.device.device,
            Size {
                width: self.config.width,
                height: self.config.height,
            },
        );
        self.render_target_texture = texture;
        self.render_target_view = view;
    }
}
