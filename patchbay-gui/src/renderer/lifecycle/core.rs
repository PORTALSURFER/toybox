//! Renderer lifecycle operations (create, resize, command updates).

use std::sync::Arc;

use crate::canvas::Size;
use crate::host::GuiError;
use crate::logging::log_line_safe;
use crate::vector::scene::VectorCommand;
use crate::win32::SurfaceWindow;
use vello::RendererOptions;

use super::{PresentationTransform, Renderer, RendererDevice, map_vello_init_error};

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
    ) -> vello::peniko::Image {
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

    /// Return true if vector text rendering is available.
    pub fn vector_text_available(&self) -> bool {
        self.vector_painter.has_text_font()
    }

    /// Replace the queued vector commands for the next render pass.
    pub fn set_vector_commands(&mut self, commands: Vec<VectorCommand>) {
        self.vector_commands = commands;
    }

    /// Set the surface transform used for the next render pass.
    pub fn set_presentation_transform(&mut self, transform: PresentationTransform) {
        self.presentation_transform = Some(transform);
    }

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

/// Fully initialized rendering resources used to construct a renderer instance.
struct RendererInitResources {
    /// GPU renderer used to rasterize the Vello scene.
    vello_renderer: vello::Renderer,
    /// Offscreen texture used as the Vello render target.
    render_target_texture: wgpu::Texture,
    /// View into the offscreen render target texture.
    render_target_view: wgpu::TextureView,
    /// Utility blitter used to copy to the swapchain texture.
    blitter: wgpu::util::TextureBlitter,
    /// Texture backing the uploaded CPU canvas.
    canvas_texture: wgpu::Texture,
    /// Vello image handle for the canvas texture.
    canvas_image: vello::peniko::Image,
    /// Initial logical canvas size.
    initial_canvas_size: Size,
}
