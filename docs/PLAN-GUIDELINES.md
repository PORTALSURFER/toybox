# Plan Guidelines

## Goals
- Build a focused CLAP plugin framework that accelerates internal plugin creation.
- Preserve existing plugin behavior by extracting tested patterns and utilities.

## Locked Decisions
- CLAP-only for v1.
- Follow patterns already used in test plugins.
- Realtime-safe audio processing (no allocations or blocking in callbacks).

## Execution Principles
- Extract utilities only after identifying duplication across at least two plugins.
- Keep modules small and single-purpose.
- Document every public API with rationale and usage constraints.

## Testing Strategy
- Unit tests for reusable DSP blocks and snapshot utilities.
- Example-based tests for helpers (e.g., snapshot read/write, delay line).
- Smoke tests for framework templates once introduced.

## Step Template
Use this exact template for each plan step:

- Step: <id + title>
- Scope: <what changes are in / out>
- Files: <exact file paths or components>
- Entry Points: <first functions/modules to edit>
- Example (primary): <example name> (<what it validates>)
- Example (required): <if additional examples are mandatory by phase end>
- Commands:
  - <exact commands to build/run/test>
- Verification:
  - <test names, compare commands, or metric thresholds>
- Constraints:
  - <manual-only, headless flag, platform limits>
- Non-goals:
  - <what not to include>
- Expected Artifacts:
  - <new files, updated files, outputs, reports>
- Acceptance Checklist:
  - <3–5 bullet checks>
- Failure Modes:
  - <common pitfalls and how to detect them>
- Update Docs:
  - <doc updates or “not applicable”>

## Acceptance Metrics
- Reusable utilities are adopted by at least one plugin or template without regressions.
- Public APIs are documented and tests are added for non-trivial logic.

## CLAP Coverage Checklist (when CLAP is in scope)
- Entry points and descriptor/factory/ABI glue defined and documented.
- Parameter metadata, value↔text conversion, and automation/event application covered.
- Audio/MIDI event handling includes full CLAP event set used by plugins.
- Process context helpers cover audio buffers, sample bounds, and transport.
- Registration macros/helpers provide minimal boilerplate path.
- Windows `.clap` bundle layout documented with build guidance.

## Future Format Checklist (when planning for VST)
- Core abstractions named and shaped to avoid CLAP-only type leaks.
- CLAP-specific helpers isolated behind format modules or adapters.
- Registration macros designed to admit additional formats without breaking API.

## Definition of Done
- Each phase delivers a tested, documented framework module with an example usage.

## Known Constraints
- Internal-only; prioritize speed of development over external polish.
- Audio thread must remain allocation-free.

## Glossary
- Framework: the new internal crate providing reusable CLAP plugin utilities.
- Snapshot: lock-free data shared from audio to GUI threads.
- Template: a minimal example plugin wired to the framework.
