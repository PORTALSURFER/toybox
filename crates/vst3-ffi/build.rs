//! Build script that validates the VST3 SDK location from environment.

use std::env;
use std::error::Error;
use std::path::PathBuf;

/// Ensure `VST3_SDK_DIR` points to a valid VST3 SDK tree.
fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=VST3_SDK_DIR");

    let sdk_dir = env::var("VST3_SDK_DIR").map(PathBuf::from).map_err(|_| {
        "VST3_SDK_DIR is not set. Set it to the VST3 SDK root directory \
             that contains the `pluginterfaces` folder."
    })?;

    if !sdk_dir.join("pluginterfaces").is_dir() {
        return Err(format!(
            "VST3 SDK not found at {}. Set VST3_SDK_DIR to a valid VST3 SDK root \
             (it must contain `pluginterfaces`).",
            sdk_dir.display()
        )
        .into());
    }

    println!("cargo:rerun-if-changed={}", sdk_dir.display());
    println!("cargo:rustc-env=VST3_SDK_DIR={}", sdk_dir.display());

    Ok(())
}
