//! Vector text rendering helpers for glyph shaping and draw submission.

use skrifa::prelude::{FontRef, LocationRef, MetadataProvider, Size};
use vello::kurbo::Affine;
use vello::peniko::{Fill, FontData};
use vello::{Glyph, Scene};

use crate::canvas::{Color, Point};
use crate::logging::log_line_safe;

use super::color_and_angle_helpers::color_to_vello;
use super::types::{TextLineMetrics, VectorScenePainter};

impl VectorScenePainter {
    /// Draw one vector text run into the scene using the loaded font, if any.
    pub(super) fn draw_text(
        &mut self,
        scene: &mut Scene,
        origin: Point,
        text: &str,
        color: Color,
        scale: u32,
        transform: Affine,
    ) {
        if self.font.is_none() {
            self.log_font_fallback_once(
                "vector_scene: no system font found; falling back to bitmap text rendering",
            );
            return;
        }
        let Some(font_ref) = self.resolve_font_ref() else {
            self.log_font_fallback_once(
                "vector_scene: failed to parse loaded font; falling back to bitmap text rendering",
            );
            return;
        };

        let font_size = (8u32.saturating_mul(scale.max(1))) as f32;
        let metrics = Self::resolve_text_line_metrics(&font_ref, font_size);
        let glyphs = Self::build_text_glyphs(&font_ref, text, origin, font_size, metrics);
        if glyphs.is_empty() {
            return;
        }

        let font = self.font.as_ref().expect("font was checked above");
        Self::draw_glyph_run(scene, &font.data, transform, color, font_size, glyphs);
    }

    /// Log a vector text fallback message once per painter instance.
    fn log_font_fallback_once(&mut self, message: &str) {
        if self.logged_missing_font {
            return;
        }
        log_line_safe(message);
        self.logged_missing_font = true;
    }

    /// Resolve and parse the currently loaded font for text rendering.
    fn resolve_font_ref(&self) -> Option<FontRef<'_>> {
        let font = self.font.as_ref()?;
        FontRef::from_index(font.bytes.as_slice(), font.index).ok()
    }

    /// Resolve ascent and line height for text layout at the given size.
    fn resolve_text_line_metrics(font_ref: &FontRef<'_>, font_size: f32) -> TextLineMetrics {
        let metrics = font_ref.metrics(Size::new(font_size), LocationRef::default());
        TextLineMetrics {
            ascent: metrics.ascent.max(font_size * 0.7),
            line_height: (metrics.ascent - metrics.descent + metrics.leading).max(font_size),
        }
    }

    /// Build positioned glyphs for one text run.
    fn build_text_glyphs(
        font_ref: &FontRef<'_>,
        text: &str,
        origin: Point,
        font_size: f32,
        metrics: TextLineMetrics,
    ) -> Vec<Glyph> {
        let glyph_metrics = font_ref.glyph_metrics(Size::new(font_size), LocationRef::default());
        let charmap = font_ref.charmap();
        let fallback = charmap.map('?');

        let mut glyphs = Vec::new();
        let mut cursor_x = origin.x as f32;
        let mut baseline_y = origin.y as f32 + metrics.ascent;
        for ch in text.chars() {
            if ch == '\n' {
                cursor_x = origin.x as f32;
                baseline_y += metrics.line_height;
                continue;
            }
            let Some(gid) = charmap.map(ch).or(fallback) else {
                continue;
            };
            glyphs.push(Glyph {
                id: gid.to_u32(),
                x: cursor_x,
                y: baseline_y,
            });
            cursor_x += glyph_metrics.advance_width(gid).unwrap_or(font_size * 0.5);
        }
        glyphs
    }

    /// Draw a positioned glyph run into the scene.
    fn draw_glyph_run(
        scene: &mut Scene,
        font_data: &FontData,
        transform: Affine,
        color: Color,
        font_size: f32,
        glyphs: Vec<Glyph>,
    ) {
        scene
            .draw_glyphs(font_data)
            .transform(transform)
            .font_size(font_size)
            .brush(color_to_vello(color))
            .draw(Fill::NonZero, glyphs.into_iter());
    }
}
