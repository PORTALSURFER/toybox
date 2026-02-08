//! Unit tests for renderer upload and surface recovery helpers.

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
