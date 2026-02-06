# Packaging

## Windows `.clap` bundles

The recommended Windows flow mirrors the Lilt reference:

- Build the plugin as a `cdylib`.
- Emit a `.clap` bundle directly via the linker `/OUT:` argument.
- For release builds, write bundles into `dist/`.
- For non-release builds, write bundles into `target/{profile}/`.
- Resolve `dist/` and `target/` from the workspace root.

The minimal example plugin includes a `build.rs` that follows this layout.
On non-Windows targets it performs a no-op and prints an informational warning.

### Example (minimal plugin)

- Debug bundle: `cargo build -p toybox-minimal-clap`
- Release bundle: `cargo build -p toybox-minimal-clap --release`

When building on Windows, the bundle is written to:

- Debug: `target/debug/toybox-minimal-v{version}.clap`
- Release: `dist/toybox-minimal-v{version}.clap`

### Helper APIs

`toybox::clap::bundle` provides:

- `windows_bundle_name`
- `windows_bundle_paths`
- `windows_rustc_link_arg`

Use these helpers if you want a custom `build.rs` or an `xtask` wrapper.
