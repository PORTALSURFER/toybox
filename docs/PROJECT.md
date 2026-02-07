# Project Description

## Summary
- Build an internal plugin framework library that extracts reusable DSP, GUI, parameter/state, and host-integration patterns from existing test plugins to speed up new plugin creation.
- Provide both CLAP and VST3 integration layers, each with entry/descriptor/factory glue, parameter/state helpers, event processing utilities, and packaging helpers.
- Keep reusable building blocks explicit and realtime-safe while minimizing per-plugin host-format boilerplate.

## Goals
- Provide reusable DSP utilities (filters, delay lines, smoothing, spectral/STFT helpers) derived from current plugins.
- Provide reusable GUI building blocks (Patchbay GUI window wrapper, common controls, visualization helpers, snapshot readers).
- Standardize parameter/state plumbing (param definitions, snapshots, state serialization, event handling) across plugins.
- Provide stable import surfaces for plugin wiring:
  - `toybox::clap::prelude`
  - `toybox::vst3::prelude` (feature-gated with `vst3`)
- Enforce plugin-state support by default in CLAP and provide shared versioned state helpers for VST3.
- Reduce time to create a new CLAP or VST3 plugin by providing templates/examples and shared utilities.

## Non-goals
- A public SDK or external distribution.
- Replacing all existing plugin code in one pass.
- Cross-platform VST3 bundle packaging in the initial rollout (Windows-first helpers are provided).

## Scope (current)
- CLAP framework coverage: entry points, descriptor/factory, ABI glue, params/state/value↔text/automation, event processing, registration macros, and Windows `.clap` bundle layout.
- VST3 framework coverage (feature `vst3`): entrypoint macro, factory/component/controller helper modules, process/params/state/gui utilities, and Windows `.vst3` bundle layout helpers.
- VST3 SDK path provided by the `VST3_SDK_DIR` environment variable.
- Minimal examples:
  - `examples/minimal-clap`
  - `examples/minimal-vst3`

## Users and Stakeholders
- Primary users: internal developers authoring new plugin formats in this repo.
- Secondary stakeholders: maintainers of current plugin crates that progressively adopt shared helpers.

## Constraints
- Preserve realtime safety and avoid allocations in the audio callback.
- Keep dependencies minimal and consistent with existing crates.
- Follow maintainability-first design and document public APIs.
- Keep CLAP users unaffected when VST3 is not enabled.

## Success Metrics
- New plugin skeleton can be created with <1 day of work using toybox.
- Shared DSP/GUI/param components are reused by multiple plugins.
- Reduced duplicate code in plugin crates when migrated.
- VST3 example compiles and exports through toybox entry helpers.

## Assumptions
- Patchbay GUI remains the default UI stack.
- Existing plugin behavior remains the reference for feature parity.
- VST3 support is exposed behind the `vst3` Cargo feature.
- Windows is the primary VST3 packaging target for the first release.

## Dependencies
- CLAP: `clack-plugin`, `clack-extensions`, `clack-common`.
- VST3: `VST3_SDK_DIR` SDK location (validated at build time) plus generated Rust bindings from `vst3`.
- GUI: `patchbay-gui` + `wgpu`.
- DSP/spectral use-cases: `rustfft` in plugin crates that need it.
