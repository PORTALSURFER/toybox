//! Build script that validates the local VST3 SDK submodule location.

use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};

/// Ensure the VST3 SDK submodule is present.
fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let workspace_root = workspace_root_from_manifest_dir(&manifest_dir);
    let default_sdk_dir = workspace_root.join("third_party/vst3sdk");
    let sdk_dir = env::var("TOYBOX_VST3SDK_DIR")
        .map(PathBuf::from)
        .unwrap_or(default_sdk_dir);

    if !sdk_dir.join("pluginterfaces").is_dir() {
        return Err(format!(
            "VST3 SDK not found at {}. Initialize submodules with `git submodule update --init --recursive`, or set TOYBOX_VST3SDK_DIR.",
            sdk_dir.display()
        )
        .into());
    }

    println!("cargo:rerun-if-changed={}", sdk_dir.display());
    println!("cargo:rustc-env=TOYBOX_VST3SDK_DIR={}", sdk_dir.display());

    Ok(())
}

/// Walk ancestors from a manifest directory to find the nearest workspace root.
fn workspace_root_from_manifest_dir(manifest_dir: &Path) -> PathBuf {
    for ancestor in manifest_dir.ancestors() {
        if cargo_manifest_declares_workspace(&ancestor.join("Cargo.toml")) {
            return ancestor.to_path_buf();
        }
    }
    manifest_dir.to_path_buf()
}

/// Return true when a Cargo manifest includes a `[workspace]` section.
fn cargo_manifest_declares_workspace(path: &Path) -> bool {
    let Ok(contents) = std::fs::read_to_string(path) else {
        return false;
    };
    contents.lines().any(|line| line.trim() == "[workspace]")
}
