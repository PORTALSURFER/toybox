# MEMORY

Last Updated (UTC): 2026-03-29 14:42:26Z

## Current State

- Handoff preflight runs through `bash scripts/run_agent_request.sh`.
- Local validation runs through `bash scripts/ci_local.sh`.
- The EQ attractor surface now renders each attractor as a single filled color circle on the visual/vector draw path.
- The EQ attractor curve now emits a filled polygon plus joined vector polylines, removing the old comb-like fill lines and preserving smoother subpixel rendering.
- The EQ attractor surface now uses a shared gravity-field wave model, so attractors bend and locally slow one curve instead of layering separate local sine waves.
- The EQ attractor surface now requires real pointer travel before emitting a move, so an off-center click selects a node without nudging it.
- Regression tests cover both node ordering and the curve command shape emitted in vector mode.
- The curve editor now supports declarative beat-guide overlays, configurable snap targets, and held shortcut-key state so plugins can render brighter grids and temporarily invert snapping while a key is held.
- Win32 shortcut handling now maps Ctrl-letter `WM_CHAR` control codes back to their ASCII letters for shortcut matching, and matched Ctrl shortcuts are swallowed while text edit is active instead of leaking characters into text boxes.
- Local cargo validation is currently blocked by the private `mts-esp-rs` fetch failing during workspace dependency resolution.

## Active Mission

- Keep toybox ready for framework iteration while preserving readable attractor styling, smooth curve rendering, and reusable editor/input primitives for plugin UIs.

## Immediate Next Actions

1. Land the paired Pump changes that consume the updated Ctrl-shortcut handling on `main`.
2. Keep `AGENTS.md`, `MEMORY.md`, and `docs/plans/*` aligned whenever mission or queue changes.
3. Restore local Cargo access to the pinned `mts-esp-rs` dependency so `bash scripts/ci_local.sh` can run cleanly again.

## Constraints And Notes

- VST3 checks remain opt-in and require `VST3_SDK_DIR`.
- Keep reusable framework behavior in `toybox`; keep plugin-specific behavior in plugin repositories.
