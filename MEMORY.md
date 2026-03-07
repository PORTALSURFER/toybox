# MEMORY

Last Updated (UTC): 2026-03-07 10:39:00Z

## Current State

- Handoff preflight runs through `bash scripts/run_agent_request.sh`.
- Local validation runs through `bash scripts/ci_local.sh`.
- The EQ attractor surface now renders each attractor as a single filled color circle on the visual/vector draw path.
- A regression test covers vector command ordering for EQ attractor node circles.
- Local cargo validation is currently blocked by the private `mts-esp-rs` fetch failing during workspace dependency resolution.

## Active Mission

- Keep toybox ready for framework iteration while preserving simple, readable attractor styling and correct vector/canvas layering.

## Immediate Next Actions

1. Keep `docs/plans/active/todo.md` as a short ordered queue for the next implementation cycle.
2. Keep `AGENTS.md`, `MEMORY.md`, and `docs/plans/*` aligned whenever mission or queue changes.
3. Restore local Cargo access to the pinned `mts-esp-rs` dependency so `bash scripts/ci_local.sh` can run cleanly again.

## Constraints And Notes

- VST3 checks remain opt-in and require `VST3_SDK_DIR`.
- Keep reusable framework behavior in `toybox`; keep plugin-specific behavior in plugin repositories.
