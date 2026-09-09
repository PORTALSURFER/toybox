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

## GUI development

Use the `gpui-gui` feature for new plugin editors on macOS and Windows, or
`gpui-vst3` when building VST3. Toybox embeds GPUI in the host's native child
window and leaves the host in charge of the event loop. The default feature
set remains empty for DSP-only consumers.

See [the embedded GPUI guide](docs/GPUI.md) for ownership, input, and lifecycle
requirements. Existing Radiant and Patchbay integrations remain optional during
consumer migration.

## Repository Layout

- `src/`: core framework crates and shared runtime code.
- `patchbay-gui/`: declarative GUI toolkit and rendering/runtime layers.
- `examples/`: minimal CLAP/VST3 reference plugins.
- `docs/`: project constraints, architecture notes, and active plans.
- `scripts/`: local guardrail and CI helper scripts.

## Development Notes

- Keep plugin-specific behavior out of `toybox`; framework behavior should be generic and reusable.
- Use Toybox host adapters and the re-exported `toybox::gpui` types for new editors; keep native host integration inside Toybox.
- For project documentation and planning, see `docs/README.md`.
