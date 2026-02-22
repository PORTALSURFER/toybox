//! Core public types and top-level painter orchestration.

use crate::canvas::{Color, Point, PointF, Rect};
use vello::Scene;
use vello::kurbo::Affine;
use vello::peniko::FontData;

use super::font_loading::load_default_font;
use super::knob_rendering::draw_knob;
use super::shapes_rendering::{
    draw_circle_fill, draw_circle_stroke, draw_line_stroke, draw_polyline_stroke, draw_rect_fill,
    draw_rect_stroke,
};

/// Vector command stream emitted by the UI layer.
#[derive(Clone, Debug)]
pub(crate) enum VectorCommand {
    /// Draw text at a top-left origin using the requested scale.
    Text {
        /// Top-left text origin in canvas-space pixels.
        origin: Point,
        /// Optional clip rectangle in canvas-space pixels.
        clip_rect: Option<Rect>,
        /// Text content.
        text: String,
        /// Text color.
        color: Color,
        /// Legacy bitmap-compatible scale.
        scale: u32,
    },
    /// Draw single-line text centered on a target x using vector glyph bounds.
    CenteredText {
        /// Left bound used for clamping within a control block.
        left_bound: i32,
        /// Maximum drawable width from `left_bound`.
        max_width: u32,
        /// Target center x position for visible glyph ink.
        target_center_x: i32,
        /// Top-left y origin in canvas-space pixels.
        origin_y: i32,
        /// Text content.
        text: String,
        /// Text color.
        color: Color,
        /// Legacy bitmap-compatible scale.
        scale: u32,
    },
    /// Draw a knob primitive with anti-aliased vector geometry.
    Knob(KnobVisual),
    /// Draw a filled rectangle.
    RectFill(RectVisual),
    /// Draw a stroked rectangle.
    RectStroke(RectStrokeVisual),
    /// Draw a stroked line segment.
    Line(LineVisual),
    /// Draw a stroked polyline path.
    Polyline(PolylineVisual),
    /// Draw a filled circle.
    CircleFill(CircleVisual),
    /// Draw a stroked circle.
    CircleStroke(CircleStrokeVisual),
}

/// Knob visual parameters emitted by [`crate::ui::Ui`].
#[derive(Clone, Copy, Debug)]
pub(crate) struct KnobVisual {
    /// Knob center in canvas-space pixels.
    pub center: Point,
    /// Filled body radius in pixels.
    pub radius: i32,
    /// Ring arc radius in pixels.
    pub arc_radius: i32,
    /// Ring arc stroke thickness in pixels.
    pub arc_thickness: f32,
    /// Arc start angle in radians.
    pub arc_start: f32,
    /// Arc end angle in radians.
    pub arc_end: f32,
    /// Current value angle in radians.
    pub value_angle: f32,
    /// Active fill color.
    pub fill: Color,
    /// Ring/outline color.
    pub outline: Color,
    /// Indicator color.
    pub indicator: Color,
}

/// Filled rectangle payload emitted by [`crate::ui::Ui`].
#[derive(Clone, Copy, Debug)]
pub(crate) struct RectVisual {
    /// Rectangle bounds in canvas-space pixels.
    pub rect: Rect,
    /// Fill color.
    pub color: Color,
}

/// Stroked rectangle payload emitted by [`crate::ui::Ui`].
#[derive(Clone, Copy, Debug)]
pub(crate) struct RectStrokeVisual {
    /// Rectangle bounds in canvas-space pixels.
    pub rect: Rect,
    /// Stroke thickness in pixels.
    pub thickness: f32,
    /// Stroke color.
    pub color: Color,
}

/// Stroked line payload emitted by [`crate::ui::Ui`].
#[derive(Clone, Copy, Debug)]
pub(crate) struct LineVisual {
    /// Start point in canvas-space pixels.
    pub start: PointF,
    /// End point in canvas-space pixels.
    pub end: PointF,
    /// Stroke thickness in pixels.
    pub thickness: f32,
    /// Stroke color.
    pub color: Color,
}

/// Stroked polyline payload emitted by [`crate::ui::Ui`].
#[derive(Clone, Debug)]
pub(crate) struct PolylineVisual {
    /// Polyline points in canvas-space pixels.
    pub points: Vec<PointF>,
    /// Stroke thickness in pixels.
    pub thickness: f32,
    /// Stroke color.
    pub color: Color,
}

/// Filled circle payload emitted by [`crate::ui::Ui`].
#[derive(Clone, Copy, Debug)]
pub(crate) struct CircleVisual {
    /// Circle center in canvas-space pixels.
    pub center: PointF,
    /// Radius in pixels.
    pub radius: f32,
    /// Fill color.
    pub color: Color,
}

/// Stroked circle payload emitted by [`crate::ui::Ui`].
#[derive(Clone, Copy, Debug)]
pub(crate) struct CircleStrokeVisual {
    /// Circle center in canvas-space pixels.
    pub center: PointF,
    /// Radius in pixels.
    pub radius: f32,
    /// Stroke thickness in pixels.
    pub thickness: f32,
    /// Stroke color.
    pub color: Color,
}

/// Loaded font payload used for vector text rendering.
#[derive(Clone, Debug)]
pub(super) struct LoadedFont {
    /// Vello font handle used by draw_glyphs.
    pub(super) data: FontData,
    /// Owned font bytes kept alive for skrifa parsing.
    pub(super) bytes: Vec<u8>,
    /// Font face index for collections.
    pub(super) index: u32,
}

/// Resolved vertical metrics for text line layout.
#[derive(Clone, Copy, Debug)]
pub(super) struct TextLineMetrics {
    /// Baseline ascent in pixels.
    pub(super) ascent: f32,
    /// Line-to-line advance in pixels.
    pub(super) line_height: f32,
}

/// Builds Vello scene primitives from UI vector commands.
#[derive(Clone, Debug)]
pub(crate) struct VectorScenePainter {
    /// Optional loaded font for vector text rendering.
    pub(super) font: Option<LoadedFont>,
    /// Guard to avoid repeatedly logging missing-font fallbacks.
    pub(super) logged_missing_font: bool,
}

impl VectorScenePainter {
    /// Create a vector painter and attempt to load a system sans-serif font.
    pub(crate) fn new() -> Self {
        Self {
            font: load_default_font(),
            logged_missing_font: false,
        }
    }

    /// Returns true when Vello text rendering is available.
    pub(crate) fn has_text_font(&self) -> bool {
        self.font.is_some()
    }

    /// Append vector commands onto a scene using the provided global transform.
    pub(crate) fn append_to_scene(
        &mut self,
        scene: &mut Scene,
        commands: &[VectorCommand],
        transform: Affine,
    ) {
        for command in commands {
            match command {
                VectorCommand::Text {
                    origin,
                    clip_rect,
                    text,
                    color,
                    scale,
                } => self.draw_text(scene, *origin, *clip_rect, text, *color, *scale, transform),
                VectorCommand::CenteredText {
                    left_bound,
                    max_width,
                    target_center_x,
                    origin_y,
                    text,
                    color,
                    scale,
                } => self.draw_centered_text(
                    scene,
                    *left_bound,
                    *max_width,
                    *target_center_x,
                    *origin_y,
                    text,
                    *color,
                    *scale,
                    transform,
                ),
                VectorCommand::Knob(knob) => draw_knob(scene, *knob, transform),
                VectorCommand::RectFill(rect) => draw_rect_fill(scene, *rect, transform),
                VectorCommand::RectStroke(rect) => draw_rect_stroke(scene, *rect, transform),
                VectorCommand::Line(line) => draw_line_stroke(scene, *line, transform),
                VectorCommand::Polyline(polyline) => {
                    draw_polyline_stroke(scene, polyline, transform)
                }
                VectorCommand::CircleFill(circle) => draw_circle_fill(scene, *circle, transform),
                VectorCommand::CircleStroke(circle) => {
                    draw_circle_stroke(scene, *circle, transform)
                }
            }
        }
    }
}
