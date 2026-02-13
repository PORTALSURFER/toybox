impl Renderer {
    /// Create the shared Vello renderer instance.
    fn create_vello_renderer(device: &RendererDevice) -> Result<vello::Renderer, vello::Error> {
        vello::Renderer::new(&device.device, RendererOptions::default())
    }

    /// Create the initial offscreen render target from the surface config.
    fn create_initial_render_target(
        device: &RendererDevice,
        config: &wgpu::SurfaceConfiguration,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        Self::create_render_target(
            &device.device,
            Size {
                width: config.width,
                height: config.height,
            },
        )
    }

    /// Create the texture blitter used for final surface presentation.
    fn create_blitter(
        device: &RendererDevice,
        format: wgpu::TextureFormat,
    ) -> wgpu::util::TextureBlitter {
        wgpu::util::TextureBlitter::new(&device.device, format)
    }

    /// Register the CPU-upload canvas texture with Vello.
    fn register_canvas_texture(
        renderer: &mut vello::Renderer,
        canvas_texture: &wgpu::Texture,
    ) -> vello::peniko::ImageData {
        renderer.register_texture(canvas_texture.clone())
    }

    /// Initialize GPU resources needed for the first renderer frame.
    fn initialize_renderer_resources(
        device: &RendererDevice,
        config: &wgpu::SurfaceConfiguration,
        size: Size,
    ) -> Result<RendererInitResources, vello::Error> {
        let mut vello_renderer = Self::create_vello_renderer(device)?;
        let initial_canvas_size = Self::normalized_size(size);
        let (render_target_texture, render_target_view) =
            Self::create_initial_render_target(device, config);
        let blitter = Self::create_blitter(device, config.format);
        let canvas_texture = Self::create_canvas_texture(&device.device, initial_canvas_size);
        let canvas_image = Self::register_canvas_texture(&mut vello_renderer, &canvas_texture);
        Ok(RendererInitResources {
            vello_renderer,
            render_target_texture,
            render_target_view,
            blitter,
            canvas_texture,
            canvas_image,
            initial_canvas_size,
        })
    }
}
