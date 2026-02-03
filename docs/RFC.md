# RFC - CLAP Plugin Framework Library

## Status
- Draft

## Summary
- Create an internal CLAP plugin framework crate that consolidates reusable DSP, GUI, parameter/state, event-processing, and CLAP entry/packaging patterns from the current test plugins (with Lilt as the baseline).

## Goals
- Provide a small, well-documented API surface for building CLAP plugins quickly.
- Extract reusable DSP primitives and parameter/GUI scaffolding from Lilt, Cellweave, RD Field Filter, and Polyphonic Reverb.
- Make Lilt-style effects depend on `toybox` for CLAP entry points, param metadata/value↔text, automation handling, event batching, and `.clap` bundle packaging.

## Non-goals
- Support VST3 in the framework.
- Provide a monolithic “one size fits all” plugin class.
- Re-architect existing plugins during the initial framework design.

## Background / Problem
- Current CLAP plugins reimplement the same patterns: audio port setup, event batching, buffer handling, parameter snapshots, tempo sync, Patchbay GUI window handling, and GUI snapshot transport. This slows down new plugin creation and increases divergence across plugins.

## Architecture
### Components
- `framework::clap`: CLAP-specific scaffolding (port helpers, event batching utilities, process-range helpers, transport helpers).
- `framework::entry`: CLAP entry/descriptor/factory/ABI glue to reduce boilerplate (modeled after the Lilt entry export).
- `framework::events`: Full CLAP event handling layer (process context, input events, output events, and helpers for all CLAP event types used by plugins).
- `framework::params`: Parameter ID conventions, atomic storage helpers, snapshot patterns, and serialization helpers.
- `framework::params::clap`: Param metadata, value↔text conversion, and automation/event plumbing aligned with CLAP.
- `framework::dsp`: Reusable DSP blocks (smoothing, delay lines, one-pole filters, biquad helpers, STFT scaffolding, windowing/OLA utilities).
- `framework::gui`: Patchbay GUI window wrapper, resizing/parent handle helpers, common widgets, and visualization primitives.
- `framework::snapshot`: Lock-free snapshot buffers for GUI visualization (ring buffers, atomic frames).
- `framework::templates`: Optional examples and plugin skeletons aligned with existing patterns (internal use only).
- `framework::registration`: Macros/helpers to expose a CLAP plugin with minimal boilerplate.
- `framework::bundle`: Windows `.clap` bundle layout helpers and build guidance (inspired by Lilt’s build setup).

### Data Flow
- Audio thread owns DSP state and reads parameter snapshots; GUI thread reads snapshot buffers (lock-free atomics). The framework provides utilities for safe handoff and standardized event/buffer processing, plus CLAP input/output event adapters.

### Key Interfaces
- `ParamSnapshot` pattern to capture atomic parameter state for audio processing.
- `GuiHostWindow` abstraction that wraps Patchbay GUI sizing, repaint, and resize coordination.
- `SnapshotBuffer` interfaces for audio→GUI visualization data.
- `PluginEntry` helper to emit format-specific descriptor/factory/ABI glue for a plugin type (CLAP first, VST later).
- `EventRouter` (or similar) to normalize CLAP input event batches and provide typed helpers for common event classes.
- `ParamApi` helper for CLAP param metadata, value↔text conversion, and automation event application.
- `ProcessContext` wrapper to standardize audio buffer access, sample bounds, and transport data.

## Decisions (Resolved)
- Target CLAP only for v1.
- Reuse the existing CLAP process pattern (batch events, convert sample bounds to ranges, process in blocks).
- Preserve realtime safety by avoiding heap allocations in audio callbacks.
- Focus `.clap` bundle guidance on Windows first.

## Open Questions
- Which DSP blocks should be stabilized in v1 versus left in plugin-local code?
- How to document migration steps for existing plugins without big-bang changes?
- Should we include a GUI theming layer or keep simple primitives only?
- Should bundle helpers live in a `build.rs` template, an `xtask`, or both?
- Which CLAP event types must be wrapped directly vs. left as raw events for plugin-specific handling?
- What abstractions need to be format-neutral to support future VST without churn?

## Alternatives Considered
- Keep plugin code fully independent: rejected due to duplication and drift.
- Build a high-level plugin superclass: rejected to avoid constraining creative DSP design.

## Risks and Mitigations
- Risk: framework becomes overly generic and hard to use -> keep API surface small and focused on proven patterns.
- Risk: migration churn -> prioritize additive extraction and opt-in usage.
- Risk: GUI abstraction too leaky -> keep GUI helpers thin and allow direct Patchbay usage.

## Testing and Validation
- Unit tests for reusable DSP blocks (e.g., delay line, filters, snapshot read/write).
- Golden tests for any shared processing math (e.g., windowing, smoothing).
- Smoke tests for example plugin skeletons in the repo.

## Rollout / Phases
- Phase 0: audit and plan, define module boundaries.
- Phase 1: extract low-risk DSP + snapshot utilities; add docs + tests.
- Phase 2: add GUI scaffolding + CLAP helpers; provide a new plugin template.
