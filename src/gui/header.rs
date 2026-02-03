//! Header helpers for egui-based plugin UIs.

use egui_baseview::egui::{self, Color32, RichText};

/// Represents a selectable UI scale step.
#[derive(Debug, Clone, Copy)]
pub struct ScaleStep {
    /// Label shown in the UI, e.g. "100%".
    pub label: &'static str,
    /// Numeric scale multiplier, e.g. 1.0.
    pub scale: f32,
}

/// Render the standard Toybox header with a title, subtitle, and scale picker.
///
/// Returns `true` if the selected scale index was changed.
pub fn header_with_scale(
    ui: &mut egui::Ui,
    title: &str,
    subtitle: &str,
    scale_label: &str,
    scale_steps: &[ScaleStep],
    selected: &mut usize,
) -> bool {
    let mut changed = false;
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(
                RichText::new(title)
                    .size(22.0)
                    .color(Color32::from_rgb(210, 220, 230)),
            );
            ui.label(
                RichText::new(subtitle)
                    .size(12.0)
                    .color(Color32::from_rgb(130, 150, 170)),
            );
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let current = (*selected).min(scale_steps.len().saturating_sub(1));
            let mut next = current;
            egui::ComboBox::from_label(scale_label)
                .selected_text(scale_steps[current].label)
                .show_ui(ui, |ui| {
                    for (index, step) in scale_steps.iter().enumerate() {
                        ui.selectable_value(&mut next, index, step.label);
                    }
                });
            if next != current {
                *selected = next;
                changed = true;
            }
        });
    });
    ui.add_space(4.0);
    changed
}
