//! Headless screenshot rendering for declarative UI specs.
//!
//! This module renders one frame directly from a spec-building callback into a CPU
//! RGBA buffer. It intentionally mirrors the two-pass transform flow used by the
//! live window renderer so pointer mapping and root sizing stay consistent.

use crate::Canvas;
use crate::canvas::{Point, Size};
use crate::declarative::{RenderResult, RootTransform, UiSpec, render_checked};
use crate::host::InputState;
use crate::ui::{Layout, Theme, Ui, UiState};

/// Pixel data and metadata for one rendered declarative frame.
#[derive(Debug)]
pub struct RenderedFrame {
    /// Final rendered width in pixels.
    pub width: u32,
    /// Final rendered height in pixels.
    pub height: u32,
    /// RGBA pixels in row-major, top-left origin order.
    pub pixels: Vec<u8>,
    /// Declarative render result metadata.
    pub render_result: RenderResult,
}

/// Copy rendered canvas pixels into a surface-sized buffer using the resolved root transform.
///
/// Pixels outside the transformed content rectangle stay transparent so the helper
/// can be used to generate headless screenshots that match on-screen composition.
fn remap_canvas_to_surface(
    source: &[u8],
    source_size: Size,
    surface_size: Size,
    transform: &RootTransform,
) -> Vec<u8> {
    let source_size = Size {
        width: source_size.width.max(1),
        height: source_size.height.max(1),
    };
    let surface_size = Size {
        width: surface_size.width.max(1),
        height: surface_size.height.max(1),
    };
    let source_stride = source_size.width as usize * 4;
    let output_stride = surface_size.width as usize * 4;
    let mut output = vec![0u8; output_stride * surface_size.height as usize];

    for y in 0..surface_size.height as i32 {
        for x in 0..surface_size.width as i32 {
            let surface_point = Point { x, y };
            let content = transform.content_rect_surface;
            if surface_point.x < content.origin.x
                || surface_point.y < content.origin.y
                || surface_point.x >= content.origin.x + content.size.width as i32
                || surface_point.y >= content.origin.y + content.size.height as i32
            {
                continue;
            }
            let mapped = transform.surface_to_design_clamped(surface_point);
            let source_x = mapped.x as usize;
            let source_y = mapped.y as usize;
            let source_index = (source_y * source_stride) + (source_x * 4);
            let output_index = (y as usize) * output_stride + (x as usize * 4);
            output[output_index..output_index + 4]
                .copy_from_slice(&source[source_index..source_index + 4]);
        }
    }
    output
}

/// Render a single UI frame into an in-memory PNG-friendly pixel buffer.
///
/// The callback receives input snapshot with the requested logical size and can
/// build a [`UiSpec`]. The spec is evaluated twice like the live loop:
///
/// - first pass: collect design/surface transform for pointer remapping
/// - second pass: render using transformed pointer coordinates
///
/// This is currently intended for test and CI screenshot generation only.
pub fn render_spec_to_frame<Build>(
    size: Size,
    mut build_spec: Build,
) -> Result<RenderedFrame, String>
where
    Build: FnMut(&InputState) -> UiSpec,
{
    let window_size = Size {
        width: size.width.max(1),
        height: size.height.max(1),
    };
    let input = InputState {
        window_size,
        ..InputState::default()
    };
    let initial_spec = build_spec(&input);
    let initial_plan = crate::declarative::plan_root_render(&initial_spec, input.window_size);

    let mut mapped_input = input.clone();
    mapped_input.pointer_pos = initial_plan
        .transform
        .surface_to_design_clamped(mapped_input.pointer_pos);

    let spec = build_spec(&mapped_input);
    let plan = crate::declarative::plan_root_render(&spec, mapped_input.window_size);
    mapped_input.pointer_pos = plan
        .transform
        .surface_to_design_clamped(mapped_input.pointer_pos);

    let layout_size = plan.layout_size;
    let mut canvas = Canvas::new(layout_size.width, layout_size.height);
    let mut layout = Layout::default();
    let mut ui_state = UiState::default();
    let theme = Theme::default();
    let mut ui = Ui::new(
        &mut canvas,
        &mapped_input,
        &mut ui_state,
        &mut layout,
        &theme,
    );
    ui.set_vector_text_enabled(false);
    ui.set_vector_shapes_enabled(false);

    let render_result = render_checked(&spec, &mut ui, crate::canvas::Point { x: 0, y: 0 })
        .map_err(|err| err.to_string())?;
    let pixels = if layout_size == input.window_size {
        canvas.pixels().to_vec()
    } else {
        remap_canvas_to_surface(
            canvas.pixels(),
            layout_size,
            input.window_size,
            &plan.transform,
        )
    };

    Ok(RenderedFrame {
        width: input.window_size.width,
        height: input.window_size.height,
        pixels,
        render_result,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::Rect;

    #[test]
    fn remap_canvas_to_surface_drops_out_of_bounds_source_pixels() {
        let source = vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ];
        let transform = RootTransform {
            scale_x: 1.0,
            scale_y: 1.0,
            offset_x: 1.0,
            offset_y: 1.0,
            content_rect_design: Rect {
                origin: Point { x: 0, y: 0 },
                size: Size {
                    width: 2,
                    height: 2,
                },
            },
            content_rect_surface: Rect {
                origin: Point { x: 1, y: 1 },
                size: Size {
                    width: 2,
                    height: 2,
                },
            },
        };

        let output = remap_canvas_to_surface(
            &source,
            Size {
                width: 2,
                height: 2,
            },
            Size {
                width: 4,
                height: 4,
            },
            &transform,
        );

        assert_eq!(output.len(), 4 * 4 * 4);
        assert_eq!(
            &output[0..16],
            &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(
            &output[16..32],
            &[0, 0, 0, 0, 255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 0, 0]
        );
        assert_eq!(
            &output[32..48],
            &[0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0]
        );
        assert_eq!(
            &output[48..64],
            &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    fn remap_canvas_to_surface_keeps_full_content_width_for_integral_scale() {
        let source = vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ];
        let transform = RootTransform {
            scale_x: 2.0,
            scale_y: 2.0,
            offset_x: 0.0,
            offset_y: 0.0,
            content_rect_design: Rect {
                origin: Point { x: 0, y: 0 },
                size: Size {
                    width: 2,
                    height: 2,
                },
            },
            content_rect_surface: Rect {
                origin: Point { x: 0, y: 0 },
                size: Size {
                    width: 4,
                    height: 4,
                },
            },
        };

        let output = remap_canvas_to_surface(
            &source,
            Size {
                width: 2,
                height: 2,
            },
            Size {
                width: 4,
                height: 4,
            },
            &transform,
        );

        assert_eq!(output.len(), 4 * 4 * 4);
        assert_eq!(
            &output[0..16],
            &[
                255, 0, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255
            ]
        );
        assert_eq!(
            &output[16..32],
            &[
                0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255
            ]
        );
        assert_eq!(&output[32..48], &output[16..32]);
        assert_eq!(&output[48..64], &output[16..32]);
    }
}
