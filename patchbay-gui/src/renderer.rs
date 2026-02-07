//! Vello-backed renderer that presents the CPU canvas to a window surface.

use crate::canvas::Size;
use crate::host::GuiError;
use crate::logging::log_line_safe;
use crate::vector_scene::{VectorCommand, VectorScenePainter};
use crate::win32::SurfaceWindow;
use std::sync::Arc;
use vello::kurbo::Affine;
use vello::peniko::{Color as VelloColor, ImageData};
use vello::{AaConfig, RenderParams, Renderer as VelloRenderer, RendererOptions, Scene};

/// Cached GPU device resources shared across window surfaces.
#[derive(Debug)]
pub struct RendererDevice {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl RendererDevice {
    /// Create a new device and queue without binding to a specific surface.
    pub fn new() -> Result<Self, GuiError> {
        log_line_safe("renderer_device: create begin");
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::from_env().unwrap_or(wgpu::Backends::PRIMARY),
            ..Default::default()
        });
        log_line_safe("renderer_device: instance created");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .map_err(|err| {
            log_line_safe(&format!("renderer_device: request_adapter error: {err:?}"));
            GuiError::AdapterNotFound
        })?;
        log_line_safe("renderer_device: adapter acquired");

        let required_features =
            adapter.features() & (wgpu::Features::CLEAR_TEXTURE | wgpu::Features::PIPELINE_CACHE);
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("patchbay-gui-device"),
            required_features,
            required_limits: wgpu::Limits::default(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
        }))
        .map_err(|err| {
            log_line_safe(&format!("renderer_device: request_device error: {err:?}"));
            GuiError::Device(err)
        })?;
        log_line_safe("renderer_device: device created");
        device.on_uncaptured_error(Arc::new(|error| {
            log_line_safe(&format!("renderer_device: uncaptured wgpu error: {error}"));
        }));

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
        })
    }
}

/// GPU renderer that uploads a CPU canvas and presents it through Vello.
pub struct Renderer {
    device: Arc<RendererDevice>,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    vello_renderer: VelloRenderer,
    scene: Scene,
    render_target_texture: wgpu::Texture,
    render_target_view: wgpu::TextureView,
    blitter: wgpu::util::TextureBlitter,
    canvas_texture: wgpu::Texture,
    canvas_image: ImageData,
    canvas_size: Size,
    upload_scratch: Vec<u8>,
    vector_painter: VectorScenePainter,
    vector_commands: Vec<VectorCommand>,
}

impl Renderer {
    /// Create a new renderer for the given window with a shared device.
    pub fn new_with_device(
        device: Arc<RendererDevice>,
        window: SurfaceWindow,
        size: Size,
    ) -> Result<Self, GuiError> {
        log_line_safe("renderer: new begin");
        let surface: wgpu::Surface<'static> =
            device.instance.create_surface(window).map_err(|err| {
                log_line_safe(&format!("renderer: create_surface error: {err:?}"));
                GuiError::Surface(err)
            })?;
        log_line_safe("renderer: surface created");
        let capabilities = surface.get_capabilities(&device.adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|candidate| {
                matches!(
                    candidate,
                    wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Bgra8Unorm
                )
            })
            .unwrap_or_else(|| capabilities.formats[0]);
        let present_mode = capabilities
            .present_modes
            .iter()
            .copied()
            .find(|mode| *mode == wgpu::PresentMode::Fifo)
            .unwrap_or(capabilities.present_modes[0]);
        let alpha_mode = capabilities.alpha_modes[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device.device, &config);
        log_line_safe("renderer: surface configured");

        let mut vello_renderer = VelloRenderer::new(&device.device, RendererOptions::default())
            .map_err(map_vello_init_error)?;
        let initial_canvas_size = Size {
            width: size.width.max(1),
            height: size.height.max(1),
        };
        let (render_target_texture, render_target_view) = Self::create_render_target(
            &device.device,
            Size {
                width: config.width,
                height: config.height,
            },
        );
        let blitter = wgpu::util::TextureBlitter::new(&device.device, config.format);
        let canvas_texture = Self::create_canvas_texture(&device.device, initial_canvas_size);
        let canvas_image = vello_renderer.register_texture(canvas_texture.clone());

        Ok(Self {
            device,
            surface,
            config,
            vello_renderer,
            scene: Scene::new(),
            render_target_texture,
            render_target_view,
            blitter,
            canvas_texture,
            canvas_image,
            canvas_size: initial_canvas_size,
            upload_scratch: Vec::new(),
            vector_painter: VectorScenePainter::new(),
            vector_commands: Vec::new(),
        })
    }

    /// Return true if vector text rendering is available.
    pub fn vector_text_available(&self) -> bool {
        self.vector_painter.has_text_font()
    }

    /// Replace the queued vector commands for the next render pass.
    pub fn set_vector_commands(&mut self, commands: Vec<VectorCommand>) {
        self.vector_commands = commands;
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

    /// Render the uploaded canvas texture to the current surface frame.
    pub fn render(&mut self) -> Result<(), GuiError> {
        let output = match self.surface.get_current_texture() {
            Ok(output) => output,
            Err(err) => {
                if should_reconfigure_surface(&err) {
                    self.surface.configure(&self.device.device, &self.config);
                    self.surface.get_current_texture().map_err(|err| {
                        log_line_safe(&format!(
                            "renderer: get_current_texture after reconfigure error: {err:?}"
                        ));
                        GuiError::SurfaceAcquire(err)
                    })?
                } else {
                    log_line_safe(&format!("renderer: get_current_texture error: {err:?}"));
                    return Err(GuiError::SurfaceAcquire(err));
                }
            }
        };
        let surface_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.scene.reset();
        let scale_x = self.config.width as f64 / self.canvas_size.width.max(1) as f64;
        let scale_y = self.config.height as f64 / self.canvas_size.height.max(1) as f64;
        let uniform_scale = scale_x.min(scale_y).max(f64::EPSILON);
        let fitted_width = self.canvas_size.width as f64 * uniform_scale;
        let fitted_height = self.canvas_size.height as f64 * uniform_scale;
        let offset_x = ((self.config.width as f64 - fitted_width) * 0.5).max(0.0);
        let offset_y = ((self.config.height as f64 - fitted_height) * 0.5).max(0.0);
        let scene_transform =
            Affine::translate((offset_x, offset_y)) * Affine::scale(uniform_scale);
        self.scene.draw_image(&self.canvas_image, scene_transform);
        self.vector_painter.append_to_scene(
            &mut self.scene,
            &self.vector_commands,
            scene_transform,
        );

        if let Err(err) = self.vello_renderer.render_to_texture(
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
        ) {
            log_line_safe(&format!("renderer: vello render_to_texture error: {err:?}"));
            return Err(GuiError::SurfaceAcquire(wgpu::SurfaceError::Other));
        }

        let mut encoder =
            self.device
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("patchbay-gui-present-blit"),
                });
        self.blitter.copy(
            &self.device.device,
            &mut encoder,
            &self.render_target_view,
            &surface_view,
        );
        self.device.queue.submit(Some(encoder.finish()));
        output.present();
        Ok(())
    }

    fn ensure_canvas_texture(&mut self, size: Size) {
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

    fn pad_rows_rgba(
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

fn map_vello_init_error(err: vello::Error) -> GuiError {
    log_line_safe(&format!("renderer: vello init error: {err:?}"));
    match err {
        vello::Error::NoCompatibleDevice => GuiError::AdapterNotFound,
        vello::Error::UnsupportedSurfaceFormat => GuiError::SurfaceFormat,
        _ => GuiError::SurfaceFormat,
    }
}

fn should_reconfigure_surface(err: &wgpu::SurfaceError) -> bool {
    matches!(err, wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated)
}

#[cfg(test)]
mod tests {
    use super::{Renderer, should_reconfigure_surface};

    #[test]
    fn pad_rows_rgba_zeroes_padding_bytes() {
        let width = 3u32;
        let height = 2u32;
        let bytes_per_row = width * 4;
        let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u32;
        let padded_bytes_per_row = ((bytes_per_row + alignment - 1) / alignment) * alignment;
        assert!(padded_bytes_per_row > bytes_per_row);

        let pixels = vec![1u8; (width * height * 4) as usize];
        let mut scratch = vec![9u8; 64];
        let required = (padded_bytes_per_row * height) as usize;
        Renderer::pad_rows_rgba(
            &pixels,
            width,
            height,
            padded_bytes_per_row,
            &mut scratch,
            required,
        );

        let dst_row = padded_bytes_per_row as usize;
        let src_row = bytes_per_row as usize;
        for row in 0..height as usize {
            let pad_start = row * dst_row + src_row;
            let pad_end = (row + 1) * dst_row;
            assert!(scratch[pad_start..pad_end].iter().all(|value| *value == 0));
        }
    }

    #[test]
    fn pad_rows_rgba_overwrites_old_padding() {
        let width = 5u32;
        let height = 3u32;
        let bytes_per_row = width * 4;
        let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u32;
        let padded_bytes_per_row = ((bytes_per_row + alignment - 1) / alignment) * alignment;
        assert!(padded_bytes_per_row > bytes_per_row);

        let pixels = vec![2u8; (width * height * 4) as usize];
        let mut scratch = vec![7u8; 512];
        let required = (padded_bytes_per_row * height) as usize;
        Renderer::pad_rows_rgba(
            &pixels,
            width,
            height,
            padded_bytes_per_row,
            &mut scratch,
            required,
        );

        let dst_row = padded_bytes_per_row as usize;
        let src_row = bytes_per_row as usize;
        for row in 0..height as usize {
            let pad_start = row * dst_row + src_row;
            let pad_end = (row + 1) * dst_row;
            assert!(scratch[pad_start..pad_end].iter().all(|value| *value == 0));
        }
    }

    #[test]
    fn surface_errors_trigger_reconfigure() {
        assert!(should_reconfigure_surface(&wgpu::SurfaceError::Lost));
        assert!(should_reconfigure_surface(&wgpu::SurfaceError::Outdated));
        assert!(!should_reconfigure_surface(&wgpu::SurfaceError::Timeout));
        assert!(!should_reconfigure_surface(
            &wgpu::SurfaceError::OutOfMemory
        ));
        assert!(!should_reconfigure_surface(&wgpu::SurfaceError::Other));
    }
}
