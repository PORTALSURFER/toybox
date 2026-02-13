impl Renderer {
    /// Create a new renderer for the given window with a shared device.
    pub fn new_with_device(
        device: Arc<RendererDevice>,
        window: SurfaceWindow,
        size: Size,
    ) -> Result<Self, GuiError> {
        log_line_safe("renderer: new begin");
        let surface = Self::create_surface(&device, window)?;
        log_line_safe("renderer: surface created");
        let config = Self::build_surface_config(&surface, &device, size);
        surface.configure(&device.device, &config);
        log_line_safe("renderer: surface configured");

        let resources = Self::initialize_renderer_resources(&device, &config, size)
            .map_err(map_vello_init_error)?;
        Ok(Self::build_renderer_instance(
            device, surface, config, resources,
        ))
    }

    /// Build a renderer instance from fully-initialized rendering resources.
    fn build_renderer_instance(
        device: Arc<RendererDevice>,
        surface: wgpu::Surface<'static>,
        config: wgpu::SurfaceConfiguration,
        resources: RendererInitResources,
    ) -> Self {
        Self {
            device,
            surface,
            config,
            vello_renderer: resources.vello_renderer,
            scene: vello::Scene::new(),
            render_target_texture: resources.render_target_texture,
            render_target_view: resources.render_target_view,
            blitter: resources.blitter,
            canvas_texture: resources.canvas_texture,
            canvas_image: resources.canvas_image,
            canvas_size: resources.initial_canvas_size,
            upload_scratch: Vec::new(),
            vector_painter: crate::vector::scene::VectorScenePainter::new(),
            vector_commands: Vec::new(),
            presentation_transform: None,
        }
    }
}
