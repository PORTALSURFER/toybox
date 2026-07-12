# Active Todo Queue

Mission: land the reusable OPT-1159 sample-offset timeline while preserving strict realtime safety and CLAP/VST3 parity.

1. Open the ready-for-review OPT-1159 PR with the fixed-capacity timeline, CLAP/VST3 adapters, and parity/realtime regression matrix.
2. After explicit sign-off and merge, let Kickforge OPT-1147 repin and keep its adoption plugin-specific.
3. Keep the local guardrail path green (`bash scripts/run_agent_request.sh` and `bash scripts/ci_local.sh`) before every handoff.
4. Preserve vector/canvas layering invariants for shared widgets so deferred vector passes cannot paint over interactive handles.
5. Move completed plan notes to `docs/plans/archive/` once their outcomes are reflected in `MEMORY.md` and stable docs.
