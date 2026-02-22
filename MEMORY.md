# MEMORY

Last Updated (UTC): 2026-02-22 09:10:57Z

## Current State

- Handoff preflight runs through `bash scripts/run_agent_request.sh`.
- Local validation runs through `bash scripts/ci_local.sh`.
- Active mission and immediate next actions live in `docs/plans/active/todo.md`.
- Plan locations are indexed in `docs/plans/index.md`.
- Project constraints and framework scope are defined in `docs/PROJECT.md`.

## Active Mission

- Keep toybox ready for framework iteration with clear, stateless handoff context and green local guardrails.

## Immediate Next Actions

1. Keep `docs/plans/active/todo.md` as a short ordered queue for the next implementation cycle.
2. Keep `AGENTS.md`, `MEMORY.md`, and `docs/plans/*` aligned whenever mission or queue changes.
3. Run `bash scripts/ci_local.sh` before handoff and document any failures with direct remediation notes.

## Constraints And Notes

- VST3 checks remain opt-in and require `VST3_SDK_DIR`.
- Keep reusable framework behavior in `toybox`; keep plugin-specific behavior in plugin repositories.
