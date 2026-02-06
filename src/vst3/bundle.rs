//! Helpers for emitting Windows `.vst3` bundles.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Paths used for Windows `.vst3` bundle output.
pub struct WindowsVst3BundlePaths {
    /// Bundle directory name, including `.vst3`.
    pub bundle_name: String,
    /// Bundle root under the target directory.
    pub target_bundle_dir: PathBuf,
    /// Bundle root under `dist/`.
    pub dist_bundle_dir: PathBuf,
    /// Binary path under the target bundle.
    pub target_binary_path: PathBuf,
    /// Binary path under the dist bundle.
    pub dist_binary_path: PathBuf,
}

impl WindowsVst3BundlePaths {
    /// Select the final binary output path based on build profile.
    pub fn output_binary_path(&self, is_release: bool) -> &Path {
        if is_release {
            &self.dist_binary_path
        } else {
            &self.target_binary_path
        }
    }

    /// Select the final bundle directory based on build profile.
    pub fn output_bundle_dir(&self, is_release: bool) -> &Path {
        if is_release {
            &self.dist_bundle_dir
        } else {
            &self.target_bundle_dir
        }
    }
}

/// Build a Windows VST3 bundle name in the `name-vX.Y.Z.vst3` format.
pub fn windows_vst3_bundle_name(name: &str, version: &str) -> String {
    format!("{name}-v{version}.vst3")
}

/// Resolve Windows VST3 bundle output paths.
///
/// Bundle layout:
/// `{root}/{location}/{bundle}.vst3/Contents/x86_64-win/{bundle}.vst3`
pub fn windows_vst3_bundle_paths(name: &str, version: &str) -> WindowsVst3BundlePaths {
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let manifest_dir = cargo_manifest_dir();
    let root_dir = workspace_root_from_manifest_dir(&manifest_dir);
    let target_dir = cargo_target_dir(&root_dir);
    windows_vst3_bundle_paths_from(&root_dir, Some(&target_dir), &profile, name, version)
}

/// Build the rustc link-arg used to emit a Windows VST3 bundle binary.
pub fn windows_vst3_rustc_link_arg(output_path: &Path) -> String {
    format!("/OUT:\"{}\"", output_path.display())
}

/// Resolve bundle paths from explicit root/target/profile inputs.
fn windows_vst3_bundle_paths_from(
    root_dir: &Path,
    target_dir: Option<&Path>,
    profile: &str,
    name: &str,
    version: &str,
) -> WindowsVst3BundlePaths {
    let bundle_name = windows_vst3_bundle_name(name, version);
    let target_dir = target_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| root_dir.join("target"));

    let target_bundle_dir = target_dir.join(profile).join(&bundle_name);
    let dist_bundle_dir = root_dir.join("dist").join(&bundle_name);

    let binary_rel = Path::new("Contents").join("x86_64-win").join(&bundle_name);
    let target_binary_path = target_bundle_dir.join(&binary_rel);
    let dist_binary_path = dist_bundle_dir.join(binary_rel);

    WindowsVst3BundlePaths {
        bundle_name,
        target_bundle_dir,
        dist_bundle_dir,
        target_binary_path,
        dist_binary_path,
    }
}

/// Resolve active Cargo target directory for the current process.
fn cargo_target_dir(root_dir: &Path) -> PathBuf {
    env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| root_dir.join("target"))
}

/// Return the current `CARGO_MANIFEST_DIR`.
fn cargo_manifest_dir() -> PathBuf {
    env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")))
}

/// Walk ancestors from a manifest directory to find the nearest workspace root.
fn workspace_root_from_manifest_dir(manifest_dir: &Path) -> PathBuf {
    for ancestor in manifest_dir.ancestors() {
        if cargo_manifest_declares_workspace(ancestor.join("Cargo.toml")) {
            return ancestor.to_path_buf();
        }
    }
    manifest_dir.to_path_buf()
}

/// Return true when a Cargo manifest includes a `[workspace]` section.
fn cargo_manifest_declares_workspace(path: PathBuf) -> bool {
    let Ok(contents) = fs::read_to_string(path) else {
        return false;
    };
    contents.lines().any(|line| line.trim() == "[workspace]")
}

#[cfg(test)]
mod tests {
    use super::{
        windows_vst3_bundle_name, windows_vst3_bundle_paths_from, windows_vst3_rustc_link_arg,
    };
    use std::path::{Path, PathBuf};

    #[test]
    fn bundle_name_includes_version() {
        assert_eq!(
            windows_vst3_bundle_name("toybox-minimal", "0.1.0"),
            "toybox-minimal-v0.1.0.vst3"
        );
    }

    #[test]
    fn link_arg_uses_out_prefix() {
        let arg = windows_vst3_rustc_link_arg(Path::new("dist/plugin.vst3"));
        assert!(arg.starts_with("/OUT:"));
    }

    #[test]
    fn bundle_paths_include_windows_contents_layout() {
        let paths = windows_vst3_bundle_paths_from(
            Path::new("workspace/toybox"),
            Some(Path::new("workspace/target")),
            "release",
            "toybox-minimal",
            "0.1.0",
        );

        assert_eq!(
            paths.target_bundle_dir,
            PathBuf::from("workspace/target/release/toybox-minimal-v0.1.0.vst3")
        );
        assert_eq!(
            paths.target_binary_path,
            PathBuf::from(
                "workspace/target/release/toybox-minimal-v0.1.0.vst3/Contents/x86_64-win/toybox-minimal-v0.1.0.vst3"
            )
        );
    }
}
