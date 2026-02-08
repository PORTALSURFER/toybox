//! Font loading helpers for vector text rendering.

use std::path::PathBuf;

use skrifa::prelude::FontRef;
use vello::peniko::{Blob, FontData};

use crate::logging::log_line_safe;

use super::types::LoadedFont;

/// Try to load a default sans-serif font from known platform locations.
pub(super) fn load_default_font() -> Option<LoadedFont> {
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
