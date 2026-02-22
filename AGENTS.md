# AGENTS

This file is a stateless wake-up portal. Keep it short and explicit.

## 60-Second Wake-Up

1. Run preflight: `bash scripts/run_agent_request.sh`.
2. Read active mission and immediate queue: `docs/plans/active/todo.md`.
3. Read current repository snapshot: `MEMORY.md`.
4. Use plan locations: `docs/plans/index.md`.
5. Before handoff, run `bash scripts/ci_local.sh`, then update `MEMORY.md` and `docs/plans/active/todo.md`.

## Source Of Truth

- Active queue: `docs/plans/active/todo.md`
- Plan map: `docs/plans/index.md`
- Current state snapshot: `MEMORY.md`
- Project constraints and goals: `docs/PROJECT.md`
- Broader documentation: `docs/README.md`

## Guardrails

- Keep this file as a portal, not a knowledge base.
- Put detailed plans and implementation notes under `docs/`.
- Remove stale instructions instead of appending exceptions.
