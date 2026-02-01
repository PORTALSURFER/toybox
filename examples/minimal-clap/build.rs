use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.1.0".into());
    let target_dir = if let Ok(dir) = env::var("CARGO_TARGET_DIR") {
        Path::new(&dir).to_path_buf()
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")))
            .join("target")
    };
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| "linux".into());

    let bundle_name = format!("toybox-minimal-v{version}.clap");
    let bundle_path = target_dir.join(&profile).join(&bundle_name);
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")));
    let dist_path = repo_root.join("dist").join(&bundle_name);

    if target_os == "windows" {
        let output_path = if profile == "release" {
            &dist_path
        } else {
            &bundle_path
        };
        if let Some(parent) = output_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        println!("cargo:rustc-cdylib-link-arg=/OUT:{}", output_path.display());
        println!("cargo:warning=writing bundle to {}", output_path.display());
    } else {
        let artifact_src = target_dir.join(&profile).join("deps").join(artifact_name(&target_os));
        copy_artifact(&artifact_src, &bundle_path);
    }
}

fn lib_basename() -> String {
    env::var("CARGO_PKG_NAME")
        .unwrap_or_else(|_| "toybox-minimal-clap".into())
        .replace('-', "_")
}

fn artifact_name(target_os: &str) -> String {
    match target_os {
        "windows" => format!("{}.dll", lib_basename()),
        "macos" => format!("lib{}.dylib", lib_basename()),
        _ => format!("lib{}.so", lib_basename()),
    }
}

fn copy_artifact(src: &Path, dst: &Path) {
    if let Err(err) = fs::copy(src, dst) {
        eprintln!(
            "warning: failed to copy {} -> {} ({})",
            src.display(),
            dst.display(),
            err
        );
    }
}
