//! Vello vector draw command encoding for text and knob primitives.
//!
//! The UI layer emits lightweight draw commands so widget interaction/layout can
//! stay independent from renderer details. The renderer consumes these commands
//! and appends high-quality vector primitives to the Vello scene.
#![cfg_attr(not(target_os = "windows"), allow(dead_code))]

use crate::canvas::{Color, Point};
use crate::logging::log_line_safe;
use skrifa::prelude::{FontRef, LocationRef, MetadataProvider, Size};
use std::f32::consts::TAU;
use std::path::PathBuf;
use vello::kurbo::{Affine, BezPath, Circle, Line, Point as KurboPoint, Stroke};
use vello::peniko::{Blob, Color as VelloColor, Fill, FontData};
use vello::{Glyph, Scene};

/// Vector command stream emitted by the UI layer.
#[derive(Clone, Debug)]
pub(crate) enum VectorCommand {
    /// Draw text at a top-left origin using the requested scale.
    Text {
        /// Top-left text origin in canvas-space pixels.
        origin: Point,
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
    pub arc_thickness: i32,
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

#[derive(Clone, Debug)]
struct LoadedFont {
    data: FontData,
    bytes: Vec<u8>,
    index: u32,
}

/// Builds Vello scene primitives from UI vector commands.
#[derive(Clone, Debug)]
pub(crate) struct VectorScenePainter {
    font: Option<LoadedFont>,
    logged_missing_font: bool,
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
                    text,
                    color,
                    scale,
                } => self.draw_text(scene, *origin, text, *color, *scale, transform),
                VectorCommand::Knob(knob) => draw_knob(scene, *knob, transform),
            }
        }
    }

    fn draw_text(
        &mut self,
        scene: &mut Scene,
        origin: Point,
        text: &str,
        color: Color,
        scale: u32,
        transform: Affine,
    ) {
        let Some(font) = &self.font else {
            if !self.logged_missing_font {
                log_line_safe(
                    "vector_scene: no system font found; falling back to bitmap text rendering",
                );
                self.logged_missing_font = true;
            }
            return;
        };

        let Ok(font_ref) = FontRef::from_index(font.bytes.as_slice(), font.index) else {
            if !self.logged_missing_font {
                log_line_safe(
                    "vector_scene: failed to parse loaded font; falling back to bitmap text rendering",
                );
                self.logged_missing_font = true;
            }
            return;
        };

        let font_size = (8u32.saturating_mul(scale.max(1))) as f32;
        let metrics = font_ref.metrics(Size::new(font_size), LocationRef::default());
        let glyph_metrics = font_ref.glyph_metrics(Size::new(font_size), LocationRef::default());
        let charmap = font_ref.charmap();
        let ascent = metrics.ascent.max(font_size * 0.7);
        let line_height = (metrics.ascent - metrics.descent + metrics.leading).max(font_size);

        let mut glyphs = Vec::new();
        let mut cursor_x = origin.x as f32;
        let mut baseline_y = origin.y as f32 + ascent;
        let fallback = charmap.map('?');

        for ch in text.chars() {
            if ch == '\n' {
                cursor_x = origin.x as f32;
                baseline_y += line_height;
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

            let advance = glyph_metrics
                .advance_width(gid)
                .unwrap_or_else(|| font_size * 0.5);
            cursor_x += advance;
        }

        if glyphs.is_empty() {
            return;
        }

        scene
            .draw_glyphs(&font.data)
            .transform(transform)
            .font_size(font_size)
            .brush(color_to_vello(color))
            .draw(Fill::NonZero, glyphs.into_iter());
    }
}

fn draw_knob(scene: &mut Scene, knob: KnobVisual, transform: Affine) {
    let radius = knob.radius.max(1) as f64;
    let center = KurboPoint::new(knob.center.x as f64, knob.center.y as f64);
    let body = Circle::new(center, radius);
    let outline_stroke = Stroke::new(2.0);
    let ring_stroke = Stroke::new(knob.arc_thickness.max(1) as f64);

    scene.fill(
        Fill::NonZero,
        transform,
        color_to_vello(knob.fill),
        None,
        &body,
    );
    scene.stroke(
        &outline_stroke,
        transform,
        color_to_vello(knob.outline),
        None,
        &body,
    );

    let ring = arc_path(
        knob.center,
        knob.arc_radius.max(1) as f32,
        knob.arc_start,
        knob.arc_end,
    );
    scene.stroke(
        &ring_stroke,
        transform,
        color_to_vello(knob.outline),
        None,
        &ring,
    );

    let active = arc_path(
        knob.center,
        knob.arc_radius.max(1) as f32,
        knob.value_angle,
        knob.arc_end,
    );
    scene.stroke(
        &ring_stroke,
        transform,
        color_to_vello(knob.indicator),
        None,
        &active,
    );

    let indicator_point = indicator_point(knob.center, knob.radius, knob.value_angle);
    let indicator_line = Line::new(
        KurboPoint::new(knob.center.x as f64, knob.center.y as f64),
        KurboPoint::new(indicator_point.x as f64, indicator_point.y as f64),
    );
    scene.stroke(
        &Stroke::new(2.0),
        transform,
        color_to_vello(knob.indicator),
        None,
        &indicator_line,
    );
}

fn arc_path(center: Point, radius: f32, start_angle: f32, end_angle: f32) -> BezPath {
    let mut start = normalize_angle(start_angle);
    let mut end = normalize_angle(end_angle);
    if (start - end).abs() < f32::EPSILON {
        return BezPath::new();
    }
    if end < start {
        end += TAU;
    }
    if start > end {
        std::mem::swap(&mut start, &mut end);
    }
    let span = (end - start).abs();
    let segments = ((span * radius.max(1.0) * 0.2).ceil() as usize).clamp(8, 96);
    let step = span / segments as f32;

    let mut path = BezPath::new();
    for idx in 0..=segments {
        let angle = start + step * idx as f32;
        let x = center.x as f32 + radius * angle.cos();
        let y = center.y as f32 - radius * angle.sin();
        let point = KurboPoint::new(x as f64, y as f64);
        if idx == 0 {
            path.move_to(point);
        } else {
            path.line_to(point);
        }
    }
    path
}

fn indicator_point(center: Point, radius: i32, angle: f32) -> Point {
    Point {
        x: center.x + (radius as f32 * angle.cos()) as i32,
        y: center.y - (radius as f32 * angle.sin()) as i32,
    }
}

fn normalize_angle(angle: f32) -> f32 {
    let mut normalized = angle % TAU;
    if normalized < 0.0 {
        normalized += TAU;
    }
    normalized
}

fn color_to_vello(color: Color) -> VelloColor {
    VelloColor::from_rgba8(color.r, color.g, color.b, color.a)
}

fn load_default_font() -> Option<LoadedFont> {
    let mut candidates = Vec::new();
    if let Some(path) = std::env::var_os("PATCHBAY_GUI_FONT_PATH")
        .map(PathBuf::from)
        .filter(|path| path.exists())
    {
        candidates.push(path);
    }

    #[cfg(target_os = "windows")]
    {
        candidates.extend([
            PathBuf::from(r"C:\Windows\Fonts\segoeui.ttf"),
            PathBuf::from(r"C:\Windows\Fonts\segoeuii.ttf"),
            PathBuf::from(r"C:\Windows\Fonts\arial.ttf"),
            PathBuf::from(r"C:\Windows\Fonts\tahoma.ttf"),
        ]);
    }
    #[cfg(target_os = "macos")]
    {
        candidates.extend([
            PathBuf::from("/System/Library/Fonts/SFNS.ttf"),
            PathBuf::from("/System/Library/Fonts/Supplemental/Arial.ttf"),
            PathBuf::from("/System/Library/Fonts/Supplemental/Helvetica.ttc"),
        ]);
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        candidates.extend([
            PathBuf::from("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"),
            PathBuf::from("/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf"),
            PathBuf::from("/usr/share/fonts/TTF/DejaVuSans.ttf"),
        ]);
    }

    for candidate in candidates {
        let Ok(bytes) = std::fs::read(&candidate) else {
            continue;
        };
        if FontRef::from_index(bytes.as_slice(), 0).is_err() {
            continue;
        }
        log_line_safe(&format!(
            "vector_scene: loaded text font from {}",
            candidate.display()
        ));
        let data = FontData::new(Blob::from(bytes.clone()), 0);
        return Some(LoadedFont {
            data,
            bytes,
            index: 0,
        });
    }
    None
}
