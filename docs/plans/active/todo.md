# Active Todo Queue

Mission: land OPT-1169's reusable modifier-gated grouped curve-segment interaction without changing legacy consumers.

1. Wait for explicit user review/sign-off on ready-for-review Toybox PR #6 for OPT-1169, including the current-head gate-release/pointer-exit cancellation fix.
2. After sign-off, merge the PR and complete the branch/working-tree cleanup procedure.
3. Let Pump OPT-1118 repin the merged Toybox revision and opt into `CurveEditorModifier::Command`; keep Pump-specific style choices downstream.
4. Keep the local guardrail path green (`bash scripts/run_agent_request.sh` and `bash scripts/ci_local.sh`) before every handoff.
5. Preserve vector/canvas layering invariants for shared widgets so deferred vector passes cannot paint over interactive handles.
