# Active Todo Queue

Mission: carry the completed OPT-1169 Toybox contract into Pump without regressing shared curve-editor behavior.

1. Let Pump OPT-1118 repin the merged Toybox revision and call `.curve_segment_move(CurveSegmentMoveOptions::new(CurveEditorModifier::Command, color))`; keep the color choice downstream.
2. Keep the local guardrail path green (`bash scripts/run_agent_request.sh` and `bash scripts/ci_local.sh`) before every handoff.
3. Preserve vector/canvas layering invariants for shared widgets so deferred vector passes cannot paint over interactive handles.
