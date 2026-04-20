//! Headless screenshot rendering for declarative UI specs.
//!
//! This module renders one frame directly from a spec-building callback into a CPU
//! RGBA buffer. It intentionally mirrors the two-pass transform flow used by the
//! live window renderer so pointer mapping and root sizing stay consistent.

use crate::Canvas;
use crate::canvas::{Point, Size};
use crate::declarative::{
    LayoutEngineState, RenderResult, RootTransform, SceneRenderFeatures, UiSpec,
    execute_scene_frame, plan_scene_frame,
};
use crate::host::InputState;
use crate::ui::{Layout, Theme, UiState};

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
/// The callback receives input snapshot with the requested logical size and is
/// planned through the same two-pass scene contract as the live renderer before
/// backend-specific headless execution happens.
///
/// Headless rendering still disables vector text and vector shapes temporarily.
/// That fallback is expected to disappear once the shared scene contract is
/// consumed by the Radiant-backed headless path tracked by `OPT-90`/`OPT-91`.
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
    let frame = plan_scene_frame(&input, &mut build_spec);
    let layout_size = frame.plan.layout_size;
    let mut canvas = Canvas::new(layout_size.width, layout_size.height);
    let mut layout = Layout::default();
    let mut ui_state = UiState::default();
    let mut engine = LayoutEngineState::default();
    let theme = Theme::default();
    let executed = execute_scene_frame(
        &frame,
        &mut canvas,
        &mut ui_state,
        &mut layout,
        &theme,
        &mut engine,
        SceneRenderFeatures {
            vector_text: false,
            vector_shapes: false,
        },
    )
    .map_err(|err| err.to_string())?;
    let pixels = if layout_size == input.window_size {
        canvas.pixels().to_vec()
    } else {
        remap_canvas_to_surface(
            canvas.pixels(),
            layout_size,
            frame.surface_input.window_size,
            &frame.plan.transform,
        )
    };

    Ok(RenderedFrame {
        width: frame.surface_input.window_size.width,
        height: frame.surface_input.window_size.height,
        pixels,
        render_result: executed.render_result,
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
