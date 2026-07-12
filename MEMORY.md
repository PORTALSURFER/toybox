# MEMORY

Last Updated (UTC): 2026-07-11 23:59:07Z

## Current State

- Active user-requested task: make Toybox own the reusable macOS VST3 hosted-view lifecycle while Radiant owns all GUI rendering, then migrate Pump off its plugin-local Cocoa renderer.
- Branch `codex/radiant-vst3-embedded-host` adds the opt-in `radiant-vst3` feature and a generic `RadiantVst3Editor` / `RadiantVst3HostedGui` contract.
- The hosted view forwards AppKit lifecycle, input, resize, and redraw events to a declarative editor and renders its `SurfacePaintPlan` only through Radiant's embedded Vello renderer pinned at `6575b0c9a6b5abad17f711a36b832b7e7434e7b1`.
- Radiant now acquires and recovers the presentation surface before rendering, so a Lost/Outdated resize cannot replace the target after the frame was drawn.
- Radiant embedded validation shares the canonical encoder's clip state, so unsupported surfaces inside suppressed clips are ignored consistently.
- `RadiantVst3HostedGui::with_text_options` owns and forwards portable embedded-font policy into Radiant's embedded renderer.
- Radiant trait-based embedded renders advance a monotonic animation clock, so focused text-input carets blink through Toybox's normal redraw path.
- Validation passes: focused `radiant_host_macos` tests, `cargo clippy --features radiant-vst3 --all-targets -- -D warnings`, normal local CI, and the main-thread `radiant-vst3-host-smoke` executable rendering a gradient `FillPath` through embedded Vello.
- The hosted-view lifecycle initializes `RadiantVst3Editor::resize` after renderer creation and before storing the runtime, so the first `drawRect:` always sees the declared logical size.
- Unhandled AppKit `keyDown:` events are forwarded to `NSView`'s superclass so host shortcuts remain available while the plugin view is focused.
- Closing a hosted view tears down native resources without clearing its last logical host size, preserving dimensions across reopen.
- AppKit text dispatch uses the event's real characters, preserves Option-generated text, and leaves Command-modified shortcuts to the host responder chain.
- GitHub Actions configures the existing secret-backed private-repository URL rewrite before Cargo runs on Linux and Windows, allowing the pinned private `mts-esp-rs` dependency to resolve.
- Linux CI installs ripgrep so enforcement scripts run for real and fail closed if `rg` is unavailable; its general clippy lane excludes SDK-dependent VST3 workspace members and leaves them to the explicit optional VST3 lane.
- The intentionally deep 300-panel layout stress test uses an explicit 8 MiB test thread, preserving the extreme-depth assertion without overflowing Windows or macOS test-harness stacks.
- Win32 aspect-ratio resize tests now reflect the growth-only minimum-size contract and upward pixel rounding, covering `534x300` for 16:9 and `667x500` for 4:3 host-client sizing.
- The macOS VST3 realtime redraw driver atomically coalesces main-thread selector requests to one pending tick, with driver state installed before the worker starts so the first completion cannot race initialization.
- AppKit's `\u{7f}` delete character maps to Radiant Backspace, while `NSDeleteFunctionKey` remains the distinct forward Delete action.
- AppKit Tab, Backtab, and keypad Enter control characters map to Radiant's semantic Tab and Enter keys instead of falling through to the host responder chain.
- VST3 key callbacks dispatch converted Shift, Option, and Command state into Radiant before key handling and clear modifier state on key-up, with redraw invalidation on both transitions.
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
