//! Build script that emits a Windows `.clap` bundle output path.

use std::env;
use std::fs;

use toybox::clap::bundle::{windows_bundle_paths, windows_rustc_link_arg};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.1.0".into());
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| "linux".into());
    let paths = windows_bundle_paths("toybox-minimal", &version);
    let output_path = if profile == "release" {
        &paths.dist_path
    } else {
        &paths.target_path
    };

    if target_os == "windows" {
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).unwrap_or_else(|err| {
                panic!(
                    "failed to create bundle output directory {}: {err}",
                    parent.display()
                );
            });
        }
        let link_arg = windows_rustc_link_arg(output_path);
        println!("cargo:rustc-cdylib-link-arg={link_arg}");
        println!("cargo:warning=writing bundle to {}", output_path.display());
    } else {
        println!(
            "cargo:warning=skipping .clap bundle emission on non-Windows target ({target_os})"
        );
    }
}
