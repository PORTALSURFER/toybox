# Design Principles

## Principle 1: Realtime Safety First
- Statement: All framework utilities used in audio callbacks must avoid heap allocation and blocking.
- Rationale: The current plugins are written to be realtime-safe; the framework must preserve this baseline.
- Constraints: Some utilities (e.g., FFT planning, GUI buffers) must be initialized outside audio processing.

## Principle 2: Extract Only Proven Patterns
- Statement: Framework APIs should represent patterns already validated in existing plugins.
- Rationale: Prevents over-generalization and keeps the framework easy to learn.
- Constraints: New abstractions must be justified by at least two plugins using the same pattern.

## Principle 3: Thin, Composable Modules
- Statement: Prefer small, focused helpers over large super-classes or monoliths.
- Rationale: Plugin DSP and UI vary widely; composable helpers keep flexibility.
- Constraints: Avoid deep inheritance-style abstractions that obscure data flow.
