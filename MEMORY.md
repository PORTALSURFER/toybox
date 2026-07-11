# MEMORY

Last Updated (UTC): 2026-07-11 12:54:39Z

## Current State

- Active user-requested task: make Toybox own the reusable macOS VST3 hosted-view lifecycle while Radiant owns all GUI rendering, then migrate Pump off its plugin-local Cocoa renderer.
- Branch `codex/radiant-vst3-embedded-host` adds the opt-in `radiant-vst3` feature and a generic `RadiantVst3Editor` / `RadiantVst3HostedGui` contract.
- The hosted view forwards AppKit lifecycle, input, resize, and redraw events to a declarative editor and renders its `SurfacePaintPlan` only through Radiant's embedded Vello renderer pinned at `9b45df71893a71f165fcae4183a189d664cecb10`.
- Validation passes: focused `radiant_host_macos` tests, `cargo clippy --features radiant-vst3 --all-targets -- -D warnings`, normal local CI, and the main-thread `radiant-vst3-host-smoke` executable rendering a gradient `FillPath` through embedded Vello.
- The hosted-view lifecycle initializes `RadiantVst3Editor::resize` after renderer creation and before storing the runtime, so the first `drawRect:` always sees the declared logical size.
- Unhandled AppKit `keyDown:` events are forwarded to `NSView`'s superclass so host shortcuts remain available while the plugin view is focused.
- Active user-requested task: make Ioskeley Mono the default Patchbay/Radiant vector text font and let Pump pick it up via a toybox revision bump.
- Branch `codex/radiant-ioskeley-default-font` vendors Ioskeley Mono v2.0.0 `Normal/Unhinted/IoskeleyMono-Regular.ttf` under `assets/IoskeleyMono/` with OFL text and source notes.
- `patchbay-gui` now prefers bundled Ioskeley Mono before the existing Sometype Mono fallback chain, while `PATCHBAY_GUI_FONT_PATH` still overrides bundled candidates.
- Focused validation passed: `cargo fmt --all -- --check`, `cargo test -p patchbay-gui bundled_font_candidates_prefer_ioskeley_mono -- --nocapture`, `VST3_SDK_DIR=/Users/portalsurfer/lib/vst3sdk cargo clippy --all-targets --all-features -- -D warnings`, and `VST3_SDK_DIR=/Users/portalsurfer/lib/vst3sdk bash scripts/ci_local.sh`.
- `VST3_SDK_DIR=/Users/portalsurfer/lib/vst3sdk cargo test --all` still aborts in the pre-existing `stress_deeply_nested_panel_tree_measures_without_failure` stack-overflow test; the local CI lane does not run that full workspace test command.
- Handoff preflight runs through `bash scripts/run_agent_request.sh`.
- Local validation runs through `bash scripts/ci_local.sh`.
- The EQ attractor surface now renders each attractor as a single filled color circle on the visual/vector draw path.
- The EQ attractor curve now emits a filled polygon plus joined vector polylines, removing the old comb-like fill lines and preserving smoother subpixel rendering.
- The EQ attractor surface now uses a shared gravity-field wave model, so attractors bend and locally slow one curve instead of layering separate local sine waves.
- The EQ attractor surface now requires real pointer travel before emitting a move, so an off-center click selects a node without nudging it.
- Regression tests cover both node ordering and the curve command shape emitted in vector mode.
- The curve editor now supports declarative beat-guide overlays, configurable snap targets, and held shortcut-key state so plugins can render brighter grids and temporarily invert snapping while a key is held.
- Win32 shortcut handling now maps Ctrl-letter `WM_CHAR` control codes back to their ASCII letters for shortcut matching, and matched Ctrl shortcuts are swallowed while text edit is active instead of leaking characters into text boxes.
- macOS VST3 entry exports use the Steinberg/Ableton lowercase `bundleEntry` and `bundleExit` symbols.
- Local preflight avoids Bash 4-only `mapfile` usage so it can run under macOS system Bash.

## Active Mission

- Own reusable plugin infrastructure in Toybox while keeping all GUI scene construction and rendering in Radiant and keeping plugins limited to DSP, state/parameters, and declarative UI composition.

## Immediate Next Actions

1. Run the full Toybox validation lane and commit/push the hosted-view API.
2. Pin Pump to the resulting Toybox revision and delete Pump's Cocoa view and primitive paint replay.
3. Build and inspect a fresh Pump VST3 in a real host before requesting review.

## Constraints And Notes

- VST3 checks remain opt-in and require `VST3_SDK_DIR`.
- Keep reusable framework behavior in `toybox`; keep plugin-specific behavior in plugin repositories.
