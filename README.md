# Toybox

`toybox` is a reusable Rust framework for audio-plugin GUI and host integration.

## Quick Start

1. Run the local preflight:
   - `bash scripts/run_agent_request.sh`
2. Read the project context:
   - `docs/PROJECT.md`
   - `docs/plans/index.md`
3. Run local validation before submitting changes:
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
- For project documentation and planning, see `docs/README.md`.
