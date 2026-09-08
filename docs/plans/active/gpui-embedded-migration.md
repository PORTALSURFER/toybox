# Embedded GPUI migration

## Objective

Replace GainSnap's Radiant GUI with GPUI and make GPUI Toybox's supported GUI
path for new plugin development. Preserve GainSnap's compact appearance, audio
processing, parameter IDs, persisted state, automation, and macOS/Windows CLAP
and VST3 behavior. User-reported broken host text input is an acceptance gate.

## Architecture

- Toybox owns native child views, host lifecycle, GPUI platform/window adapters,
  rendering, input delivery, and format adapters. GainSnap owns its GPUI view
  and parameter interaction model.
- Use GPUI's externally driven `run_embedded`/`ApplicationHandle` API. Never
  start an application event loop or replace the DAW's AppKit delegate.
- Use GPUI core and its WGPU renderer without linking GPUI's standalone native
  application platforms. Native classes and callbacks must be safe when two
  independently linked plugin libraries are loaded in one host process.
- On macOS create an explicitly owned CAMetalLayer surface; do not rely on
  raw-window-metal's process-global view/layer registration. Tear the renderer
  down before releasing its layer or view.
- UI work stays on the host UI thread. No GPUI work, allocation, blocking, or
  callbacks are introduced on the audio thread.
- Serialize GPUI updates on the host UI thread: nested native events queue
  until the outer update finishes. Callback slots use generations so callbacks
  replaced during dispatch cannot be restored over their replacements.
- Native callbacks upgrade a weak window-state token and retain the native view
  for the full call. Closing immediately marks state unavailable; destruction
  waits until active callbacks release ownership.
- VST3 callbacks never block on a reentrant GUI mutex. A reentrant removal
  records terminal pending teardown, then closes once the outer call releases
  the GUI; other contended calls report failure.
- Preserve the optional Radiant facade for existing consumers during migration.
  GainSnap's default dependency graph must contain no Radiant.
- Pin all remote dependencies to exact commits. Any local path dependencies are
  temporary development aids and must be replaced before committing.

The initial GPUI dependency is `PORTALSURFER/gpui-embedded` commit
`77a1325a13bd0d3631f737b98445b70c8c936cb7`, based on Zed's embedded-lifecycle
commit `7bc1c05d8b30db2b5bd1855847ba5e3b5899863a`. It exposes
`WgpuRenderer::new_with_surface_target` and disambiguates the GPUI workspace
dependency from an unrelated lint-test fixture. It also enables native Metal
and DirectX 12 backends, and provides offscreen RGBA capture using the same
scene rendering path for visual tests. Both GPUI crates must use the same
repository and revision to keep their Rust types identical. Source review is
tracked in https://github.com/PORTALSURFER/gpui-embedded/pull/1.

## Visual reference

The previous GainSnap UI was rendered from the merged RMS/keyboard source into
`/Users/portalsurfer/dev/audiodev/dist/gainsnap-gpui-reference/gainsnap/`.
Compare idle, active bright, active dim, and RMS states at 208 by 212 logical
pixels. Preserve the existing dimensions, dark colors, monospaced typography,
orange output meter, target marker, Normalize label, and synchronized pulse.

## Work and acceptance

1. Prove native GPUI rendering and input in a host-owned child window, without
   application-loop takeover; implement both NSView and HWND paths.
2. Port the GainSnap interface and host gestures. Preserve parameter/state/DSP
   contracts and visibility auto-stop without stopping on focus loss.
3. Test real GPUI input: numeric edits, selection, paste, caret movement,
   Enter/Escape, 1 dB arrows and 0.1 dB Shift-arrows while meters update.
4. Verify native attach, resize/DPI, hide/minimize, close/reopen, callback
   containment, and two independently linked plugin libraries in both orders.
5. Run applicable formatting, lint, unit, VST3 SDK, Windows CI, screenshot,
   packaging, signature and symbol checks. Build fresh root-dist artifacts.
6. Publish reviewable dependency/plugin PRs and report the exact tested build.
   Manual DAW keyboard and audible acceptance remain the user's final check.

## Status

The backend passes macOS, Windows and Linux CI, including strict lint and
platform tests. Native macOS VST3 probes pass target editing, clipboard input,
resize/reopen, visibility auto-stop and both independently linked plugin load
orders. GainSnap's four live captures preserve the compact visual design.
Fresh macOS CLAP/VST3 bundles pass packaging, signature and symbol audits.

Final acceptance is pending for native button keyboard activation and the
remaining GainSnap Windows runtime and release-contract checks. Rebuild and
audit the final plugin revision after these fixes. Manual DAW keyboard and
audible acceptance remain the user's final check. No release is authorized by
this plan.
