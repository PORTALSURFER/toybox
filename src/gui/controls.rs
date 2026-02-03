//! Common interactive controls for egui-based plugin UIs.

use std::ops::RangeInclusive;

use egui_baseview::egui;
use egui_baseview::egui::{Align2, Color32, FontId, Pos2, Rect, Response, Sense, Shape, Stroke, Vec2};

/// Shared configuration for knob rendering and interaction.
#[derive(Debug, Clone)]
pub struct KnobStyle {
    /// Diameter of the knob in points.
    pub size: f32,
    /// Total width reserved for the knob cell.
    pub cell_width: f32,
    /// Total height reserved for the knob cell.
    pub cell_height: f32,
    /// Pointer-drag sensitivity in points per full-range sweep.
    pub drag_sensitivity: f32,
    /// Stroke width for the knob arc.
    pub arc_stroke: f32,
    /// Radius for the indicator line.
    pub indicator_radius: f32,
    /// Accent color for the active arc and indicator.
    pub accent: Color32,
}

impl Default for KnobStyle {
    fn default() -> Self {
        Self {
            size: 56.0,
            cell_width: 90.0,
            cell_height: 90.0,
            drag_sensitivity: 240.0,
            arc_stroke: 3.0,
            indicator_radius: 20.0,
            accent: Color32::from_rgb(90, 200, 220),
        }
    }
}

/// Draw and interact with a twist-style knob control.
///
/// The knob supports click-and-drag, click-to-set, and double-click to reset.
pub fn knob(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    range: RangeInclusive<f32>,
    default_value: f32,
    value_text: &str,
    style: &KnobStyle,
) -> Response {
    let range_min = *range.start();
    let range_max = *range.end();
    let span = (range_max - range_min).max(f32::EPSILON);
    let normalized = normalize_value(*value, range_min, range_max);

    let mut response = ui
        .allocate_ui_with_layout(
            Vec2::new(style.cell_width, style.cell_height),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.small(label);
                let (rect, _) = ui.allocate_exact_size(
                    Vec2::new(style.size, style.size),
                    egui::Sense::hover(),
                );
                let knob_id = ui.id().with(label);
                let response = ui.interact(rect, knob_id, Sense::click_and_drag());
                draw_knob(ui, rect, normalized, response.hovered(), response.dragged(), style);
                ui.small(value_text);
                response
            },
        )
        .inner;

    let pointer_pos = ui.input(|input| input.pointer.interact_pos());
    let primary_down = ui.input(|input| input.pointer.primary_down());
    let pointer_delta = ui.input(|input| input.pointer.delta().y);
    let active_id_key = ui.id().with("active_knob");

    let mut changed = false;
    if response.drag_started() || response.clicked() {
        let drag_start_id = response.id.with("drag_start");
        if let Some(pos) = pointer_pos {
            ui.memory_mut(|mem| mem.data.insert_temp(drag_start_id, (*value, pos.y)));
        }
        ui.memory_mut(|mem| mem.data.insert_temp(active_id_key, response.id));
    }
    if response.double_clicked() {
        let next = default_value.clamp(range_min, range_max);
        if (next - *value).abs() > f32::EPSILON {
            *value = next;
            changed = true;
        }
    }

    let active_id = ui.memory(|mem| mem.data.get_temp::<egui::Id>(active_id_key));
    if let Some(active_id) = active_id {
        if primary_down && active_id == response.id {
            let speed = if ui.input(|input| input.modifiers.shift) {
                0.2
            } else {
                1.0
            };
            let delta = -(pointer_delta / style.drag_sensitivity) * span * speed;
            let next = (*value + delta).clamp(range_min, range_max);
            if (next - *value).abs() > f32::EPSILON {
                *value = next;
                changed = true;
            }
        }
    }
    if !primary_down {
        ui.memory_mut(|mem| mem.data.remove::<egui::Id>(active_id_key));
    } else if response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            let center = response.rect.center();
            let vec = pos - center;
            let angle = vec.y.atan2(vec.x);
            if let Some(t) = angle_to_normalized(angle) {
                let next = (range_min + span * t).clamp(range_min, range_max);
                if (next - *value).abs() > f32::EPSILON {
                    *value = next;
                    changed = true;
                }
            }
        }
    }

    if changed {
        response.mark_changed();
    }
    response
}

/// Style configuration for the XY pad control.
#[derive(Debug, Clone)]
pub struct XYPadStyle {
    /// Size of the pad in points.
    pub size: Vec2,
    /// Stroke used for the pad outline.
    pub outline: Stroke,
    /// Stroke used for the crosshair.
    pub crosshair: Stroke,
    /// Radius of the indicator dot.
    pub dot_radius: f32,
    /// Background color for the pad.
    pub background: Color32,
    /// Accent color for the active indicator.
    pub accent: Color32,
}

impl Default for XYPadStyle {
    fn default() -> Self {
        Self {
            size: Vec2::new(160.0, 120.0),
            outline: Stroke::new(1.0, Color32::from_gray(50)),
            crosshair: Stroke::new(1.0, Color32::from_gray(80)),
            dot_radius: 4.0,
            background: Color32::from_gray(24),
            accent: Color32::from_rgb(90, 200, 220),
        }
    }
}

/// Draw and interact with an XY pad control.
///
/// The pad maps pointer position to the provided X/Y ranges and updates the
/// bound values when dragged or clicked.
pub fn xy_pad(
    ui: &mut egui::Ui,
    label: &str,
    x: &mut f32,
    y: &mut f32,
    x_range: RangeInclusive<f32>,
    y_range: RangeInclusive<f32>,
    value_text: &str,
    style: &XYPadStyle,
) -> Response {
    let x_min = *x_range.start();
    let x_max = *x_range.end();
    let y_min = *y_range.start();
    let y_max = *y_range.end();

    let mut response = ui
        .allocate_ui_with_layout(
            Vec2::new(style.size.x, style.size.y + 32.0),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.small(label);
                let (rect, _) = ui.allocate_exact_size(style.size, Sense::click_and_drag());
                let response = ui.interact(rect, ui.id().with(label), Sense::click_and_drag());
                draw_xy_pad(ui, rect, *x, *y, x_min, x_max, y_min, y_max, style);
                ui.small(value_text);
                response
            },
        )
        .inner;

    if response.dragged() || response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            let normalized = normalize_point(pos, response.rect);
            let next_x = denormalize_value(normalized.x, x_min, x_max);
            let next_y = denormalize_value(1.0 - normalized.y, y_min, y_max);
            let mut changed = false;
            if (next_x - *x).abs() > f32::EPSILON {
                *x = next_x;
                changed = true;
            }
            if (next_y - *y).abs() > f32::EPSILON {
                *y = next_y;
                changed = true;
            }
            if changed {
                response.mark_changed();
            }
        }
    }

    response
}

/// Draw a labeled slider with an explicit value label.
pub fn labeled_slider(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    range: RangeInclusive<f32>,
    value_text: &str,
) -> Response {
    ui.horizontal(|ui| {
        ui.label(label);
        let response = ui.add(egui::Slider::new(value, range.clone()).show_value(false));
        let input_id = ui.id().with(label).with("slider_input");
        let mut text = if ui.memory(|mem| mem.has_focus(input_id)) {
            ui.memory(|mem| mem.data.get_temp::<String>(input_id))
                .unwrap_or_else(|| value_text.to_string())
        } else {
            value_text.to_string()
        };
        let edit = ui
            .add(egui::TextEdit::singleline(&mut text).desired_width(64.0));
        if edit.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
            if let Some(parsed) = parse_f32_from_text(&text) {
                let min = *range.start();
                let max = *range.end();
                *value = parsed.clamp(min, max);
            }
        }
        ui.memory_mut(|mem| mem.data.insert_temp(input_id, text));
        ui.label(value_text);
        response
    })
    .inner
}

/// Draw a toggle checkbox with the given label.
pub fn toggle(ui: &mut egui::Ui, label: &str, value: &mut bool) -> Response {
    ui.checkbox(value, label)
}

/// Style configuration for the keyboard visualization.
#[derive(Debug, Clone)]
pub struct KeyboardStyle {
    /// Width of a single white key in points.
    pub white_key_width: f32,
    /// Height of the white keys in points.
    pub white_key_height: f32,
    /// Height of the black keys in points.
    pub black_key_height: f32,
    /// Relative width of the black keys (0..1) against white keys.
    pub black_key_width_ratio: f32,
    /// White key fill color.
    pub white_key_fill: Color32,
    /// White key outline color.
    pub white_key_outline: Color32,
    /// Black key fill color.
    pub black_key_fill: Color32,
    /// Black key outline color.
    pub black_key_outline: Color32,
    /// Fill color for active notes.
    pub active_fill: Color32,
    /// Outline color for active notes.
    pub active_outline: Color32,
    /// Text color for octave labels.
    pub label_color: Color32,
    /// Font size for octave labels.
    pub label_size: f32,
    /// Octave offset for labeling (host-dependent).
    pub octave_offset: i8,
}

impl Default for KeyboardStyle {
    fn default() -> Self {
        Self {
            white_key_width: 22.0,
            white_key_height: 80.0,
            black_key_height: 48.0,
            black_key_width_ratio: 0.6,
            white_key_fill: Color32::from_gray(220),
            white_key_outline: Color32::from_gray(80),
            black_key_fill: Color32::from_gray(30),
            black_key_outline: Color32::from_gray(10),
            active_fill: Color32::from_rgb(90, 200, 220),
            active_outline: Color32::from_rgb(40, 120, 140),
            label_color: Color32::from_gray(40),
            label_size: 10.0,
            octave_offset: -2,
        }
    }
}

/// Draw a simple piano keyboard visualization.
///
/// The keyboard renders white keys first, then black keys on top. `active_notes`
/// is a 128-entry MIDI map where true indicates the key should be highlighted.
pub fn keyboard(
    ui: &mut egui::Ui,
    note_range: RangeInclusive<u8>,
    active_notes: &[bool; 128],
    note_colors: Option<&[Color32; 128]>,
    style: &KeyboardStyle,
) -> Response {
    let white_notes = collect_white_notes(note_range.clone());
    let white_count = white_notes.len().max(1);
    let total_width = style.white_key_width * white_count as f32;
    let desired = Vec2::new(total_width, style.white_key_height);
    let (rect, response) = ui.allocate_exact_size(desired, Sense::hover());

    let painter = ui.painter();
    for (index, note) in white_notes.iter().enumerate() {
        let key_rect = Rect::from_min_size(
            Pos2::new(rect.left() + index as f32 * style.white_key_width, rect.top()),
            Vec2::new(style.white_key_width, style.white_key_height),
        );
        let is_active = active_notes[*note as usize];
        let (fill, outline) = if is_active {
            let color = note_colors
                .and_then(|colors| colors.get(*note as usize))
                .copied()
                .unwrap_or(style.active_fill);
            (color, darken_color(color, 0.6))
        } else {
            (style.white_key_fill, style.white_key_outline)
        };
        painter.rect_filled(key_rect, 2.0, fill);
        painter.rect_stroke(key_rect, 2.0, Stroke::new(1.0, outline), egui::StrokeKind::Inside);

        if is_c_note(*note) {
            let label = format!("C{}", midi_octave(*note, style.octave_offset));
            painter.text(
                Pos2::new(key_rect.center().x, key_rect.bottom() - 6.0),
                Align2::CENTER_BOTTOM,
                label,
                FontId::proportional(style.label_size),
                style.label_color,
            );
        }
    }

    let black_width = style.white_key_width * style.black_key_width_ratio;
    for note in note_range {
        if is_white_note(note) {
            continue;
        }
        let white_index = white_index_before(note, &white_notes);
        let x = rect.left() + (white_index as f32 + 1.0) * style.white_key_width - black_width * 0.5;
        let key_rect = Rect::from_min_size(
            Pos2::new(x, rect.top()),
            Vec2::new(black_width, style.black_key_height),
        );
        let is_active = active_notes[note as usize];
        let (fill, outline) = if is_active {
            let color = note_colors
                .and_then(|colors| colors.get(note as usize))
                .copied()
                .unwrap_or(style.active_fill);
            (color, darken_color(color, 0.6))
        } else {
            (style.black_key_fill, style.black_key_outline)
        };
        painter.rect_filled(key_rect, 2.0, fill);
        painter.rect_stroke(key_rect, 2.0, Stroke::new(1.0, outline), egui::StrokeKind::Inside);
    }

    response
}

fn draw_knob(ui: &egui::Ui, rect: Rect, t: f32, hovered: bool, active: bool, style: &KnobStyle) {
    let painter = ui.painter();
    let center = rect.center();
    let radius = rect.width().min(rect.height()) * 0.5 - 1.0;
    let base = if active {
        Color32::from_gray(60)
    } else if hovered {
        Color32::from_gray(52)
    } else {
        Color32::from_gray(45)
    };
    painter.circle_filled(center, radius, base);
    painter.circle_stroke(center, radius, Stroke::new(1.5, Color32::from_gray(20)));

    let start_angle = -5.0 * std::f32::consts::PI / 4.0;
    let end_angle = std::f32::consts::PI / 4.0;
    let angle = start_angle + (end_angle - start_angle) * t;
    let bg_points = arc_points(center, radius - 4.0, start_angle, end_angle, 32);
    painter.add(Shape::line(
        bg_points,
        Stroke::new(style.arc_stroke, Color32::from_gray(28)),
    ));
    let fg_points = arc_points(center, radius - 4.0, start_angle, angle, 32);
    painter.add(Shape::line(
        fg_points,
        Stroke::new(style.arc_stroke, style.accent),
    ));

    let indicator = center
        + egui::vec2(angle.cos(), angle.sin()) * style.indicator_radius;
    painter.line_segment([center, indicator], Stroke::new(2.0, style.accent));
}

fn draw_xy_pad(
    ui: &egui::Ui,
    rect: Rect,
    x: f32,
    y: f32,
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
    style: &XYPadStyle,
) {
    let painter = ui.painter();
    painter.rect_filled(rect, 4.0, style.background);
    painter.rect_stroke(rect, 4.0, style.outline, egui::StrokeKind::Inside);

    let t_x = normalize_value(x, x_min, x_max);
    let t_y = 1.0 - normalize_value(y, y_min, y_max);
    let pos = Pos2::new(
        rect.left() + rect.width() * t_x,
        rect.top() + rect.height() * t_y,
    );

    painter.line_segment(
        [Pos2::new(rect.left(), pos.y), Pos2::new(rect.right(), pos.y)],
        style.crosshair,
    );
    painter.line_segment(
        [Pos2::new(pos.x, rect.top()), Pos2::new(pos.x, rect.bottom())],
        style.crosshair,
    );
    painter.circle_filled(pos, style.dot_radius, style.accent);
}

fn arc_points(center: Pos2, radius: f32, start_angle: f32, end_angle: f32, steps: usize) -> Vec<Pos2> {
    let steps = steps.max(2);
    let mut points = Vec::with_capacity(steps);
    let step = (end_angle - start_angle) / (steps - 1) as f32;
    for index in 0..steps {
        let angle = start_angle + step * index as f32;
        points.push(center + egui::vec2(angle.cos(), angle.sin()) * radius);
    }
    points
}

fn angle_to_normalized(angle: f32) -> Option<f32> {
    let start_angle = -5.0 * std::f32::consts::PI / 4.0;
    let end_angle = std::f32::consts::PI / 4.0;
    if angle < start_angle || angle > end_angle {
        return None;
    }
    let span = end_angle - start_angle;
    let normalized = (angle - start_angle) / span;
    Some(normalized.clamp(0.0, 1.0))
}

fn normalize_value(value: f32, min: f32, max: f32) -> f32 {
    let span = (max - min).max(f32::EPSILON);
    ((value - min) / span).clamp(0.0, 1.0)
}

fn denormalize_value(value: f32, min: f32, max: f32) -> f32 {
    min + (max - min) * value.clamp(0.0, 1.0)
}

fn normalize_point(pos: Pos2, rect: Rect) -> Vec2 {
    let x = ((pos.x - rect.left()) / rect.width().max(1.0)).clamp(0.0, 1.0);
    let y = ((pos.y - rect.top()) / rect.height().max(1.0)).clamp(0.0, 1.0);
    Vec2::new(x, y)
}

fn parse_f32_from_text(text: &str) -> Option<f32> {
    let trimmed = text.trim().replace(',', ".");
    if trimmed.is_empty() {
        return None;
    }
    trimmed.parse::<f32>().ok()
}

fn darken_color(color: Color32, factor: f32) -> Color32 {
    let factor = factor.clamp(0.0, 1.0);
    let r = (color.r() as f32 * factor).round() as u8;
    let g = (color.g() as f32 * factor).round() as u8;
    let b = (color.b() as f32 * factor).round() as u8;
    Color32::from_rgb(r, g, b)
}

fn collect_white_notes(range: RangeInclusive<u8>) -> Vec<u8> {
    range.filter(|note| is_white_note(*note)).collect()
}

fn is_white_note(note: u8) -> bool {
    matches!(note % 12, 0 | 2 | 4 | 5 | 7 | 9 | 11)
}

fn is_c_note(note: u8) -> bool {
    note % 12 == 0
}

fn midi_octave(note: u8, offset: i8) -> i8 {
    (note / 12) as i8 + offset
}

fn white_index_before(note: u8, whites: &[u8]) -> usize {
    whites.iter().take_while(|key| **key < note).count().saturating_sub(1)
}

#[cfg(all(test, feature = "gui"))]
mod tests {
    use super::*;

    #[test]
    fn normalize_roundtrip_is_stable() {
        let value = 0.42;
        let t = normalize_value(value, 0.0, 1.0);
        let restored = denormalize_value(t, 0.0, 1.0);
        assert!((value - restored).abs() < 1e-6);
    }

    #[test]
    fn angle_to_normalized_clamps() {
        let start = -5.0 * std::f32::consts::PI / 4.0;
        let t = angle_to_normalized(start).unwrap();
        assert!((t - 0.0).abs() < 1e-6);
    }

    #[test]
    fn parse_f32_from_text_handles_commas() {
        let value = parse_f32_from_text("1,5").unwrap();
        assert!((value - 1.5).abs() < 1e-6);
    }
}
