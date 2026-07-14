//! Stable CPU benchmark for base-rate, 2x, and 4x nonlinear processing.

use std::hint::black_box;
use std::time::{Duration, Instant};
use toybox::dsp::{MonoOversampler, OversamplingFactor};

/// Block sizes required by the reusable oversampling contract.
const BLOCK_SIZES: [usize; 5] = [1, 16, 64, 256, 1024];
/// Parallel mono instance counts required by the benchmark contract.
const INSTANCE_COUNTS: [usize; 3] = [1, 16, 64];
/// Approximate base-rate sample/instance operations per reported row.
const TARGET_OPERATIONS: usize = 1_048_576;

/// Deterministic nonlinear workload shared by every quality mode.
#[inline]
fn nonlinear(sample: f32) -> f32 {
    let driven = sample * 2.7;
    driven / (1.0 + driven.abs())
}

/// Return the number of outer iterations for one benchmark shape.
fn iteration_count(block_size: usize, instances: usize) -> usize {
    (TARGET_OPERATIONS / (block_size * instances)).max(2)
}

/// Run the base-rate nonlinear workload.
fn run_base(block_size: usize, instances: usize) -> (Duration, usize) {
    let iterations = iteration_count(block_size, instances);
    let mut phase = 0.0_f32;
    let mut accumulator = 0.0_f32;
    let start = Instant::now();
    for _ in 0..iterations {
        for _ in 0..instances {
            for _ in 0..block_size {
                phase += 0.013_579;
                accumulator += nonlinear(black_box(phase.sin()));
            }
        }
    }
    black_box(accumulator);
    (start.elapsed(), iterations * block_size * instances)
}

/// Run one fixed oversampling factor.
fn run_oversampled(
    factor: OversamplingFactor,
    block_size: usize,
    instances: usize,
) -> (Duration, usize) {
    let iterations = iteration_count(block_size, instances);
    let mut processors: Vec<_> = (0..instances)
        .map(|_| MonoOversampler::new(factor))
        .collect();
    let mut phase = 0.0_f32;
    let mut accumulator = 0.0_f32;
    let start = Instant::now();
    for _ in 0..iterations {
        for processor in &mut processors {
            for _ in 0..block_size {
                phase += 0.013_579;
                accumulator += processor.process(black_box(phase.sin()), nonlinear);
            }
        }
    }
    black_box(accumulator);
    (start.elapsed(), iterations * block_size * instances)
}

/// Convert a benchmark duration to nanoseconds per base-rate sample/instance.
fn nanos_per_operation(duration: Duration, operations: usize) -> f64 {
    duration.as_secs_f64() * 1.0e9 / operations as f64
}

/// Print the full benchmark matrix as a Markdown-compatible table.
fn main() {
    println!(
        "| block | instances | base ns/sample | 2x ns/sample | 2x ratio | 4x ns/sample | 4x ratio |"
    );
    println!("|---:|---:|---:|---:|---:|---:|---:|");
    for block_size in BLOCK_SIZES {
        for instances in INSTANCE_COUNTS {
            let (base_duration, operations) = run_base(block_size, instances);
            let (two_duration, _) = run_oversampled(OversamplingFactor::Two, block_size, instances);
            let (four_duration, _) =
                run_oversampled(OversamplingFactor::Four, block_size, instances);
            let base = nanos_per_operation(base_duration, operations);
            let two = nanos_per_operation(two_duration, operations);
            let four = nanos_per_operation(four_duration, operations);
            println!(
                "| {block_size} | {instances} | {base:.2} | {two:.2} | {:.2}x | {four:.2} | {:.2}x |",
                two / base,
                four / base
            );
        }
    }
}
