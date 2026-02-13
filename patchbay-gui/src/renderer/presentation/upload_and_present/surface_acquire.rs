impl Renderer {
    /// Acquire the next surface frame, reconfiguring once on recoverable errors.
    fn acquire_surface_texture(&self) -> Result<wgpu::SurfaceTexture, GuiError> {
        match self.surface.get_current_texture() {
            Ok(output) => Ok(output),
            Err(err) => self.acquire_surface_texture_after_error(err),
        }
    }

    /// Retry surface acquisition after an initial get-current-texture failure.
    fn acquire_surface_texture_after_error(
        &self,
        err: wgpu::SurfaceError,
    ) -> Result<wgpu::SurfaceTexture, GuiError> {
        if should_reconfigure_surface(&err) {
            self.surface.configure(&self.device.device, &self.config);
            return self.surface.get_current_texture().map_err(|retry_err| {
                log_line_safe(&format!(
                    "renderer: get_current_texture after reconfigure error: {retry_err:?}"
                ));
                GuiError::SurfaceAcquire(retry_err)
            });
        }
        log_line_safe(&format!("renderer: get_current_texture error: {err:?}"));
        Err(GuiError::SurfaceAcquire(err))
    }

    /// Create a default texture view for a surface output frame.
    fn surface_view(output: &wgpu::SurfaceTexture) -> wgpu::TextureView {
        output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default())
    }
}
