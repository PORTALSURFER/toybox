# Active Todo Queue

Mission: hand the merged OPT-1170 Toybox runtime/state contract to Kickforge OPT-1150.

## Completed PR

- Branch: `wsvasek/opt-1170-toybox-provide-reusable-realtime-safe-vst3-runtime-and-state`
- PR: `https://github.com/PORTALSURFER/toybox/pull/7`
- Scope: generic latest-wins runtime publication/adoption/deferred retirement plus bounded owned snapshot-and-generation state publication under `toybox::vst3`
- Definition of Done: no allocation, blocking, or destruction on audio; stale and redundant candidates retire on control; old-or-complete state observations; documented unsafe/lifecycle boundaries; deterministic concurrency, stress, Miri, external adoption, clippy, and test evidence
- Status: `signed off`; PR #7 is the complete Toybox delivery for OPT-1170

## Immediate Queue

1. Merge PR #7 and complete branch cleanup; no further Toybox implementation remains for OPT-1170.
2. Let Kickforge OPT-1150 repin Toybox and replace its local raw-pointer handoff while retaining Kickforge-specific runtime construction, sample-rate equality, state shape, reset, and tail policy.

## Validation Note

- Canonical local CI, VST3 warnings-denied clippy/tests, 500 repeated focused runs, and focused Miri pass.
- The issue's workspace `--all-features` commands remain blocked by the unchanged `origin/main` format-string error in `patchbay-gui/src/declarative/render/grid/axis/resolve.rs` under `layout-overflow-warnings`; keep that baseline cleanup outside this single-intent PR.
