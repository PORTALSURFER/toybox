# RFC - CLAP Plugin Framework Library

## Status
- Draft

## Summary
- Create an internal CLAP plugin framework crate that consolidates reusable DSP, GUI, parameter/state, and event-processing patterns from the current test plugins.

## Goals
- Provide a small, well-documented API surface for building CLAP plugins quickly.
- Extract reusable DSP primitives and parameter/GUI scaffolding from Lilt, Cellweave, RD Field Filter, and Polyphonic Reverb.

## Non-goals
- Support VST3 in the framework.
- Provide a monolithic “one size fits all” plugin class.
- Re-architect existing plugins during the initial framework design.

## Background / Problem
- Current CLAP plugins reimplement the same patterns: audio port setup, event batching, buffer handling, parameter snapshots, tempo sync, baseview/egui window handling, and GUI snapshot transport. This slows down new plugin creation and increases divergence across plugins.

## Architecture
### Components
- `framework::clap`: CLAP-specific scaffolding (port helpers, event batching utilities, process-range helpers, transport helpers).
- `framework::params`: Parameter ID conventions, atomic storage helpers, snapshot patterns, and serialization helpers.
- `framework::dsp`: Reusable DSP blocks (smoothing, delay lines, one-pole filters, biquad helpers, STFT scaffolding, windowing/OLA utilities).
- `framework::gui`: Baseview/egui window wrapper, resizing/parent handle helpers, common widgets, and visualization primitives.
- `framework::snapshot`: Lock-free snapshot buffers for GUI visualization (ring buffers, atomic frames).
- `framework::templates`: Optional examples and plugin skeletons aligned with existing patterns (internal use only).

### Data Flow
- Audio thread owns DSP state and reads parameter snapshots; GUI thread reads snapshot buffers (lock-free atomics). The framework provides utilities for safe handoff and standardized event/buffer processing.

### Key Interfaces
- `ParamSnapshot` pattern to capture atomic parameter state for audio processing.
- `GuiHostWindow` abstraction that wraps baseview/egui sizing, repaint, and resize coordination.
- `SnapshotBuffer` interfaces for audio→GUI visualization data.

## Decisions (Resolved)
- Target CLAP only for v1.
- Reuse the existing CLAP process pattern (batch events, convert sample bounds to ranges, process in blocks).
- Preserve realtime safety by avoiding heap allocations in audio callbacks.

## Open Questions
- Which DSP blocks should be stabilized in v1 versus left in plugin-local code?
- How to document migration steps for existing plugins without big-bang changes?
- Should we include a GUI theming layer or keep simple primitives only?

## Alternatives Considered
- Keep plugin code fully independent: rejected due to duplication and drift.
- Build a high-level plugin superclass: rejected to avoid constraining creative DSP design.

## Risks and Mitigations
- Risk: framework becomes overly generic and hard to use -> keep API surface small and focused on proven patterns.
- Risk: migration churn -> prioritize additive extraction and opt-in usage.
- Risk: GUI abstraction too leaky -> keep GUI helpers thin and allow direct egui usage.

## Testing and Validation
- Unit tests for reusable DSP blocks (e.g., delay line, filters, snapshot read/write).
- Golden tests for any shared processing math (e.g., windowing, smoothing).
- Smoke tests for example plugin skeletons in the repo.

## Rollout / Phases
- Phase 0: audit and plan, define module boundaries.
- Phase 1: extract low-risk DSP + snapshot utilities; add docs + tests.
- Phase 2: add GUI scaffolding + CLAP helpers; provide a new plugin template.
