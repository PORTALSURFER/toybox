//! Wgpu renderer that presents the CPU canvas to a window surface.

use crate::canvas::Size;
use crate::host::GuiError;
use crate::logging::log_line_safe;
use crate::win32::SurfaceWindow;
use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wgpu::util::DeviceExt;

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2],
}

const VERTICES: [Vertex; 3] = [
    Vertex {
        pos: [-1.0, -3.0],
        uv: [0.0, 2.0],
    },
    Vertex {
        pos: [3.0, 1.0],
        uv: [2.0, 0.0],
    },
    Vertex {
        pos: [-1.0, 1.0],
        uv: [0.0, 0.0],
    },
];

/// Cached GPU device resources shared across window surfaces.
#[derive(Debug)]
pub struct RendererDevice {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    shader: wgpu::ShaderModule,
    vertex_buffer: wgpu::Buffer,
    pipelines: Mutex<HashMap<wgpu::TextureFormat, wgpu::RenderPipeline>>,
}

impl RendererDevice {
    /// Create a new device and queue without binding to a specific surface.
    pub fn new() -> Result<Self, GuiError> {
        log_line_safe("renderer_device: create begin");
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
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

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("patchbay-gui-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            },
        ))
        .map_err(|err| {
            log_line_safe(&format!("renderer_device: request_device error: {err:?}"));
            GuiError::Device(err)
        })?;
        log_line_safe("renderer_device: device created");

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("patchbay-gui-texture-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("patchbay-gui-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("patchbay-gui-pipeline-layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            immediate_size: 0,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("patchbay-gui-vertex-buffer"),
            contents: bytemuck::cast_slice(&VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            texture_bind_group_layout,
            pipeline_layout,
            shader,
            vertex_buffer,
            pipelines: Mutex::new(HashMap::new()),
        })
    }

    fn pipeline_for(&self, format: wgpu::TextureFormat) -> Result<wgpu::RenderPipeline, GuiError> {
        let mut cache = self
            .pipelines
            .lock()
            .map_err(|_| GuiError::DeviceCachePoison)?;
        if let Some(pipeline) = cache.get(&format) {
            return Ok(pipeline.clone());
        }
        let pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("patchbay-gui-pipeline"),
            layout: Some(&self.pipeline_layout),
            vertex: wgpu::VertexState {
                module: &self.shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &self.shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        cache.insert(format, pipeline.clone());
        Ok(pipeline)
    }
}

/// GPU renderer that uploads a CPU canvas into a surface texture.
pub struct Renderer {
    device: Arc<RendererDevice>,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    upload_scratch: Vec<u8>,
}

impl Renderer {
    /// Create a new renderer for the given window with a shared device.
    pub fn new_with_device(
        device: Arc<RendererDevice>,
        window: SurfaceWindow,
        size: Size,
    ) -> Result<Self, GuiError> {
        log_line_safe("renderer: new begin");
        let surface: wgpu::Surface<'static> = device.instance.create_surface(window).map_err(|err| {
            log_line_safe(&format!("renderer: create_surface error: {err:?}"));
            GuiError::Surface(err)
        })?;
        log_line_safe("renderer: surface created");
        let capabilities = surface.get_capabilities(&device.adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|format| format.is_srgb())
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

        let (texture, texture_view, sampler) = Self::create_texture(&device.device, size);
        let bind_group = device.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("patchbay-gui-texture-bind-group"),
            layout: &device.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        let pipeline = device.pipeline_for(format)?;

        Ok(Self {
            device,
            surface,
            config,
            texture,
            texture_view,
            sampler,
            bind_group,
            pipeline,
            upload_scratch: Vec::new(),
        })
    }

    /// Create a new renderer with a freshly created device.
    pub fn new(window: SurfaceWindow, size: Size) -> Result<Self, GuiError> {
        let device = Arc::new(RendererDevice::new()?);
        Self::new_with_device(device, window, size)
    }

    /// Resize the surface and backing texture.
    pub fn resize(&mut self, size: Size) {
        self.config.width = size.width.max(1);
        self.config.height = size.height.max(1);
        self.surface.configure(&self.device.device, &self.config);
        let (texture, texture_view, sampler) = Self::create_texture(&self.device.device, size);
        self.texture = texture;
        self.texture_view = texture_view;
        self.sampler = sampler;
        self.bind_group = self.device.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("patchbay-gui-texture-bind-group"),
            layout: &self.device.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
    }

    /// Upload the latest canvas pixels to the GPU texture.
    pub fn upload(&mut self, size: Size, pixels: &[u8]) {
        let bytes_per_pixel = 4u32;
        let bytes_per_row = bytes_per_pixel * size.width;
        let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u32;
        let padded_bytes_per_row = ((bytes_per_row + alignment - 1) / alignment) * alignment;

        if padded_bytes_per_row == bytes_per_row {
            self.write_texture(size, pixels, bytes_per_row);
            return;
        }

        let required = (padded_bytes_per_row * size.height) as usize;
        self.upload_scratch.resize(required, 0);
        let padded = &mut self.upload_scratch;
        let src_row = bytes_per_row as usize;
        let dst_row = padded_bytes_per_row as usize;
        for row in 0..size.height as usize {
            let src_offset = row * src_row;
            let dst_offset = row * dst_row;
            padded[dst_offset..dst_offset + src_row]
                .copy_from_slice(&pixels[src_offset..src_offset + src_row]);
        }

        self.write_texture(size, padded, padded_bytes_per_row);
    }

    /// Render the canvas to the surface.
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
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("patchbay-gui-encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("patchbay-gui-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_vertex_buffer(0, self.device.vertex_buffer.slice(..));
            pass.draw(0..3, 0..1);
        }

        self.device.queue.submit(Some(encoder.finish()));
        output.present();
        Ok(())
    }

    fn create_texture(
        device: &wgpu::Device,
        size: Size,
    ) -> (wgpu::Texture, wgpu::TextureView, wgpu::Sampler) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("patchbay-gui-canvas-texture"),
            size: wgpu::Extent3d {
                width: size.width.max(1),
                height: size.height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("patchbay-gui-sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        (texture, texture_view, sampler)
    }

    fn write_texture(&self, size: Size, pixels: &[u8], bytes_per_row: u32) {
        self.device.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
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
}

fn should_reconfigure_surface(err: &wgpu::SurfaceError) -> bool {
    matches!(err, wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated)
}

#[cfg(test)]
mod tests {
    use super::should_reconfigure_surface;

    #[test]
    fn surface_errors_trigger_reconfigure() {
        assert!(should_reconfigure_surface(&wgpu::SurfaceError::Lost));
        assert!(should_reconfigure_surface(&wgpu::SurfaceError::Outdated));
        assert!(!should_reconfigure_surface(&wgpu::SurfaceError::Timeout));
        assert!(!should_reconfigure_surface(&wgpu::SurfaceError::OutOfMemory));
    }
}
