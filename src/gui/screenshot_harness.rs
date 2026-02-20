//! Screenshot harness helpers for Patchbay declarative UIs.
//!
//! Plugin repos frequently want a minimal "does the UI render and lay out
//! correctly at a few sizes" check. Historically each plugin duplicated:
//!
//! - env gating (`TOYBOX_UI_SCREENSHOT`)
//! - a size matrix (base size plus a few scaled variants)
//! - headless rendering (`render_spec_to_frame`)
//! - PNG writing and output path conventions
//!
//! This module centralizes that logic so plugins can implement the canonical
//! `screenshot_renders_initial_ui` test as a small wrapper.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use image::{ImageBuffer, Rgba};

use super::{InputState, Size, declarative::UiSpec, render_spec_to_frame};

/// Default screenshot directory used when `TOYBOX_UI_SCREENSHOT_DIR` is not set.
pub const DEFAULT_SCREENSHOT_ROOT: &str = "target/ui-screenshots";
/// Optional screenshot output variant subdirectory.
///
/// When this env var is set to a valid token (for example: `normal`,
/// `debug-layout`), screenshots are written under:
/// `$(TOYBOX_UI_SCREENSHOT_DIR)/<plugin>/<variant>/...`.
///
/// Invalid values are ignored and the harness falls back to the legacy path:
/// `$(TOYBOX_UI_SCREENSHOT_DIR)/<plugin>/...`.
const SCREENSHOT_VARIANT_ENV: &str = "TOYBOX_UI_SCREENSHOT_VARIANT";

/// Return true when screenshot capture is enabled for the current test run.
///
/// The harness enables screenshot output when `TOYBOX_UI_SCREENSHOT` is set to a
/// non-zero value. The suite scripts set this to `"1"`.
pub fn screenshots_enabled() -> bool {
    match env::var("TOYBOX_UI_SCREENSHOT") {
        Ok(value) => value != "0",
        Err(_) => false,
    }
}

/// Standard screenshot size set used by the suite.
///
/// The returned sizes are derived from a base window size and match the
/// historical per-plugin contract:
///
/// - base
/// - 0.75x
/// - 1.25x
/// - 1.5x
pub fn default_screenshot_sizes(base: Size) -> [Size; 4] {
    let base_w = base.width.max(1);
    let base_h = base.height.max(1);
    [
        Size {
            width: base_w,
            height: base_h,
        },
        Size {
            width: (base_w.saturating_mul(3) / 4).max(1),
            height: (base_h.saturating_mul(3) / 4).max(1),
        },
        Size {
            width: (base_w.saturating_mul(5) / 4).max(1),
            height: (base_h.saturating_mul(5) / 4).max(1),
        },
        Size {
            width: (base_w.saturating_mul(3) / 2).max(1),
            height: (base_h.saturating_mul(3) / 2).max(1),
        },
    ]
}

/// Resolve the screenshot output root for the current run.
pub fn screenshot_output_root() -> PathBuf {
    env::var_os("TOYBOX_UI_SCREENSHOT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_SCREENSHOT_ROOT))
}

/// Resolve the optional screenshot variant from the environment.
fn screenshot_output_variant() -> Option<String> {
    env::var(SCREENSHOT_VARIANT_ENV)
        .ok()
        .and_then(|raw| normalize_screenshot_variant(&raw))
}

/// Normalize a variant token used as a subdirectory component.
///
/// Accepted characters are ASCII alphanumeric, `-`, and `_`.
/// Invalid or empty inputs are rejected.
fn normalize_screenshot_variant(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.len() > 64 {
        return None;
    }
    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return None;
    }
    Some(trimmed.to_ascii_lowercase())
}

/// Build the output directory from root/plugin and optional variant.
fn screenshot_output_dir(root: &Path, plugin: &str, variant: Option<&str>) -> PathBuf {
    let plugin_dir = root.join(plugin);
    match variant {
        Some(value) => plugin_dir.join(value),
        None => plugin_dir,
    }
}

/// Resolve the output file path for one screenshot.
///
/// The returned path follows the suite contract:
/// `$(TOYBOX_UI_SCREENSHOT_DIR)/<plugin>/initial-ui-<width>x<height>.png`.
/// When `TOYBOX_UI_SCREENSHOT_VARIANT` is set to a valid token, the path is:
/// `$(TOYBOX_UI_SCREENSHOT_DIR)/<plugin>/<variant>/initial-ui-<width>x<height>.png`.
pub fn screenshot_output_path(plugin: &str, width: u32, height: u32) -> Result<PathBuf, String> {
    let root = screenshot_output_root();
    let variant = screenshot_output_variant();
    let dir = screenshot_output_dir(&root, plugin, variant.as_deref());
    fs::create_dir_all(&dir).map_err(|err| format!("create screenshot directory failed: {err}"))?;
    Ok(dir.join(format!("initial-ui-{width}x{height}.png")))
}

/// Write an RGBA8 buffer as a PNG file.
pub fn write_png_rgba8(
    path: impl AsRef<Path>,
    width: u32,
    height: u32,
    pixels: Vec<u8>,
) -> Result<(), String> {
    let path = path.as_ref();
    let image = ImageBuffer::<Rgba<u8>, _>::from_vec(width, height, pixels)
        .ok_or_else(|| "failed to build image buffer".to_string())?;
    image
        .save(path)
        .map_err(|err| format!("save PNG failed: {err}"))?;
    Ok(())
}

/// Render and write `initial-ui-*.png` screenshots for the given `UiSpec` builder.
///
/// This function does nothing when `TOYBOX_UI_SCREENSHOT` is not enabled. It is
/// intended to be called from a `#[test]` named `screenshot_renders_initial_ui`.
pub fn capture_initial_ui_screenshots_if_enabled<Build>(
    plugin: &str,
    base_size: Size,
    mut build_spec: Build,
) -> Result<(), String>
where
    Build: FnMut(&InputState) -> UiSpec,
{
    if !screenshots_enabled() {
        return Ok(());
    }

    for size in default_screenshot_sizes(base_size) {
        let frame = render_spec_to_frame(size, &mut build_spec).map_err(|err| {
            format!(
                "headless render failed ({plugin} {0}x{1}): {err}",
                size.width, size.height
            )
        })?;
        if frame.width != size.width || frame.height != size.height {
            return Err(format!(
                "headless render size mismatch ({plugin}): got {}x{}, expected exactly {}x{}",
                frame.width, frame.height, size.width, size.height
            ));
        }

        let path = screenshot_output_path(plugin, size.width, size.height)?;
        write_png_rgba8(&path, frame.width, frame.height, frame.pixels)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{normalize_screenshot_variant, screenshot_output_dir};

    #[test]
    fn normalize_screenshot_variant_accepts_expected_tokens() {
        assert_eq!(
            normalize_screenshot_variant("normal"),
            Some("normal".to_string())
        );
        assert_eq!(
            normalize_screenshot_variant("DEBUG-LAYOUT"),
            Some("debug-layout".to_string())
        );
        assert_eq!(
            normalize_screenshot_variant("baseline_2"),
            Some("baseline_2".to_string())
        );
    }

    #[test]
    fn normalize_screenshot_variant_rejects_empty_or_invalid_values() {
        assert_eq!(normalize_screenshot_variant(""), None);
        assert_eq!(normalize_screenshot_variant("   "), None);
        assert_eq!(normalize_screenshot_variant("../debug"), None);
        assert_eq!(normalize_screenshot_variant("debug/layout"), None);
        assert_eq!(normalize_screenshot_variant("debug layout"), None);
    }

    #[test]
    fn screenshot_output_dir_uses_variant_when_present() {
        let root = Path::new("target/ui-screenshots");
        let with_variant = screenshot_output_dir(root, "pump", Some("debug-layout"));
        assert_eq!(
            with_variant,
            Path::new("target/ui-screenshots/pump/debug-layout")
        );
        let without_variant = screenshot_output_dir(root, "pump", None);
        assert_eq!(without_variant, Path::new("target/ui-screenshots/pump"));
    }
}
