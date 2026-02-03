//! Standard layout helpers for egui-based plugin UIs.

use egui_baseview::egui::{self, Color32, Frame};

use crate::gui::header::{header_with_scale, ScaleStep};

/// Render a common Toybox window layout with a header and a content panel.
///
/// Returns `true` if the selected scale index changed.
pub fn standard_layout<F>(
    ctx: &egui::Context,
    header_bg: Color32,
    content_bg: Color32,
    title: &str,
    subtitle: &str,
    scale_label: &str,
    scale_steps: &[ScaleStep],
    selected_scale: &mut usize,
    scale: f32,
    content: F,
) -> bool
where
    F: FnOnce(&mut egui::Ui),
{
    let mut changed = false;

    egui::TopBottomPanel::top("header")
        .frame(Frame::NONE.fill(header_bg))
        .show(ctx, |ui| {
            if header_with_scale(
                ui,
                title,
                subtitle,
                scale_label,
                scale_steps,
                selected_scale,
                scale,
            ) {
                changed = true;
            }
        });

    egui::CentralPanel::default()
        .frame(Frame::NONE.fill(content_bg))
        .show(ctx, |ui| {
            content(ui);
        });

    changed
}
