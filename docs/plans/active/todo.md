# Active Todo Queue

Mission: deliver OPT-1176 so Pump can opt into shared Shift+Option vertical curve-point dragging without duplicating framework interaction state.

## Active PR

- Branch: `wsvasek/opt-1176-toybox-add-opt-in-shiftoption-vertical-constraint-for-curve`
- PR: `https://github.com/PORTALSURFER/toybox/pull/11`
- Pass-1 reviewed head: `9de2f7436bbe8934568f59b29b11c557ac64bb17`
- Review-fix code head: `3967799aef3491c9d9b2928bf281f1c76ebe4f45`
- Scope: add an opt-in Shift+Option vertical point constraint through the reusable Patchbay curve editor, compose it with the existing Shift-only horizontal constraint, and preserve legacy consumers
- Definition of Done: stable x anchor from press or mid-drag engagement; smooth release to normal or Shift-only motion; precedence over Command x snapping; boundary, cleanup, public API, and platform input coverage; canonical warnings-denied validation
- Status: `reviewing` after independent pass 1
- Review: 1 pass; PR-001 (stale status sources) and PR-002 (missing vertical focus-loss regression) are resolved in the pushed branch and await fresh complete-diff re-review

## Immediate Queue

1. Run a fresh complete-diff review at the current PR #11 head and wait for GitHub CI.
2. If clean, move to `waiting for user review`.
3. Wait for explicit user review/sign-off before merge or downstream Pump repin.

## Validation Note

- 14 focused point-constraint/decorator tests and 6 external GUI public-API tests pass.
- The real declarative 220x160 render path proves the vertical decorator reaches runtime and emits a gain-only point move while x stays anchored.
- macOS Radiant/VST3 tests prove Shift, Option, and Command are dispatched before pointer movement and preserve Shift+Option to Shift-only transitions.
- `VST3_SDK_DIR=/Users/portalsurfer/lib/vst3sdk bash scripts/ci_local.sh`: passed, including warnings-denied GUI/VST3 clippy and 128 VST3-feature tests.
- `VST3_SDK_DIR=/Users/portalsurfer/lib/vst3sdk cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `VST3_SDK_DIR=/Users/portalsurfer/lib/vst3sdk cargo test --all`: passed, including 311 Patchbay GUI tests, compile-fail API coverage, 128 Toybox VST3-feature tests, and external integration tests.
