# Toybox

`toybox` is a reusable Rust framework for audio-plugin GUI and host integration.

## Quick Start

1. Run the handoff preflight:
   - `bash scripts/run_agent_request.sh`
2. Read active work context:
   - `docs/plans/active/todo.md`
   - `MEMORY.md`
3. Run local validation before handoff:
   - `bash scripts/ci_local.sh`

## Repository Layout

- `src/`: core framework crates and shared runtime code.
- `patchbay-gui/`: declarative GUI toolkit and rendering/runtime layers.
- `examples/`: minimal CLAP/VST3 reference plugins.
- `docs/`: project constraints, architecture notes, and active plans.
- `scripts/`: local guardrail and CI helper scripts.

## Development Notes

- Keep plugin-specific behavior out of `toybox`; framework behavior should be generic and reusable.
- Prefer `toybox` and `patchbay-gui` APIs over direct host/toolkit coupling in plugins.
- For planning/handoff standards, see `AGENTS.md` and `docs/README.md`.
