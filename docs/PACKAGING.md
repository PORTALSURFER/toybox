# Packaging

## Windows `.clap` bundles

The recommended Windows CLAP flow mirrors the Lilt reference:

- Build the plugin as a `cdylib`.
- Emit a `.clap` bundle directly via the linker `/OUT:` argument.
- For release builds, write bundles into `dist/`.
- For non-release builds, write bundles into `target/{profile}/`.
- Resolve `dist/` and `target/` from the workspace root.

The minimal CLAP example includes a `build.rs` that follows this layout.
On non-Windows targets it performs a no-op and prints an informational warning.

### CLAP example

- Debug bundle: `cargo build -p toybox-minimal-clap`
- Release bundle: `cargo build -p toybox-minimal-clap --release`

On Windows, output paths are:

- Debug: `target/debug/toybox-minimal-v{version}.clap`
- Release: `dist/toybox-minimal-v{version}.clap`

### CLAP helper APIs

CLAP build scripts use `toybox::bundle::windows` with:

- `WindowsBundleFormat::Clap`
- `windows_bundle_name`
- `windows_bundle_paths`
- `windows_rustc_link_arg`

## Windows `.vst3` bundles

The VST3 helper flow follows the standard bundle directory shape:

- Build the plugin as a `cdylib`.
- Emit the final binary to:
  - `{bundle}.vst3/Contents/x86_64-win/{bundle}.vst3`
- For release builds, write bundles into `dist/`.
- For non-release builds, write bundles into `target/{profile}/`.
- Resolve `dist/` and `target/` from the workspace root.

The minimal VST3 example includes a `build.rs` that applies this layout.
On non-Windows targets it performs a no-op and prints an informational warning.

### VST3 example

- Debug bundle: `cargo build -p toybox-minimal-vst3 --features toybox/vst3`
- Release bundle: `cargo build -p toybox-minimal-vst3 --features toybox/vst3 --release`

When `VST3_SDK_DIR` is not set, non-Windows or minimal workflows should build the
non-VST3 targets without invoking the VST3 bundle path.

On Windows, output paths are:

- Debug binary:
  `target/debug/toybox-minimal-v{version}.vst3/Contents/x86_64-win/toybox-minimal-v{version}.vst3`
- Release binary:
  `dist/toybox-minimal-v{version}.vst3/Contents/x86_64-win/toybox-minimal-v{version}.vst3`

### VST3 helper APIs

VST3 build scripts use `toybox::bundle::windows` with:

- `WindowsBundleFormat::Vst3`
- `windows_bundle_name`
- `windows_bundle_paths`
- `windows_rustc_link_arg`

## SDK requirement for VST3

VST3 builds require `VST3_SDK_DIR` to be set to a valid VST3 SDK root
directory (it must contain `pluginterfaces`).
