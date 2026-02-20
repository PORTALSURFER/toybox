impl Renderer {
    /// Read back the final render target texture as RGBA8 pixels.
    #[cfg(feature = "frame-capture")]
    pub(crate) fn readback_render_target_rgba8(
        &self,
    ) -> Result<crate::CapturedWindowFrame, String> {
        let size = crate::canvas::Size {
            width: self.config.width.max(1),
            height: self.config.height.max(1),
        };
        let unpadded_bytes_per_row = size.width.saturating_mul(4);
        let padded_bytes_per_row = align_bytes_per_row(unpadded_bytes_per_row);
        let total_bytes = u64::from(padded_bytes_per_row) * u64::from(size.height);
        let staging = self.device.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("patchbay-gui-readback-staging"),
            size: total_bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder =
            self.device
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("patchbay-gui-readback-encoder"),
                });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.render_target_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &staging,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(size.height),
                },
            },
            wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
        );
        self.device.queue.submit(Some(encoder.finish()));

        let slice = staging.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });

        let _ = self.device.device.poll(wgpu::PollType::wait_indefinitely());
        let map_result = rx
            .recv()
            .map_err(|err| format!("map callback channel failed: {err}"))?;
        map_result.map_err(|err| format!("buffer map failed: {err}"))?;

        let mapped = slice.get_mapped_range();
        let pixels = copy_unpadded_rows(
            mapped.as_ref(),
            size.width,
            size.height,
            padded_bytes_per_row,
            unpadded_bytes_per_row,
        )?;
        drop(mapped);
        staging.unmap();

        Ok(crate::CapturedWindowFrame {
            width: size.width,
            height: size.height,
            pixels,
        })
    }
}

/// Align row bytes to WGPU's copy alignment requirement.
#[cfg(feature = "frame-capture")]
fn align_bytes_per_row(unpadded: u32) -> u32 {
    let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u32;
    ((unpadded + alignment - 1) / alignment) * alignment
}

/// Strip row padding from mapped staging bytes into tight RGBA8 rows.
#[cfg(feature = "frame-capture")]
pub(super) fn copy_unpadded_rows(
    mapped: &[u8],
    width: u32,
    height: u32,
    padded_bytes_per_row: u32,
    unpadded_bytes_per_row: u32,
) -> Result<Vec<u8>, String> {
    let expected_len = usize::try_from(u64::from(padded_bytes_per_row) * u64::from(height))
        .map_err(|_| "invalid mapped readback size".to_string())?;
    if mapped.len() < expected_len {
        return Err(format!(
            "mapped readback is too short: got {}, expected at least {}",
            mapped.len(),
            expected_len
        ));
    }
    if padded_bytes_per_row < unpadded_bytes_per_row {
        return Err("padded bytes per row is smaller than unpadded bytes".to_string());
    }

    let output_len = usize::try_from(u64::from(width) * u64::from(height) * 4)
        .map_err(|_| "invalid output pixel size".to_string())?;
    let mut pixels = vec![0_u8; output_len];
    let padded_row = usize::try_from(padded_bytes_per_row)
        .map_err(|_| "invalid padded bytes per row".to_string())?;
    let unpadded_row = usize::try_from(unpadded_bytes_per_row)
        .map_err(|_| "invalid unpadded bytes per row".to_string())?;

    for row in 0..usize::try_from(height).map_err(|_| "invalid height".to_string())? {
        let src = row.saturating_mul(padded_row);
        let dst = row.saturating_mul(unpadded_row);
        pixels[dst..dst + unpadded_row].copy_from_slice(&mapped[src..src + unpadded_row]);
    }

    Ok(pixels)
}
