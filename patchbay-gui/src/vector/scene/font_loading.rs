//! Font loading helpers for vector text rendering.

use std::path::PathBuf;

use skrifa::prelude::FontRef;
use vello::peniko::{Blob, FontData};

use crate::logging::log_line_safe;

use super::types::LoadedFont;

/// Try to load a default UI font from bundled assets and known platform locations.
pub(super) fn load_default_font() -> Option<LoadedFont> {
    let mut candidates = bundled_font_candidates();

    if let Some(path) = std::env::var_os("PATCHBAY_GUI_FONT_PATH")
        .map(PathBuf::from)
        .filter(|path| path.exists())
    {
        candidates.insert(0, path);
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

/// Return bundled font candidates in default preference order.
fn bundled_font_candidates() -> Vec<PathBuf> {
    // Prefer the repository-bundled monospace faces for stable knob/value text
    // alignment across hosts and developer machines.
    let ioskeley_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../assets/IoskeleyMono");
    let sometype_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../assets/Sometype_Mono");

    vec![
        ioskeley_root.join("IoskeleyMono-Regular.ttf"),
        sometype_root.join("static/SometypeMono-Regular.ttf"),
        sometype_root.join("SometypeMono-VariableFont_wght.ttf"),
        sometype_root.join("static/SometypeMono-Medium.ttf"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ioskeley Mono remains the first bundled default and can be parsed by the
    /// text renderer.
    #[test]
    fn bundled_font_candidates_prefer_ioskeley_mono() {
        let candidates = bundled_font_candidates();

        let first = candidates.first().expect("bundled font candidate");
        assert!(first.ends_with("assets/IoskeleyMono/IoskeleyMono-Regular.ttf"));
        assert!(first.exists(), "missing bundled font: {}", first.display());

        let bytes = std::fs::read(first).expect("read bundled Ioskeley Mono font");
        FontRef::from_index(bytes.as_slice(), 0).expect("bundled Ioskeley Mono parses");
    }
}
