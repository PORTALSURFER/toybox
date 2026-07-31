//! License-safe typography defaults shared by Radiant-hosted editors.
//!
//! The bundled faces are embedded in preference order: Ioskeley Mono first,
//! followed by Sometype Mono. Both assets are distributed under the SIL Open
//! Font License 1.1; the adjacent `assets/IoskeleyMono/README.txt`,
//! `assets/IoskeleyMono/OFL.txt`, `assets/Sometype_Mono/README.txt`, and
//! `assets/Sometype_Mono/OFL.txt` files carry the source and license notices.
//!
//! Radiant still owns the native/system fallback chain. These options only
//! prepend the two application-owned faces to that chain.

/// Return the shared embedded-font preference for Radiant native text.
pub fn bundled_text_options() -> radiant::runtime::NativeTextOptions {
    radiant::runtime::NativeTextOptions::default()
        .embedded_font(radiant::runtime::EmbeddedFont::from_static(include_bytes!(
            "../../assets/IoskeleyMono/IoskeleyMono-Regular.ttf"
        )))
        .embedded_font(radiant::runtime::EmbeddedFont::from_static(include_bytes!(
            "../../assets/Sometype_Mono/static/SometypeMono-Regular.ttf"
        )))
}

/// Create a headless Radiant capture using the live macOS host's font policy.
///
/// `logical_size` sets the capture viewport in logical pixels. `dpi_scale` scales
/// that viewport to physical pixels in the captured RGBA output.
///
/// Returns [`radiant::gui_runtime::EmbeddedVelloError`] when the offscreen Vello
/// renderer cannot initialize, such as when no compatible GPU device is available.
pub fn bundled_offscreen_capture(
    logical_size: radiant::gui::types::Vector2,
    dpi_scale: radiant::theme::DpiScale,
) -> Result<radiant::gui_runtime::OffscreenVelloCapture, radiant::gui_runtime::EmbeddedVelloError> {
    radiant::gui_runtime::OffscreenVelloCapture::new_with_text_options(
        logical_size,
        dpi_scale,
        &bundled_text_options(),
    )
}
