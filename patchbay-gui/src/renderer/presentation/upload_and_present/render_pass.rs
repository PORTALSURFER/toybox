impl Renderer {
    /// Render the uploaded canvas texture to the current surface frame.
    pub fn render(&mut self) -> Result<(), GuiError> {
        crate::renderer::frame_capture_trace("Renderer::render: begin");
        let output = self.acquire_surface_texture()?;
        crate::renderer::frame_capture_trace("Renderer::render: surface texture acquired");
        let surface_view = Self::surface_view(&output);
        self.scene.reset();
        let scene_transform = self.resolve_scene_transform();
        self.scene.draw_image(&self.canvas_image, scene_transform);
        self.vector_painter.append_to_scene(
            &mut self.scene,
            &self.vector_commands,
            scene_transform,
        );

        crate::renderer::frame_capture_trace("Renderer::render: Vello render begin");
        self.render_scene_to_target()?;
        crate::renderer::frame_capture_trace("Renderer::render: Vello render completed");
        self.present_output(&surface_view, output);
        crate::renderer::frame_capture_trace("Renderer::render: presented");
        Ok(())
    }

    /// Resolve the affine transform used to map canvas coordinates to the surface.
    fn resolve_scene_transform(&self) -> Affine {
        let transform = self.presentation_transform.unwrap_or_else(|| {
            PresentationTransform::stretch(self.config.width, self.config.height, self.canvas_size)
        });
        Affine::new([
            transform.scale_x as f64,
            0.0,
            0.0,
            transform.scale_y as f64,
            transform.offset_x as f64,
            transform.offset_y as f64,
        ])
    }

    /// Render the assembled scene into the intermediate render target texture.
    fn render_scene_to_target(&mut self) -> Result<(), GuiError> {
        self.vello_renderer
            .render_to_texture(
                &self.device.device,
                &self.device.queue,
                &self.scene,
                &self.render_target_view,
                &RenderParams {
                    base_color: VelloColor::BLACK,
                    width: self.config.width,
                    height: self.config.height,
                    antialiasing_method: AaConfig::Area,
                },
            )
            .map_err(|err| {
                log_line_safe(&format!("renderer: vello render_to_texture error: {err:?}"));
                GuiError::SurfaceAcquire
            })
    }
}
