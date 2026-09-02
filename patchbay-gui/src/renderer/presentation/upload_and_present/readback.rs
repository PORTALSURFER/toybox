#[cfg(feature = "frame-capture")]
use std::time::{Duration, Instant};

#[cfg(feature = "frame-capture")]
const READBACK_TIMEOUT: Duration = Duration::from_secs(5);

#[cfg(feature = "frame-capture")]
const READBACK_POLL_INTERVAL: Duration = Duration::from_millis(1);

impl Renderer {
    /// Read back the final render target texture as RGBA8 pixels.
    #[cfg(feature = "frame-capture")]
    pub(crate) fn readback_render_target_rgba8(
        &self,
    ) -> Result<crate::CapturedWindowFrame, String> {
        eprintln!("[frame-capture-probe] readback begin");
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
        eprintln!("[frame-capture-probe] pre-readback wait begin");
        self.device.instance.poll_all(true);
        eprintln!("[frame-capture-probe] pre-readback wait returned");
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
        eprintln!("[frame-capture-probe] readback submit");
        let (tx, rx) = std::sync::mpsc::channel();
        encoder.map_buffer_on_submit(
            &staging,
            wgpu::MapMode::Read,
            ..,
            move |result| {
            eprintln!("[frame-capture-probe] map callback result={result:?}");
            let _ = tx.send(result);
            },
        );
        eprintln!("[frame-capture-probe] map scheduled");
        self.device.queue.submit(Some(encoder.finish()));
        eprintln!("[frame-capture-probe] readback submitted");

        let slice = staging.slice(..);

        let deadline = Instant::now() + READBACK_TIMEOUT;
        loop {
            if Instant::now() >= deadline {
                return Err(format!(
                    "buffer map timed out after {READBACK_TIMEOUT:?}"
                ));
            }

            eprintln!("[frame-capture-probe] poll begin");
            self.device.instance.poll_all(false);
            eprintln!("[frame-capture-probe] poll returned");

            if Instant::now() >= deadline {
                return Err(format!(
                    "buffer map timed out after {READBACK_TIMEOUT:?}"
                ));
            }

            match rx.try_recv() {
                Ok(map_result) => {
                    map_result.map_err(|err| format!("buffer map failed: {err}"))?;
                    break;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    return Err("map callback channel disconnected".to_string());
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
            }

            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(format!(
                    "buffer map timed out after {READBACK_TIMEOUT:?}"
                ));
            }
            std::thread::sleep(remaining.min(READBACK_POLL_INTERVAL));
        }
        eprintln!("[frame-capture-probe] map completed");

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
    let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    unpadded.div_ceil(alignment) * alignment
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
