//! Build script that emits a Windows `.clap` bundle output path.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.1.0".into());
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| "linux".into());
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = workspace_root_from_manifest_dir(manifest_dir);
    let target_dir = if let Ok(dir) = env::var("CARGO_TARGET_DIR") {
        PathBuf::from(dir)
    } else {
        workspace_root.join("target")
    };

    let bundle_name = format!("toybox-minimal-v{version}.clap");
    let bundle_path = target_dir.join(&profile).join(&bundle_name);
    let dist_path = workspace_root.join("dist").join(&bundle_name);

    if target_os == "windows" {
        let output_path = if profile == "release" {
            &dist_path
        } else {
            &bundle_path
        };
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).unwrap_or_else(|err| {
                panic!(
                    "failed to create bundle output directory {}: {err}",
                    parent.display()
                );
            });
        }
        println!("cargo:rustc-cdylib-link-arg=/OUT:{}", output_path.display());
        println!("cargo:warning=writing bundle to {}", output_path.display());
    } else {
        println!(
            "cargo:warning=skipping .clap bundle emission on non-Windows target ({target_os})"
        );
    }
}

/// Walk upward from the crate manifest directory to find the workspace root.
fn workspace_root_from_manifest_dir(manifest_dir: &Path) -> PathBuf {
    for ancestor in manifest_dir.ancestors() {
        if cargo_manifest_declares_workspace(ancestor.join("Cargo.toml")) {
            return ancestor.to_path_buf();
        }
    }
    manifest_dir.to_path_buf()
}

/// Returns `true` when the manifest file declares a Cargo workspace section.
fn cargo_manifest_declares_workspace(path: PathBuf) -> bool {
    let Ok(contents) = fs::read_to_string(path) else {
        return false;
    };
    contents.lines().any(|line| line.trim() == "[workspace]")
}
