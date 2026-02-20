//! Helpers for emitting Windows plugin bundles.
//!
//! These helpers keep CLAP and VST3 build scripts on one shared path resolver
//! and linker argument formatter.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Windows plugin bundle formats supported by toybox build helpers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowsBundleFormat {
    /// CLAP file bundle (`*.clap`).
    Clap,
    /// VST3 directory bundle (`*.vst3/Contents/x86_64-win/*.vst3`).
    Vst3,
}

impl WindowsBundleFormat {
    /// Return the file extension used by this bundle format, without a leading dot.
    fn extension(self) -> &'static str {
        match self {
            Self::Clap => "clap",
            Self::Vst3 => "vst3",
        }
    }
}

/// Resolved output paths for Windows plugin bundles.
pub struct WindowsBundlePaths {
    /// Bundle filename/directory name, including extension.
    pub bundle_name: String,
    /// Final linker output path under the target directory.
    pub target_path: PathBuf,
    /// Final linker output path under `dist/`.
    pub dist_path: PathBuf,
    /// Optional bundle directory under the target directory.
    ///
    /// This is `Some` for VST3 bundles and `None` for CLAP bundles.
    pub target_bundle_dir: Option<PathBuf>,
    /// Optional bundle directory under `dist/`.
    ///
    /// This is `Some` for VST3 bundles and `None` for CLAP bundles.
    pub dist_bundle_dir: Option<PathBuf>,
}

impl WindowsBundlePaths {
    /// Select the final linker output path for the selected profile.
    pub fn output_path(&self, is_release: bool) -> &Path {
        if is_release {
            &self.dist_path
        } else {
            &self.target_path
        }
    }

    /// Select the bundle directory for the selected profile.
    ///
    /// CLAP bundles do not use a containing directory, so this returns `None`
    /// when the format is CLAP.
    pub fn output_bundle_dir(&self, is_release: bool) -> Option<&Path> {
        if is_release {
            self.dist_bundle_dir.as_deref()
        } else {
            self.target_bundle_dir.as_deref()
        }
    }
}

/// Build a Windows bundle name in the `name-vX.Y.Z.<ext>` format.
pub fn windows_bundle_name(format: WindowsBundleFormat, name: &str, version: &str) -> String {
    format!("{name}-v{version}.{}", format.extension())
}

/// Resolve Windows bundle output paths for CLAP or VST3 build scripts.
///
/// Paths are resolved from the workspace root when `CARGO_MANIFEST_DIR` points
/// to a workspace member. If no workspace root can be detected, the package
/// manifest directory is used as the root.
pub fn windows_bundle_paths(
    format: WindowsBundleFormat,
    name: &str,
    version: &str,
) -> WindowsBundlePaths {
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let manifest_dir = cargo_manifest_dir();
    let root_dir = workspace_root_from_manifest_dir(&manifest_dir);
    let target_dir = cargo_target_dir(&root_dir);
    windows_bundle_paths_from(
        &root_dir,
        Some(&target_dir),
        &profile,
        format,
        name,
        version,
    )
}

/// Build the rustc link-arg used to emit a Windows plugin bundle payload.
pub fn windows_rustc_link_arg(output_path: &Path) -> String {
    format!("/OUT:\"{}\"", output_path.display())
}

/// Resolve bundle paths from explicit root/target/profile inputs.
fn windows_bundle_paths_from(
    root_dir: &Path,
    target_dir: Option<&Path>,
    profile: &str,
    format: WindowsBundleFormat,
    name: &str,
    version: &str,
) -> WindowsBundlePaths {
    let bundle_name = windows_bundle_name(format, name, version);
    let target_dir = target_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| root_dir.join("target"));

    match format {
        WindowsBundleFormat::Clap => {
            let target_path = target_dir.join(profile).join(&bundle_name);
            let dist_path = root_dir.join("dist").join(&bundle_name);
            WindowsBundlePaths {
                bundle_name,
                target_path,
                dist_path,
                target_bundle_dir: None,
                dist_bundle_dir: None,
            }
        }
        WindowsBundleFormat::Vst3 => {
            let target_bundle_dir = target_dir.join(profile).join(&bundle_name);
            let dist_bundle_dir = root_dir.join("dist").join(&bundle_name);
            let binary_rel = Path::new("Contents").join("x86_64-win").join(&bundle_name);
            let target_path = target_bundle_dir.join(&binary_rel);
            let dist_path = dist_bundle_dir.join(&binary_rel);

            WindowsBundlePaths {
                bundle_name,
                target_path,
                dist_path,
                target_bundle_dir: Some(target_bundle_dir),
                dist_bundle_dir: Some(dist_bundle_dir),
            }
        }
    }
}

/// Resolve the active Cargo target directory for the current process.
fn cargo_target_dir(root_dir: &Path) -> PathBuf {
    let target_dir = env::var("CARGO_TARGET_DIR").ok().map(PathBuf::from);
    cargo_target_dir_from(root_dir, target_dir)
}

/// Resolve the target directory, honoring an explicit override when supplied.
fn cargo_target_dir_from(root_dir: &Path, target_dir: Option<PathBuf>) -> PathBuf {
    target_dir.unwrap_or_else(|| root_dir.join("target"))
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
        WindowsBundleFormat, cargo_target_dir_from, windows_bundle_name, windows_bundle_paths_from,
        windows_rustc_link_arg, workspace_root_from_manifest_dir,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn clap_bundle_name_includes_version() {
        assert_eq!(
            windows_bundle_name(WindowsBundleFormat::Clap, "lilt", "0.3.0"),
            "lilt-v0.3.0.clap"
        );
    }

    #[test]
    fn vst3_bundle_name_includes_version() {
        assert_eq!(
            windows_bundle_name(WindowsBundleFormat::Vst3, "toybox-minimal", "0.1.0"),
            "toybox-minimal-v0.1.0.vst3"
        );
    }

    #[test]
    fn link_arg_uses_out_prefix() {
        let arg = windows_rustc_link_arg(Path::new("dist/plugin.vst3"));
        assert!(arg.starts_with("/OUT:"));
    }

    #[test]
    fn clap_bundle_paths_resolve_under_manifest_dir() {
        let root_dir = Path::new("workspace/toybox");
        let target_dir = Path::new("workspace/target");
        let paths = windows_bundle_paths_from(
            root_dir,
            Some(target_dir),
            "release",
            WindowsBundleFormat::Clap,
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
        assert_eq!(paths.target_bundle_dir, None);
        assert_eq!(paths.dist_bundle_dir, None);
    }

    #[test]
    fn vst3_bundle_paths_include_windows_contents_layout() {
        let paths = windows_bundle_paths_from(
            Path::new("workspace/toybox"),
            Some(Path::new("workspace/target")),
            "release",
            WindowsBundleFormat::Vst3,
            "toybox-minimal",
            "0.1.0",
        );

        assert_eq!(
            paths.target_bundle_dir,
            Some(PathBuf::from(
                "workspace/target/release/toybox-minimal-v0.1.0.vst3"
            ))
        );
        assert_eq!(
            paths.target_path,
            PathBuf::from(
                "workspace/target/release/toybox-minimal-v0.1.0.vst3/Contents/x86_64-win/toybox-minimal-v0.1.0.vst3"
            )
        );
    }

    #[test]
    fn output_bundle_dir_is_absent_for_clap() {
        let paths = windows_bundle_paths_from(
            Path::new("workspace/toybox"),
            Some(Path::new("workspace/target")),
            "debug",
            WindowsBundleFormat::Clap,
            "toybox-minimal",
            "0.1.0",
        );
        assert!(paths.output_bundle_dir(false).is_none());
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
