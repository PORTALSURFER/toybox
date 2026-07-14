# Active Todo Queue

Mission: deliver OPT-1173 as an opt-in reusable Shift horizontal constraint for Patchbay curve-point dragging.

## Active PR

- Branch: `wsvasek/opt-1173-toybox-add-opt-in-shift-horizontal-constraint-for-curve`
- PR: intended ready-for-review PR for OPT-1173
- Scope: add a source-compatible declarative point-horizontal-constraint decorator and keep its anchor/rebase state inside `CurveEditorDragMode::MovePoint`
- Definition of Done: opt-in Shift constraint; stable engage/release transitions; unchanged unconfigured behavior; Command x-snap composition; ordering, spacing, sticky removal, endpoints, release/focus cleanup, and consecutive-gesture coverage
- Status: `validated`

## Immediate Queue

1. Commit and push the validated OPT-1173 implementation.
2. Open a ready-for-review PR and stop for explicit user review/sign-off.
3. After merge, let Pump OPT-1116 repin Toybox and adopt `.curve_point_horizontal_constraint(CurveEditorModifier::Shift)`.

## Validation Note

- `cargo test -p patchbay-gui`: 303 passed, including six focused constraint tests and declarative render coverage.
- `VST3_SDK_DIR=/Users/portalsurfer/lib/vst3sdk cargo test -p toybox --features radiant-vst3 radiant_host_macos::tests`: 17 platform-input/host tests passed.
- `VST3_SDK_DIR=/Users/portalsurfer/lib/vst3sdk bash scripts/ci_local.sh`: passed, including warnings-denied GUI/VST3 clippy and 116 VST3-feature tests.
- `VST3_SDK_DIR=/Users/portalsurfer/lib/vst3sdk cargo test --features gui --test gui_public_api`: 4 passed.
