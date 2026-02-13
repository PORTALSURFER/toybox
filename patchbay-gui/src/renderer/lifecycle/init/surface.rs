impl Renderer {
    /// Create a WGPU surface for the given window and map errors to GUI errors.
    fn create_surface(
        device: &RendererDevice,
        window: SurfaceWindow,
    ) -> Result<wgpu::Surface<'static>, GuiError> {
        device.instance.create_surface(window).map_err(|err| {
            log_line_safe(&format!("renderer: create_surface error: {err:?}"));
            GuiError::Surface(err)
        })
    }

    /// Return a normalized size where both dimensions are at least one pixel.
    fn normalized_size(size: Size) -> Size {
        Size {
            width: size.width.max(1),
            height: size.height.max(1),
        }
    }

    /// Build the WGPU surface configuration from surface capabilities.
    fn build_surface_config(
        surface: &wgpu::Surface<'static>,
        device: &RendererDevice,
        size: Size,
    ) -> wgpu::SurfaceConfiguration {
        let capabilities = surface.get_capabilities(&device.adapter);
        let format = Self::select_surface_format(&capabilities);
        let present_mode = Self::select_surface_present_mode(&capabilities);
        wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        }
    }

    /// Select the preferred surface format from adapter capabilities.
    fn select_surface_format(capabilities: &wgpu::SurfaceCapabilities) -> wgpu::TextureFormat {
        capabilities
            .formats
            .iter()
            .copied()
            .find(|candidate| {
                matches!(
                    candidate,
                    wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Bgra8Unorm
                )
            })
            .unwrap_or_else(|| capabilities.formats[0])
    }

    /// Select the preferred present mode from adapter capabilities.
    fn select_surface_present_mode(capabilities: &wgpu::SurfaceCapabilities) -> wgpu::PresentMode {
        capabilities
            .present_modes
            .iter()
            .copied()
            .find(|mode| *mode == wgpu::PresentMode::Fifo)
            .unwrap_or(capabilities.present_modes[0])
    }

    /// Create the intermediate Vello render target texture and view.
    fn create_render_target(
        device: &wgpu::Device,
        size: Size,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("patchbay-gui-vello-target"),
            size: wgpu::Extent3d {
                width: size.width.max(1),
                height: size.height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }
}
