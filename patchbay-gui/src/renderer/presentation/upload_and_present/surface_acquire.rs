impl Renderer {
    /// Acquire the next surface frame, reconfiguring once on recoverable errors.
    fn acquire_surface_texture(&mut self) -> Result<wgpu::SurfaceTexture, GuiError> {
        match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(output) => Ok(output),
            status => self.acquire_surface_texture_after_error(status),
        }
    }

    /// Retry surface acquisition after an initial get-current-texture failure.
    fn acquire_surface_texture_after_error(
        &mut self,
        err: wgpu::CurrentSurfaceTexture,
    ) -> Result<wgpu::SurfaceTexture, GuiError> {
        if should_reconfigure_surface(&err) {
            let recreate_surface = should_recreate_surface(&err);
            let was_suboptimal = matches!(&err, wgpu::CurrentSurfaceTexture::Suboptimal(_));
            drop(err);
            if was_suboptimal {
                log_line_safe(
                    "renderer: discarded suboptimal surface texture before reconfigure",
                );
            }
            if recreate_surface {
                let surface = Self::create_surface(&self.device, self.surface_window.clone())?;
                surface.configure(&self.device.device, &self.config);
                self.surface = surface;
            } else {
                self.surface.configure(&self.device.device, &self.config);
            }
            return self.acquire_surface_texture_after_reconfigure();
        }
        log_line_safe(&format!("renderer: get_current_texture error: {err:?}"));
        Err(GuiError::SurfaceAcquire)
    }

    /// Acquire once after reconfiguring, never presenting a second suboptimal frame.
    fn acquire_surface_texture_after_reconfigure(
        &self,
    ) -> Result<wgpu::SurfaceTexture, GuiError> {
        match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(output) => Ok(output),
            wgpu::CurrentSurfaceTexture::Suboptimal(output) => {
                drop(output);
                log_line_safe(
                    "renderer: get_current_texture after reconfigure remained suboptimal",
                );
                Err(GuiError::SurfaceAcquire)
            }
            retry_status => {
                log_line_safe(&format!(
                    "renderer: get_current_texture after reconfigure error: {retry_status:?}"
                ));
                Err(GuiError::SurfaceAcquire)
            }
        }
    }

    /// Create a default texture view for a surface output frame.
    fn surface_view(output: &wgpu::SurfaceTexture) -> wgpu::TextureView {
        output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default())
    }
}
