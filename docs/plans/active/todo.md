# Active Todo Queue

Mission: deliver OPT-1174 as reusable realtime-safe fixed 2x/4x mono audio oversampling infrastructure for downstream plugins.

## Active PR

- Branch: `wsvasek/opt-1174-toybox-provide-reusable-realtime-2x4x-audio-oversampling`
- PR: `https://github.com/PORTALSURFER/toybox/pull/9`
- Scope: deterministic 2x half-band interpolation/decimation stages, composable 4x input-processing and source-generation paths, exact rational latency, dry alignment, reset behavior, spectral/realtime validation, benchmark coverage, and a selective downstream example
- Definition of Done: reusable 2x/4x stages and callbacks; source decimation; no post-construction audio-thread allocation/locking/logging/panic/dynamic dispatch; documented response and latency; alias, stability, reset, equivalence, and allocator coverage; required CPU matrix; full Toybox validation
- Status: `waiting for user review`

## Immediate Queue

1. Wait for explicit user review/sign-off on ready-for-review PR #9.
2. After merge, let Kickforge OPT-1152 repin Toybox and adopt the primitives with Kickforge-owned stage policy.

## Validation Note

- The 111-tap deterministic half-band FIR measures under 0.001 dB passband ripple through 90% of base-rate Nyquist, exactly -6.0206 dB at Nyquist, and at most -85 dB from 110% onward.
- Twelve focused tests cover response, exact 2x/4x delay, impulse placement, reset determinism, block independence, callback counts, DC/low-frequency gain, 44.1/48/96/192 kHz stability, hard-clip/foldback/FM-like alias reduction at 44.1/48/96 kHz, and allocator/deallocator auditing.
- External API coverage and `examples/selective_oversampling.rs` prove downstream input-processing and source-decimation composition without whole-voice oversampling.
- `cargo bench --bench oversampling` reports the required 1/16/64 instance x 1/16/64/256/1024 block matrix; the 2026-07-15 Apple M5 Pro reference run measured about 51-53 ns/sample/instance at 2x and 163-175 ns/sample/instance at 4x outside the block-size-1 single-instance cold edge.
- `VST3_SDK_DIR=/Users/portalsurfer/lib/vst3sdk bash scripts/ci_local.sh`: passed, including warnings-denied GUI/VST3 clippy and 128 VST3-feature tests.
- `VST3_SDK_DIR=/Users/portalsurfer/lib/vst3sdk cargo clippy --all-targets -- -D warnings`: passed.
- `VST3_SDK_DIR=/Users/portalsurfer/lib/vst3sdk cargo test --all`: passed, including 304 Patchbay GUI tests and 128 Toybox VST3-feature tests.
