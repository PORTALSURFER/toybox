# Project Description

## Summary
- Build an internal CLAP plugin framework library that extracts reusable DSP, GUI, parameter/state, and host-integration patterns from existing test plugins to speed up new plugin creation.
- Include CLAP entry/descriptor/factory/ABI glue, parameter metadata + value/text + automation plumbing, event handling helpers, registration macros, and Windows-focused `.clap` bundle packaging guidance.
- The framework targets the current in-repo plugin style (clack + Patchbay GUI) and focuses on reusable building blocks, not a full product SDK.

## Goals
- Provide reusable DSP utilities (filters, delay lines, smoothing, spectral/STFT helpers) derived from current plugins.
- Provide reusable GUI building blocks (Patchbay GUI window wrapper, common controls, visualization helpers, snapshot readers).
- Standardize parameter/state plumbing (param definitions, snapshots, state serialization, event handling) across plugins.
- Reduce time to create a new CLAP plugin by providing templates and example wiring.
- Enable a Lilt-style effect to depend only on `toybox` for CLAP-specific wiring (entry points, params, events, packaging).

## Non-goals
- VST3 support in the framework (CLAP only for v1).
- A public SDK or external distribution.
- Replacing all existing plugin code in one pass.

## Scope (v1)
- New framework crate design and plan (planning-only here), with a clear module layout and responsibilities.
- Inventory of reusable components from existing CLAP plugins: Lilt, Cellweave, RD Field Filter, Polyphonic Reverb.
- Guidance for how new plugins should consume the framework (API shape, examples, conventions).
- CLAP plugin framework coverage: entry points, descriptor/factory, ABI glue, params/state/value↔text/automation, event processing, registration macros, and Windows `.clap` bundle layout.

## Users and Stakeholders
- Primary users: internal developers authoring new CLAP plugins in this repo.
- Secondary stakeholders: maintainers of the current test plugins.

## Constraints
- Follow patterns proven in current test plugins (clack, Patchbay GUI, rustfft, no-alloc in audio path).
- Preserve realtime safety and avoid allocations in the audio callback.
- Keep dependencies minimal and consistent with existing crates.
- Maintainability over cleverness; documentation required for public APIs.

## Success Metrics
- New plugin skeleton can be created with <1 day of work using the framework.
- At least 3 shared DSP/GUI/param components are reused by multiple plugins.
- Reduced duplicate code in test plugins when migrated (measured by deleted LOC or extracted modules).

## Assumptions
- CLAP remains the only target format for the framework in v1.
- Patchbay GUI remains the default UI stack.
- Existing plugin behavior stays the reference for feature parity.
- The Lilt CLAP plugin structure remains the baseline for minimal boilerplate expectations.
- The framework should evolve to support VST in a future version without redesigning core abstractions.

## Dependencies
- clack/clack_extensions and the existing CLAP integration patterns in this repo.
- patchbay-gui + wgpu for GUI.
- rustfft for spectral processing in plugins that need it.
- Lilt CLAP reference implementation (used for entry/param/event/bundle patterns).
