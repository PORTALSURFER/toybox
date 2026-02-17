/// Measure root frame size including header and padding.
fn measure_root_frame(frame: &RootFrameSpec, tokens: &ThemeTokens) -> Size {
    let content = measure_node(&frame.content, tokens);
    let header = panel_header_height(frame.title.as_deref(), tokens).max(0) as u32;
    let padding = frame.padding.max(0) as u32;
    let padding_total = padding.saturating_mul(2);
    let measured = Size {
        width: content.width.saturating_add(padding_total),
        height: content
            .height
            .saturating_add(padding_total)
            .saturating_add(header),
    };
    resolve_size(frame.layout, measured, measured)
}
