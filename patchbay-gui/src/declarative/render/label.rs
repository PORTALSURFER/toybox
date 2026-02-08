/// Render a label node.
fn render_label(label: &LabelSpec, rect: Rect, ui: &mut Ui<'_>, tokens: &ThemeTokens) {
    let color = label.color.unwrap_or(tokens.colors.text);
    let _ = ui.text_single_line_hard_clamped_in_rect_scaled(
        rect,
        &label.text,
        color,
        tokens.typography.text_scale,
    );
}
