# Active Todo Queue

Mission: deliver OPT-1175 so downstream plugins can use the CLAP latency extension exclusively through Toybox's re-export.

## Active PR

- Branch: `wsvasek/opt-1175-toybox-enable-and-verify-the-clap-latency-extension-re`
- PR: intended ready-for-review OPT-1175 pull request
- Scope: enable the existing `clack-extensions` latency feature, verify `PluginLatency` and `PluginLatencyImpl` through a Toybox-only downstream fixture, and keep latency values and oversampling policy plugin-owned
- Definition of Done: downstream CLAP latency registration through Toybox only; a known nonzero behavior fixture; no direct downstream `clack-extensions` dependency; formatting, warnings-denied clippy, canonical local CI, and all workspace tests pass
- Status: `validated`

## Immediate Queue

1. Commit and push the validated OPT-1175 change.
2. Open a ready-for-review PR and wait for explicit user review/sign-off before merge.
3. After merge, let Kickforge OPT-1152 repin Toybox and report its plugin-owned 124-sample latency in CLAP and VST3.

## Validation Note

- `examples/minimal-clap` now depends only on Toybox, imports `PluginLatency` and `PluginLatencyImpl` through `toybox::clack_extensions`, registers the extension, and reports a fixed nonzero 124-sample fixture value.
- Canonical CI now runs the minimal CLAP behavior fixture instead of compile-checking it only.
- `VST3_SDK_DIR=/Users/portalsurfer/lib/vst3sdk bash scripts/ci_local.sh`: passed, including the fixture, warnings-denied GUI/VST3 clippy, and 128 VST3-feature tests.
- `VST3_SDK_DIR=/Users/portalsurfer/lib/vst3sdk cargo clippy --all-targets -- -D warnings`: passed.
- `VST3_SDK_DIR=/Users/portalsurfer/lib/vst3sdk cargo test --all`: passed, including 304 Patchbay GUI tests, 128 Toybox VST3-feature tests, and the minimal CLAP latency fixture.
