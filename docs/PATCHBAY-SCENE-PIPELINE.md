# Patchbay Scene Pipeline

This note documents the shared frame-planning contract introduced for `OPT-100`.

## Shared contract

Live hosted windows and headless screenshot capture now converge on the same
declarative scene pipeline in `patchbay-gui/src/declarative/render/entry/scene_frame.rs`.

That pipeline owns exactly three steps:

1. Build a first-pass `UiSpec` from the host surface input.
2. Resolve the root render plan and remap surface input into design-space input.
3. Build and execute the final scene from that remapped input.

The output of planning is a `PlannedSceneFrame` containing:

- the original surface input,
- the remapped frame input used by the scene,
- the final `UiSpec`,
- the resolved `RootRenderPlan`.

## Ownership boundary

Everything above the backend seam should consume `PlannedSceneFrame` rather than
re-implementing root transform or pointer remap logic.

- Shared declarative ownership:
  - two-pass spec planning
  - root-transform resolution
  - surface-to-design pointer remapping
  - declarative scene execution
- Backend/runtime ownership:
  - canvas allocation and resize
  - presentation transforms and GPU upload
  - readback / PNG capture
  - platform window lifecycle and reducer dispatch

## Temporary fallback

Headless screenshot rendering still disables vector text and vector shapes while
it uses the CPU canvas execution backend. That fallback is temporary and should
be removed once the Radiant-backed headless path can consume the same planned
scene contract directly (`OPT-90` and `OPT-91`).

At that point, `screenshot.rs` should remain only as a thin backend adapter or
disappear entirely if capture moves behind the renderer seam.
