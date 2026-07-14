# Active Todo Queue

Mission: deliver OPT-1170 as Toybox-owned realtime-safe VST3 runtime and coherent-state handoff infrastructure.

## Active PR

- Branch: `wsvasek/opt-1170-toybox-provide-reusable-realtime-safe-vst3-runtime-and-state`
- PR: intended ready-for-review PR for OPT-1170
- Scope: generic latest-wins runtime publication/adoption/deferred retirement plus a bounded coherent multi-field state-generation gate under `toybox::vst3`
- Definition of Done: no allocation, blocking, or destruction on audio; stale and redundant candidates retire on control; old-or-complete state observations; documented unsafe/lifecycle boundaries; deterministic concurrency, stress, Miri, external adoption, clippy, and test evidence
- Status: `validated`

## Immediate Queue

1. Commit and push the validated OPT-1170 implementation, then open the PR ready for review.
2. Stop at `waiting for user review` until the user explicitly signs off.
3. After merge, let Kickforge OPT-1150 repin Toybox and replace its local raw-pointer handoff while retaining Kickforge-specific runtime construction, sample-rate equality, state shape, reset, and tail policy.

## Validation Note

- Canonical local CI, VST3 warnings-denied clippy/tests, 500 repeated focused runs, and focused Miri pass.
- The issue's workspace `--all-features` commands remain blocked by the unchanged `origin/main` format-string error in `patchbay-gui/src/declarative/render/grid/axis/resolve.rs` under `layout-overflow-warnings`; keep that baseline cleanup outside this single-intent PR.
