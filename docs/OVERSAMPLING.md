# Realtime 2x/4x Oversampling

Toybox exposes fixed-factor mono oversampling under `toybox::dsp`. It owns the
interpolation and decimation filters, inline state, reset behavior, latency,
dry-path alignment, and reusable measurements. Plugins retain stage selection,
quality policy, nonlinear functions, oscillators, and sound-design decisions.

## API

- `HalfBandInterpolator2x` and `HalfBandDecimator2x` are the reusable stages.
- `MonoOversampler2x` and `MonoOversampler4x` wrap one input sample around an
  inline generic callback invoked exactly 2 or 4 times.
- `MonoOversampler` selects an explicit `OversamplingFactor` at construction and
  keeps callback dispatch static.
- `SourceDecimator2x` and `SourceDecimator4x` accept caller-generated arrays of
  chronological high-rate samples.
- `DryPathAligner` delays a dry path by the selected round-trip processing
  latency. Fractional samples use first-order linear interpolation.

The [selective downstream example](../examples/selective_oversampling.rs) shows
a drive callback and a high-rate folded source without oversampling the entire
voice, while an intentionally aliased crusher remains direct.

## Filter response

Every stage uses the same deterministic 111-tap, linear-phase half-band FIR.
It is a Kaiser-windowed ideal half-band (`beta = 8.6`) with exact symmetry,
structural zero taps, an exact 0.5 center tap, and unity DC normalization. Only
the 56 nonzero even-phase coefficients and the center tap are evaluated.

Frequencies below are relative to the base-rate Nyquist frequency:

| Region | Documented response |
|---|---:|
| DC | 0 dB |
| Passband, 0-90% | < 0.001 dB peak-to-peak ripple |
| Transition midpoint, 100% | -6.0206 dB |
| Stopband, 110-200% | <= -85 dB |

The 90-110% transition is intentional: no causal finite filter can both remain
flat through base-rate Nyquist and reject immediately above it. Downstream
quality decisions should account for content deliberately placed in that band.

The linear-phase FIR preserves DC and low fundamentals without phase warping;
it adds only the reported constant group delay. Coefficients are embedded `f32`
constants, so construction does not depend on platform math libraries or host
sample rate. Tests cover finite, bounded processing at 44.1, 48, 96, and 192
kHz and spectral alias reduction for hard clip, foldback, and FM-like sources.

## Exact latency

`SampleDelay` reports an exact reduced rational number of base-rate samples.

| Path | 2x | 4x |
|---|---:|---:|
| Interpolation only | 55/2 (27.5) | 165/4 (41.25) |
| Source decimation only | 55/2 (27.5) | 165/4 (41.25) |
| Input -> callback -> output | 55 | 165/2 (82.5) |

For 4x, the second 2x stage runs at twice the first stage's rate, which is why
its base-rate delay contribution is half as large. Host latency reporting that
accepts only integers must choose policy downstream; `DryPathAligner` retains
the exact fractional delay internally. Its linear interpolation has unity DC
gain and exact group delay at DC, with increasing magnitude droop toward
Nyquist for the half-sample 4x alignment case.

## Realtime contract

All audio state is fixed-size and stored inline. After construction,
`process` and `reset` do not allocate or deallocate, lock, log, panic, or use
trait-object dispatch. The fixed-factor wrapper performs one bounded enum match
per base-rate sample. A thread-local allocator audit covers every public audio
operation and reset path.

## CPU scaling and benchmark

The half-band polyphase layout evaluates 56 multiply-accumulates in its
nontrivial phase instead of 111 taps at every high-rate phase. Cost is linear
in instance count and approximately linear in the number of composed 2x stage
operations: 4x performs the first 2x stage plus a second stage at twice the
rate. Block size does not change DSP results and has little effect on this
sample-oriented API beyond caller loop overhead.

Run the required matrix in release mode:

```sh
cargo bench --bench oversampling
```

It reports base-rate, 2x, and 4x nanoseconds per base-rate sample/instance plus
relative cost for block sizes 1, 16, 64, 256, and 1024 across 1, 16, and 64
instances. Results are machine- and compiler-specific; compare rows from the
same run rather than treating one machine's absolute timing as a guarantee.

Reference run on 2026-07-15, Apple M5 Pro arm64, Rust 1.96.0, optimized bench
profile:

| Block | Instances | Base ns/sample | 2x ns/sample | 4x ns/sample |
|---:|---:|---:|---:|---:|
| 1 | 1 | 1.89 | 53.34 | 184.08 |
| 1 | 16 | 1.82 | 51.90 | 163.88 |
| 1 | 64 | 1.80 | 51.63 | 163.39 |
| 16 | 1 | 1.79 | 51.45 | 165.10 |
| 16 | 16 | 1.86 | 51.83 | 164.73 |
| 16 | 64 | 1.86 | 51.07 | 165.13 |
| 64 | 1 | 1.86 | 51.06 | 165.69 |
| 64 | 16 | 1.80 | 50.93 | 165.15 |
| 64 | 64 | 1.84 | 51.17 | 165.31 |
| 256 | 1 | 1.78 | 51.13 | 169.34 |
| 256 | 16 | 1.93 | 52.54 | 174.10 |
| 256 | 64 | 1.83 | 51.53 | 169.31 |
| 1024 | 1 | 1.86 | 52.78 | 171.64 |
| 1024 | 16 | 1.82 | 51.55 | 165.97 |
| 1024 | 64 | 1.87 | 52.07 | 174.91 |

The absolute base-rate loop is small enough for aggressive compiler
vectorization, so the ratio columns can look disproportionately large. The
per-sample nanosecond columns are the useful capacity-planning numbers.
