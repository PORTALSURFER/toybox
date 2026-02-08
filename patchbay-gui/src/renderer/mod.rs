//! Vello-backed renderer that presents the CPU canvas to a window surface.

use std::sync::Arc;

use vello::peniko::ImageData;
use vello::{Renderer as VelloRenderer, Scene};

use crate::canvas::Size;
use crate::vector::scene::{VectorCommand, VectorScenePainter};

#[path = "error/handling.rs"]
mod error_handling;
#[path = "device/initialization.rs"]
mod initialization;
#[path = "lifecycle/core.rs"]
mod lifecycle;
#[cfg(test)]
#[path = "tests/unit.rs"]
mod tests;
#[path = "presentation/upload_and_present.rs"]
mod upload_and_present;

pub(crate) use error_handling::{map_vello_init_error, should_reconfigure_surface};

/// Surface-space transform used to present the CPU canvas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PresentationTransform {
    /// X-axis scale factor.
    pub scale_x: f32,
    /// Y-axis scale factor.
    pub scale_y: f32,
    /// X-axis translation in surface pixels.
    pub offset_x: f32,
    /// Y-axis translation in surface pixels.
    pub offset_y: f32,
}

impl PresentationTransform {
    /// Build a transform that stretches the canvas to the full surface.
    fn stretch(surface_width: u32, surface_height: u32, canvas_size: Size) -> Self {
        Self {
            scale_x: surface_width.max(1) as f32 / canvas_size.width.max(1) as f32,
            scale_y: surface_height.max(1) as f32 / canvas_size.height.max(1) as f32,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }
}

/// Cached GPU device resources shared across window surfaces.
#[derive(Debug)]
pub struct RendererDevice {
    /// WGPU instance used to create adapters and surfaces.
    instance: wgpu::Instance,
    /// Selected adapter for rendering.
    adapter: wgpu::Adapter,
    /// Device used for creating and running GPU work.
    device: wgpu::Device,
    /// Queue used to submit render and upload commands.
    queue: wgpu::Queue,
}

/// GPU renderer that uploads a CPU canvas and presents it through Vello.
pub struct Renderer {
    /// Shared device and queue resources.
    device: Arc<RendererDevice>,
    /// Window surface used for swapchain presentation.
    surface: wgpu::Surface<'static>,
    /// Surface configuration for resize/present operations.
    config: wgpu::SurfaceConfiguration,
    /// Vello renderer used to rasterize the scene into the target texture.
    vello_renderer: VelloRenderer,
    /// Scene graph containing the canvas image and vector overlays.
    scene: Scene,
    /// Intermediate render target texture.
    render_target_texture: wgpu::Texture,
    /// View for the intermediate render target texture.
    render_target_view: wgpu::TextureView,
    /// Utility for copying from render target into swapchain surface.
    blitter: wgpu::util::TextureBlitter,
    /// Texture that stores uploaded CPU canvas pixels.
    canvas_texture: wgpu::Texture,
    /// Vello texture registration handle for the canvas texture.
    canvas_image: ImageData,
    /// Current uploaded canvas size.
    canvas_size: Size,
    /// Reused scratch buffer for row-padded uploads.
    upload_scratch: Vec<u8>,
    /// Painter for vector overlay commands.
    vector_painter: VectorScenePainter,
    /// Commands queued for vector overlay rendering.
    vector_commands: Vec<VectorCommand>,
    /// Optional presentation transform; defaults to stretch.
    presentation_transform: Option<PresentationTransform>,
}
