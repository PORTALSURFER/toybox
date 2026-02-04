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

impl WindowsBundlePaths {
    /// Select the output path based on the build profile.
    ///
    /// Lilt writes release bundles into `dist/` and non-release bundles into
    /// `target/{profile}`. This mirrors that behavior.
    pub fn output_path(&self, is_release: bool) -> &Path {
        if is_release {
            &self.dist_path
        } else {
            &self.target_path
        }
    }
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
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let manifest_dir = cargo_manifest_dir();
    let target_dir = cargo_target_dir(&manifest_dir);
    windows_bundle_paths_from(
        &manifest_dir,
        Some(&target_dir),
        &profile,
        name,
        version,
    )
}

/// Build the rustc link-arg used to emit a Windows `.clap` bundle.
pub fn windows_rustc_link_arg(output_path: &Path) -> String {
    format!("/OUT:{}", output_path.display())
}

fn windows_bundle_paths_from(
    manifest_dir: &Path,
    target_dir: Option<&Path>,
    profile: &str,
    name: &str,
    version: &str,
) -> WindowsBundlePaths {
    let bundle_name = windows_bundle_name(name, version);
    let target_dir = target_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| manifest_dir.join("target"));
    let target_path = target_dir.join(profile).join(&bundle_name);
    let dist_path = manifest_dir.join("dist").join(&bundle_name);

    WindowsBundlePaths {
        bundle_name,
        target_path,
        dist_path,
    }
}

fn cargo_target_dir(manifest_dir: &Path) -> PathBuf {
    if let Ok(dir) = env::var("CARGO_TARGET_DIR") {
        PathBuf::from(dir)
    } else {
        manifest_dir.join("target")
    }
}

fn cargo_manifest_dir() -> PathBuf {
    env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")))
}

#[cfg(test)]
mod tests {
    use super::{windows_bundle_name, windows_bundle_paths_from, windows_rustc_link_arg};
    use std::path::{Path, PathBuf};

    #[test]
    fn bundle_name_includes_version() {
        assert_eq!(windows_bundle_name("lilt", "0.3.0"), "lilt-v0.3.0.clap");
    }

    #[test]
    fn link_arg_prefix_matches_lilt() {
        let arg = windows_rustc_link_arg(Path::new("dist/lilt-v0.3.0.clap"));
        assert!(arg.starts_with("/OUT:"));
    }

    #[test]
    fn bundle_paths_resolve_under_manifest_dir() {
        let manifest_dir = Path::new("workspace/toybox");
        let target_dir = Path::new("workspace/target");
        let paths = windows_bundle_paths_from(
            manifest_dir,
            Some(target_dir),
            "release",
            "lilt",
            "0.3.0",
        );
        assert_eq!(paths.bundle_name, "lilt-v0.3.0.clap");
        assert_eq!(
            paths.target_path,
            PathBuf::from("workspace/target/release/lilt-v0.3.0.clap")
        );
        assert_eq!(
            paths.dist_path,
            PathBuf::from("workspace/toybox/dist/lilt-v0.3.0.clap")
        );
    }
}
