//! Canvas upload and presentation pass implementation.

use vello::kurbo::Affine;
use vello::peniko::Color as VelloColor;
use vello::{AaConfig, RenderParams};

use crate::canvas::Size;
use crate::host::GuiError;
use crate::logging::log_line_safe;

use super::{PresentationTransform, Renderer, should_reconfigure_surface};

impl Renderer {
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
        let output = self.acquire_surface_texture()?;
        let surface_view = Self::surface_view(&output);
        self.scene.reset();
        let scene_transform = self.resolve_scene_transform();
        self.scene.draw_image(&self.canvas_image, scene_transform);
        self.vector_painter.append_to_scene(
            &mut self.scene,
            &self.vector_commands,
            scene_transform,
        );

        self.render_scene_to_target()?;
        self.present_output(&surface_view, output);
        Ok(())
    }

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
                GuiError::SurfaceAcquire(wgpu::SurfaceError::Other)
            })
    }

    /// Blit the offscreen target to the surface and present the frame.
    fn present_output(&self, surface_view: &wgpu::TextureView, output: wgpu::SurfaceTexture) {
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
            surface_view,
        );
        self.device.queue.submit(Some(encoder.finish()));
        output.present();
    }

    /// Write pixels into the canvas texture using the given row stride.
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

    /// Pad tightly packed RGBA rows to WGPU's row alignment requirement.
    pub(super) fn pad_rows_rgba(
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
