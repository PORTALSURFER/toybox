//! Helpers for emitting Windows `.clap` bundles.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Paths used for Windows `.clap` bundle output.
pub struct WindowsBundlePaths {
    /// Bundle filename, including the `.clap` extension.
    pub bundle_name: String,
    /// Output path under the target directory.
    pub target_path: PathBuf,
    /// Output path under a `dist/` directory at the workspace root.
    ///
    /// If no workspace root is detected, this falls back to the package
    /// manifest directory.
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
///
/// Paths are resolved from the workspace root when `CARGO_MANIFEST_DIR` points
/// to a workspace member. If no workspace root can be detected, the package
/// manifest directory is used as the root.
pub fn windows_bundle_paths(name: &str, version: &str) -> WindowsBundlePaths {
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let manifest_dir = cargo_manifest_dir();
    let root_dir = workspace_root_from_manifest_dir(&manifest_dir);
    let target_dir = cargo_target_dir(&root_dir);
    windows_bundle_paths_from(&root_dir, Some(&target_dir), &profile, name, version)
}

/// Build the rustc link-arg used to emit a Windows `.clap` bundle.
pub fn windows_rustc_link_arg(output_path: &Path) -> String {
    format!("/OUT:\"{}\"", output_path.display())
}

fn windows_bundle_paths_from(
    root_dir: &Path,
    target_dir: Option<&Path>,
    profile: &str,
    name: &str,
    version: &str,
) -> WindowsBundlePaths {
    let bundle_name = windows_bundle_name(name, version);
    let target_dir = target_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| root_dir.join("target"));
    let target_path = target_dir.join(profile).join(&bundle_name);
    let dist_path = root_dir.join("dist").join(&bundle_name);

    WindowsBundlePaths {
        bundle_name,
        target_path,
        dist_path,
    }
}

fn cargo_target_dir(root_dir: &Path) -> PathBuf {
    let target_dir = env::var("CARGO_TARGET_DIR").ok().map(PathBuf::from);
    cargo_target_dir_from(root_dir, target_dir)
}

fn cargo_target_dir_from(root_dir: &Path, target_dir: Option<PathBuf>) -> PathBuf {
    target_dir.unwrap_or_else(|| root_dir.join("target"))
}

fn cargo_manifest_dir() -> PathBuf {
    env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")))
}

fn workspace_root_from_manifest_dir(manifest_dir: &Path) -> PathBuf {
    for ancestor in manifest_dir.ancestors() {
        if cargo_manifest_declares_workspace(ancestor.join("Cargo.toml")) {
            return ancestor.to_path_buf();
        }
    }
    manifest_dir.to_path_buf()
}

fn cargo_manifest_declares_workspace(path: PathBuf) -> bool {
    let Ok(contents) = fs::read_to_string(path) else {
        return false;
    };
    contents.lines().any(|line| line.trim() == "[workspace]")
}

#[cfg(test)]
mod tests {
    use super::{
        cargo_target_dir_from, windows_bundle_name, windows_bundle_paths_from,
        windows_rustc_link_arg, workspace_root_from_manifest_dir,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn bundle_name_includes_version() {
        assert_eq!(windows_bundle_name("lilt", "0.3.0"), "lilt-v0.3.0.clap");
    }

    #[test]
    fn link_arg_prefix_matches_lilt() {
        let arg = windows_rustc_link_arg(Path::new("dist/lilt-v0.3.0.clap"));
        assert!(arg.starts_with("/OUT:"));
        assert!(arg.contains("\"dist/lilt-v0.3.0.clap\""));
    }

    #[test]
    fn bundle_paths_resolve_under_manifest_dir() {
        let root_dir = Path::new("workspace/toybox");
        let target_dir = Path::new("workspace/target");
        let paths =
            windows_bundle_paths_from(root_dir, Some(target_dir), "release", "lilt", "0.3.0");
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

    #[test]
    fn workspace_root_falls_back_to_manifest_dir_without_workspace_file() {
        let manifest_dir = Path::new("/tmp/plugin");
        assert_eq!(workspace_root_from_manifest_dir(manifest_dir), manifest_dir);
    }

    #[test]
    fn workspace_root_resolves_for_workspace_member() {
        let temp_root = std::env::temp_dir().join(format!(
            "toybox-bundle-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock should be monotonic for tests")
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&temp_root);
        let member_dir = temp_root.join("examples/minimal-clap");
        fs::create_dir_all(&member_dir).expect("should create test workspace dirs");
        fs::write(
            temp_root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"examples/minimal-clap\"]\n",
        )
        .expect("should create workspace Cargo.toml");
        fs::write(
            member_dir.join("Cargo.toml"),
            "[package]\nname = \"example\"\nversion = \"0.1.0\"\n",
        )
        .expect("should create member Cargo.toml");

        assert_eq!(workspace_root_from_manifest_dir(&member_dir), temp_root);
        let _ = fs::remove_dir_all(&temp_root);
    }

    #[test]
    fn cargo_target_dir_prefers_explicit_override() {
        let resolved = cargo_target_dir_from(
            Path::new("workspace"),
            Some(PathBuf::from("custom-target-dir")),
        );
        assert_eq!(resolved, PathBuf::from("custom-target-dir"));
    }
}
