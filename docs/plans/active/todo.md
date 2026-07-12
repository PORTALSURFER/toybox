# Active Todo Queue

Mission: keep the framework repo handoff-ready while preserving strict local validation and VST3 host compatibility.

1. Keep OPT-1148's identity-safe VST3 connection PR at `waiting for user review`; do not merge or begin a new PR without explicit resolution.
2. After sign-off and merge, let Kickforge repin to the canonical Toybox commit and remove its creation-order registry.
3. Keep the local guardrail path green (`bash scripts/run_agent_request.sh` and `bash scripts/ci_local.sh`) before every handoff.
4. Preserve vector/canvas layering invariants for shared widgets so deferred vector passes cannot paint over interactive handles.
5. Move completed plan notes to `docs/plans/archive/` once their outcomes are reflected in `MEMORY.md` and stable docs.
