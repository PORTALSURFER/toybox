//! Helpers for emitting Windows `.clap` bundles.

use std::env;
use std::path::{Path, PathBuf};

/// Paths used for Windows `.clap` bundle output.
pub struct WindowsBundlePaths {
    /// Bundle filename, including the `.clap` extension.
    pub bundle_name: String,
    /// Output path under the target directory.
    pub target_path: PathBuf,
    /// Output path under a `dist/` directory at the repo root.
    pub dist_path: PathBuf,
}

/// Build a Windows bundle name in the `name-vX.Y.Z.clap` format.
pub fn windows_bundle_name(name: &str, version: &str) -> String {
    format!("{name}-v{version}.clap")
}

/// Resolve the Windows bundle output paths, following the Lilt layout.
///
/// This mirrors the Lilt build script behavior by emitting bundles to:
/// - `target/{profile}/{name}-v{version}.clap`
/// - `dist/{name}-v{version}.clap`
pub fn windows_bundle_paths(name: &str, version: &str) -> WindowsBundlePaths {
    let bundle_name = windows_bundle_name(name, version);
    let target_dir = cargo_target_dir();
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let target_path = target_dir.join(&profile).join(&bundle_name);
    let repo_root = cargo_manifest_dir()
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| cargo_manifest_dir());
    let dist_path = repo_root.join("dist").join(&bundle_name);

    WindowsBundlePaths {
        bundle_name,
        target_path,
        dist_path,
    }
}

fn cargo_target_dir() -> PathBuf {
    if let Ok(dir) = env::var("CARGO_TARGET_DIR") {
        PathBuf::from(dir)
    } else {
        cargo_manifest_dir()
            .parent()
            .unwrap_or_else(|| cargo_manifest_dir())
            .join("target")
    }
}

fn cargo_manifest_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}
