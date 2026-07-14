# MEMORY

Last Updated (UTC): 2026-07-14 11:58:47Z

## Current State

- Active user-requested task: implement OPT-1169, modifier-gated grouped curve-segment dragging and dedicated feedback in the reusable Patchbay curve editor.
- Branch `wsvasek/opt-1169-toybox-add-modifier-gated-grouped-curve-segment-dragging-and` adds `.curve_segment_move(CurveSegmentMoveOptions)` as the opt-in contract while keeping legacy unmodified near-segment dragging as the default.
- Command-hover and Command-press now select a complete segment before direct-line insertion, while point interaction, empty-canvas insertion, Alt tension adjustment, and unmodified direct-line insertion retain their existing precedence.
- `CurveSegmentMoveOptions` combines the required modifier and dedicated segment-translation stroke/marker color, and feedback resolves cleanly across modifier release, pointer exit, release, focus loss, and consecutive gestures.
- Segment translation now clamps one shared x/y delta for both endpoints, preserving pair offset and slope at normalized y bounds, neighbor/minimum-spacing limits, fixed endpoint x constraints, and coupled endpoint y constraints.
- Focused coverage includes modifier-gated hover/color, insertion suppression, direct-line/point/empty-canvas precedence, legacy defaults, translation, commit/cancel/consecutive gestures, feedback clearing, and all group-clamp boundaries.
- Validation passes with `VST3_SDK_DIR=/Users/portalsurfer/lib/vst3sdk`: 295 Patchbay GUI tests, all-target/all-feature clippy, `bash scripts/ci_local.sh` (107 VST3-feature tests plus external API coverage), and the legacy exhaustive-literal integration test under the GUI feature.
- OPT-1169 is signed off and complete in Toybox through PR #6 at `https://github.com/PORTALSURFER/toybox/pull/6`; no further Toybox implementation remains before Pump adoption.
- The current-head review fix cancels a modifier-gated `MoveSegment` before mutation when Command is released or the pointer leaves the editor/window; regression coverage proves the model and changed response remain untouched, and the 295-test GUI suite plus full local CI pass.
- `CurveEditorModifier` is re-exported through both `patchbay_gui` and `toybox::gui::declarative`; an external integration test compiles and names `Command` through both supported downstream APIs.
- `CurveInteractionOptions`, `CurveEditorStyle`, and `Node` retain their legacy public shapes. Modifier and highlight settings travel through the existing opaque `SlotSpec` wrapper, with regression coverage for external exhaustive literals, fluent builder ordering, and declarative render dispatch.
- `row_slots` and `column_slots` recognize the opaque segment-move decorator as a widget-layout proxy, so `weighted_slot(...).width_bounds()/height_bounds()` still reach the wrapped curve editor regardless of builder ordering.
- Active user-requested task: implement OPT-1159, the reusable realtime-safe sample-offset event timeline for CLAP and VST3.
- Branch `wsvasek/opt-1159-toybox-provide-a-realtime-safe-sample-offset-event-timeline` adds a format-neutral fixed-capacity `BlockEventTimeline<P, E>`, CLAP classifier ingestion, and VST3 parameter-queue plus `IEventList` ingestion.
- Timeline ordering is deterministic by clamped sample offset, parameter-before-event priority, and stable source sequence. Full capacity retains the earliest events and explicitly reports replacements and drops without growing storage.
- Regression coverage includes before/at/after-note points, repeated points for one parameter, unsorted and null VST3 queues, CLAP/VST3 parity, zero capacity, final state at the inclusive block boundary, required block sizes `1/16/64/512/2048`, and thread-local allocation/deallocation auditing.
- Validation passes with `VST3_SDK_DIR=/Users/portalsurfer/lib/vst3sdk`: focused event tests, all-features clippy, `bash scripts/ci_local.sh` (107 VST3-feature tests plus external API coverage), and `cargo test --all` (including 285 Patchbay GUI tests).
- Commit `fad729cc35a39e5bca160906b83f79c52d68ea37` is pushed and ready-for-review PR #5 is open at `https://github.com/PORTALSURFER/toybox/pull/5`; current status is `waiting for user review` while GitHub checks run.
- OPT-1148 is signed off and complete. PR #4 adds `InstanceConnection<T>`, processor/controller roles, and the `impl_vst3_instance_connection!` delegation macro.
- The exact host-connected processor publishes an owned `Arc<T>` reference through the standard VST3 `IConnectionPoint::notify(IMessage*)` channel; only the matching controller adopts it, with no process-global creation-order registry and no retained COM peer cycle.
- Shared-state handles use `TypeId` rather than diagnostic type-name strings, so duplicate crate/type paths cannot make distinct concrete types compatible.
- Message attributes own exported handles for the synchronous transfer lifetime; receivers borrow handles to clone compatible state, so rejected offers release the exported `Arc` exactly once.
- Focused tests cover reversed creation order, two independent simultaneous pairs, either callback direction, a host proxy that exposes no Toybox private interface, exact `kNoInterface` bridge-query semantics, concrete-type mismatch rejection, rejected-offer ownership, incompatible processor-to-processor connections, and 128 disconnect/destroy/reconnect cycles.
- Validation passes with `VST3_SDK_DIR=/Users/portalsurfer/lib/vst3sdk`: focused connection tests, VST3 clippy, `scripts/run_agent_request.sh`, and `scripts/ci_local.sh` (97 VST3-feature tests).
- Toybox owns the reusable macOS VST3 hosted-view lifecycle while Radiant owns all GUI rendering; Pump consumes the shared host path instead of a plugin-local Cocoa renderer.
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
- Ioskeley Mono is the default Patchbay/Radiant vector text font and is available to downstream plugins through Toybox.
- Branch `codex/radiant-ioskeley-default-font` vendors Ioskeley Mono v2.0.0 `Normal/Unhinted/IoskeleyMono-Regular.ttf` under `assets/IoskeleyMono/` with OFL text and source notes.
- `patchbay-gui` now prefers bundled Ioskeley Mono before the existing Sometype Mono fallback chain, while `PATCHBAY_GUI_FONT_PATH` still overrides bundled candidates.
- Focused validation passed: `cargo fmt --all -- --check`, `cargo test -p patchbay-gui bundled_font_candidates_prefer_ioskeley_mono -- --nocapture`, `VST3_SDK_DIR=/Users/portalsurfer/lib/vst3sdk cargo clippy --all-targets --all-features -- -D warnings`, and `VST3_SDK_DIR=/Users/portalsurfer/lib/vst3sdk bash scripts/ci_local.sh`.
- `VST3_SDK_DIR=/Users/portalsurfer/lib/vst3sdk cargo test --all` currently passes, including `stress_deeply_nested_panel_tree_measures_without_failure`.
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

- Carry the completed OPT-1169 Toybox curve-editor contract into Pump OPT-1118.

## Immediate Next Actions

1. Let Pump OPT-1118 repin Toybox and opt in with `.curve_segment_move(CurveSegmentMoveOptions::new(CurveEditorModifier::Command, color))`.
2. Keep future shared curve-editor changes source-compatible for legacy `CurveInteractionOptions`, `CurveEditorStyle`, and `Node` consumers.

## Constraints And Notes

- VST3 checks remain opt-in and require `VST3_SDK_DIR`.
- OPT-1148 plugin migration adds `IConnectionPoint` and `IToyboxSharedState` to processor/controller interface tuples, gives each object an `InstanceConnection<Shared>`, invokes `impl_vst3_instance_connection!`, and reads controller state through `connection.shared()`.
- Keep reusable framework behavior in `toybox`; keep plugin-specific behavior in plugin repositories.
