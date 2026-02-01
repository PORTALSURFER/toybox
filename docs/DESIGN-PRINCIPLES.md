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

## Principle 4: Minimal CLAP Boilerplate
- Statement: A Lilt-style effect should only need `toybox` for CLAP entry points, params, events, and packaging.
- Rationale: Reduces repeated CLAP wiring across plugins and keeps authoring focused on DSP/UI.
- Constraints: Keep CLAP glue centralized and avoid hiding critical data flow.

## Principle 5: Future-Format Friendly
- Statement: Core abstractions should not assume CLAP-specific types when a format-neutral shape is feasible.
- Rationale: Keeps the path open for VST support without redesigning the framework.
- Constraints: Do not compromise CLAP ergonomics in v1.
