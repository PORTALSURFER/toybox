/// Render a text-box node.
fn render_text_box(text_box: &TextBoxSpec, rect: Rect, ui: &mut Ui<'_>, tokens: &ThemeTokens) {
    let color = text_box.color.unwrap_or(tokens.colors.text);
    let _ = ui.text_single_line_hard_clamped_in_rect_scaled(
        rect,
        &text_box.text,
        color,
        tokens.typography.text_scale,
    );
}
