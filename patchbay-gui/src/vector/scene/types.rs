//! Core public types and top-level painter orchestration.

use crate::canvas::{Color, Point, Rect};
use vello::Scene;
use vello::kurbo::Affine;
use vello::peniko::FontData;

use super::font_loading::load_default_font;
use super::knob_rendering::draw_knob;

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
            }
        }
    }
}
