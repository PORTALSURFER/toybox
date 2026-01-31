# Project Description

## Summary
- Build an internal CLAP plugin framework library that extracts reusable DSP, GUI, parameter/state, and host-integration patterns from existing test plugins to speed up new plugin creation.
- The framework targets the current in-repo plugin style (clack + egui_baseview) and focuses on reusable building blocks, not a full product SDK.

## Goals
- Provide reusable DSP utilities (filters, delay lines, smoothing, spectral/STFT helpers) derived from current plugins.
- Provide reusable GUI building blocks (baseview/egui window wrapper, common controls, visualization helpers, snapshot readers).
- Standardize parameter/state plumbing (param definitions, snapshots, state serialization, event handling) across plugins.
- Reduce time to create a new CLAP plugin by providing templates and example wiring.

## Non-goals
- VST3 support in the framework (CLAP only for v1).
- A public SDK or external distribution.
- Replacing all existing plugin code in one pass.

## Scope (v1)
- New framework crate design and plan (planning-only here), with a clear module layout and responsibilities.
- Inventory of reusable components from existing CLAP plugins: Lilt, Cellweave, RD Field Filter, Polyphonic Reverb.
- Guidance for how new plugins should consume the framework (API shape, examples, conventions).

## Users and Stakeholders
- Primary users: internal developers authoring new CLAP plugins in this repo.
- Secondary stakeholders: maintainers of the current test plugins.

## Constraints
- Follow patterns proven in current test plugins (clack, egui_baseview, baseview, rustfft, no-alloc in audio path).
- Preserve realtime safety and avoid allocations in the audio callback.
- Keep dependencies minimal and consistent with existing crates.
- Maintainability over cleverness; documentation required for public APIs.

## Success Metrics
- New plugin skeleton can be created with <1 day of work using the framework.
- At least 3 shared DSP/GUI/param components are reused by multiple plugins.
- Reduced duplicate code in test plugins when migrated (measured by deleted LOC or extracted modules).

## Assumptions
- CLAP remains the only target format for the framework in v1.
- Egui + baseview remains the default UI stack.
- Existing plugin behavior stays the reference for feature parity.

## Dependencies
- clack/clack_extensions and the existing CLAP integration patterns in this repo.
- egui_baseview + baseview for GUI.
- rustfft for spectral processing in plugins that need it.
