# Active Todo Queue

Mission: keep the framework repo handoff-ready while preserving strict local validation and VST3 host compatibility.

1. Bump Pump to this Toybox revision so macOS VST3 builds export lowercase `bundleEntry` / `bundleExit`.
2. Keep the local guardrail path green (`bash scripts/run_agent_request.sh` and `bash scripts/ci_local.sh`) before every handoff.
3. Add or update focused plan files under `docs/plans/active/` before starting non-trivial feature slices.
4. Preserve vector/canvas layering invariants for shared widgets so deferred vector passes cannot paint over interactive handles.
5. Move completed plan notes to `docs/plans/archive/` once their outcomes are reflected in `MEMORY.md` and stable docs.
