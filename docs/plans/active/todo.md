# Active Todo Queue

Mission: keep the framework repo handoff-ready while preserving strict local validation and VST3 host compatibility.

1. Let Kickforge repin to the canonical Toybox commit and remove its creation-order registry.
2. Keep the local guardrail path green (`bash scripts/run_agent_request.sh` and `bash scripts/ci_local.sh`) before every handoff.
3. Preserve vector/canvas layering invariants for shared widgets so deferred vector passes cannot paint over interactive handles.
4. Move completed plan notes to `docs/plans/archive/` once their outcomes are reflected in `MEMORY.md` and stable docs.
