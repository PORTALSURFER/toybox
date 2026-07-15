# Active Todo Queue

Mission: deliver OPT-1176 so Pump can opt into shared Shift+Option vertical curve-point dragging without duplicating framework interaction state.

## Active PR

- Branch: `wsvasek/opt-1176-toybox-add-opt-in-shiftoption-vertical-constraint-for-curve`
- PR: intended ready-for-review PR; not yet published
- Scope: add an opt-in Shift+Option vertical point constraint through the reusable Patchbay curve editor, compose it with the existing Shift-only horizontal constraint, and preserve legacy consumers
- Definition of Done: stable x anchor from press or mid-drag engagement; smooth release to normal or Shift-only motion; precedence over Command x snapping; boundary, cleanup, public API, and platform input coverage; canonical warnings-denied validation
- Status: `validated`

## Immediate Queue

1. Commit, push, and publish the ready OPT-1176 PR.
2. Run the required complete-diff review/fix loop at the exact pushed head.
3. Wait for explicit user review/sign-off before merge or downstream Pump repin.

## Validation Note

- 13 focused point-constraint/decorator tests and 6 external GUI public-API tests pass.
- The real declarative 220x160 render path proves the vertical decorator reaches runtime and emits a gain-only point move while x stays anchored.
- macOS Radiant/VST3 tests prove Shift, Option, and Command are dispatched before pointer movement and preserve Shift+Option to Shift-only transitions.
- `VST3_SDK_DIR=/Users/portalsurfer/lib/vst3sdk bash scripts/ci_local.sh`: passed, including warnings-denied GUI/VST3 clippy and 128 VST3-feature tests.
- `VST3_SDK_DIR=/Users/portalsurfer/lib/vst3sdk cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `VST3_SDK_DIR=/Users/portalsurfer/lib/vst3sdk cargo test --all`: passed, including 310 Patchbay GUI tests, compile-fail API coverage, 128 Toybox VST3-feature tests, and external integration tests.
