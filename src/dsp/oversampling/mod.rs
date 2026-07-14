//! Fixed-factor realtime mono audio oversampling.
//!
//! The stages use one deterministic 111-tap linear-phase half-band FIR. Its
//! zero taps are removed from the runtime dot products, leaving 56 multiply-
//! accumulates for the nontrivial polyphase branch. The response is flat to
//! within 0.001 dB through 90% of the base-rate Nyquist frequency and is below
//! -85 dB from 110% of base-rate Nyquist onward. See `docs/oversampling.md` for
//! the full response, latency, and CPU contract.
//!
//! All processing state is stored inline. Calls to `process` and `reset` do not
//! allocate, lock, log, panic, or use dynamic dispatch.

/// Number of nonzero coefficients in the even polyphase branch.
const EVEN_TAP_COUNT: usize = 56;
/// Full half-band filter length, including structural zero taps.
const FILTER_TAP_COUNT: usize = 111;
/// Linear-phase group delay in samples at the stage's high sample rate.
const HIGH_RATE_GROUP_DELAY: u32 = (FILTER_TAP_COUNT as u32 - 1) / 2;
/// Base-input delay used by the interpolation branch at odd high-rate phases.
const INTERPOLATOR_CENTER_DELAY: usize = 27;
/// High-rate odd-phase pair delay used by the decimator center tap.
const DECIMATOR_CENTER_DELAY: usize = 28;
/// Largest whole-sample delay needed by [`DryPathAligner`].
const MAX_DRY_DELAY_SAMPLES: usize = 83;
/// Inline ring-buffer capacity for dry-path alignment.
const DRY_BUFFER_LEN: usize = MAX_DRY_DELAY_SAMPLES + 1;

/// Nonzero even-indexed coefficients of the 111-tap half-band FIR.
///
/// These values were generated once from a Kaiser-windowed ideal half-band
/// impulse (`beta = 8.6`), made exactly symmetric, and normalized so the
/// off-center taps sum to 0.5. The omitted odd taps are zero except for the
/// center tap at index 55, which is exactly 0.5.
const EVEN_COEFFICIENTS: [f32; EVEN_TAP_COUNT] = [
    -7.711_9e-6,
    2.261_266_7e-5,
    -4.858_167_8e-5,
    9.021_717_4e-5,
    -1.531_966_3e-4,
    2.443_755_8e-4,
    -3.718_784e-4,
    5.451_852e-4,
    -7.752_225e-4,
    1.074_474_1e-3,
    -1.457_133_4e-3,
    1.939_335_6e-3,
    -2.539_523e-3,
    3.279_03e-3,
    -4.183_018_6e-3,
    5.281_977_4e-3,
    -6.614_138_4e-3,
    8.229_431e-3,
    -1.019_609_7e-2,
    1.261_215_1e-2,
    -1.562_616_8e-2,
    1.947_742_3e-2,
    -2.457_990_3e-2,
    3.171_840_7e-2,
    -4.258_075e-2,
    6.156_697e-2,
    -1.048_347_1e-1,
    3.178_864_4e-1,
    3.178_864_4e-1,
    -1.048_347_1e-1,
    6.156_697e-2,
    -4.258_075e-2,
    3.171_840_7e-2,
    -2.457_990_3e-2,
    1.947_742_3e-2,
    -1.562_616_8e-2,
    1.261_215_1e-2,
    -1.019_609_7e-2,
    8.229_431e-3,
    -6.614_138_4e-3,
    5.281_977_4e-3,
    -4.183_018_6e-3,
    3.279_03e-3,
    -2.539_523e-3,
    1.939_335_6e-3,
    -1.457_133_4e-3,
    1.074_474_1e-3,
    -7.752_225e-4,
    5.451_852e-4,
    -3.718_784e-4,
    2.443_755_8e-4,
    -1.531_966_3e-4,
    9.021_717_4e-5,
    -4.858_167_8e-5,
    2.261_266_7e-5,
    -7.711_9e-6,
];

/// An exact rational delay measured in base-rate samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SampleDelay {
    /// Delay numerator.
    numerator: u32,
    /// Delay denominator, always nonzero.
    denominator: u32,
}

impl SampleDelay {
    /// Construct an already-reduced positive rational delay.
    const fn new(numerator: u32, denominator: u32) -> Self {
        Self {
            numerator,
            denominator,
        }
    }

    /// Return the exact delay numerator.
    pub const fn numerator(self) -> u32 {
        self.numerator
    }

    /// Return the exact delay denominator.
    pub const fn denominator(self) -> u32 {
        self.denominator
    }

    /// Return the whole-sample portion of the delay.
    pub const fn whole_samples(self) -> u32 {
        self.numerator / self.denominator
    }

    /// Return the numerator of the fractional remainder.
    pub const fn fractional_numerator(self) -> u32 {
        self.numerator % self.denominator
    }

    /// Return whether the delay is an integer number of base-rate samples.
    pub const fn is_integer(self) -> bool {
        self.fractional_numerator() == 0
    }

    /// Return the delay as a floating-point number of base-rate samples.
    pub fn as_f64(self) -> f64 {
        f64::from(self.numerator) / f64::from(self.denominator)
    }
}

/// Supported fixed oversampling factors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OversamplingFactor {
    /// Two high-rate samples per base-rate sample.
    Two,
    /// Four high-rate samples per base-rate sample.
    Four,
}

impl OversamplingFactor {
    /// Return the integer factor.
    pub const fn as_usize(self) -> usize {
        match self {
            Self::Two => 2,
            Self::Four => 4,
        }
    }

    /// Return the interpolation-only group delay at the base sample rate.
    pub const fn interpolation_delay(self) -> SampleDelay {
        match self {
            Self::Two => SampleDelay::new(HIGH_RATE_GROUP_DELAY, 2),
            Self::Four => SampleDelay::new(HIGH_RATE_GROUP_DELAY * 3, 4),
        }
    }

    /// Return the source-decimation group delay at the base sample rate.
    pub const fn decimation_delay(self) -> SampleDelay {
        self.interpolation_delay()
    }

    /// Return the input-process-output latency at the base sample rate.
    pub const fn processing_delay(self) -> SampleDelay {
        match self {
            Self::Two => SampleDelay::new(HIGH_RATE_GROUP_DELAY, 1),
            Self::Four => SampleDelay::new(HIGH_RATE_GROUP_DELAY * 3, 2),
        }
    }
}

/// Advance a ring position by one slot.
#[inline]
fn advance(position: usize) -> usize {
    if position + 1 == EVEN_TAP_COUNT {
        0
    } else {
        position + 1
    }
}

/// Resolve a delay relative to the current ring position.
#[inline]
fn delayed_index(position: usize, delay: usize) -> usize {
    (position + EVEN_TAP_COUNT - delay) % EVEN_TAP_COUNT
}

/// Evaluate the nonzero even phase of the half-band FIR.
#[inline]
fn even_phase(history: &[f32; EVEN_TAP_COUNT], position: usize) -> f32 {
    let mut output = 0.0;
    for (delay, coefficient) in EVEN_COEFFICIENTS.iter().enumerate() {
        output += coefficient * history[delayed_index(position, delay)];
    }
    output
}

/// A realtime-safe 2x half-band interpolation stage.
#[derive(Debug, Clone)]
pub struct HalfBandInterpolator2x {
    /// Base-rate input history for the nonzero polyphase branch.
    history: [f32; EVEN_TAP_COUNT],
    /// Ring position containing the newest input sample.
    position: usize,
}

impl Default for HalfBandInterpolator2x {
    fn default() -> Self {
        Self {
            history: [0.0; EVEN_TAP_COUNT],
            position: EVEN_TAP_COUNT - 1,
        }
    }
}

impl HalfBandInterpolator2x {
    /// Construct a cleared interpolation stage.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all filter history.
    pub fn reset(&mut self) {
        self.history.fill(0.0);
        self.position = EVEN_TAP_COUNT - 1;
    }

    /// Interpolate one base-rate sample into two chronological high-rate samples.
    #[inline]
    pub fn process(&mut self, input: f32) -> [f32; 2] {
        self.position = advance(self.position);
        self.history[self.position] = input;
        let even = 2.0 * even_phase(&self.history, self.position);
        let odd = self.history[delayed_index(self.position, INTERPOLATOR_CENTER_DELAY)];
        [even, odd]
    }

    /// Return this stage's group delay in base-input samples.
    pub const fn latency() -> SampleDelay {
        SampleDelay::new(HIGH_RATE_GROUP_DELAY, 2)
    }
}

/// A realtime-safe 2x half-band decimation stage.
#[derive(Debug, Clone)]
pub struct HalfBandDecimator2x {
    /// Even high-rate phase history.
    even_history: [f32; EVEN_TAP_COUNT],
    /// Odd high-rate phase history used by the exact center tap.
    odd_history: [f32; EVEN_TAP_COUNT],
    /// Ring position containing the newest input pair.
    position: usize,
}

impl Default for HalfBandDecimator2x {
    fn default() -> Self {
        Self {
            even_history: [0.0; EVEN_TAP_COUNT],
            odd_history: [0.0; EVEN_TAP_COUNT],
            position: EVEN_TAP_COUNT - 1,
        }
    }
}

impl HalfBandDecimator2x {
    /// Construct a cleared decimation stage.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all filter history.
    pub fn reset(&mut self) {
        self.even_history.fill(0.0);
        self.odd_history.fill(0.0);
        self.position = EVEN_TAP_COUNT - 1;
    }

    /// Decimate two chronological high-rate samples into one base-rate sample.
    #[inline]
    pub fn process(&mut self, high_rate: [f32; 2]) -> f32 {
        self.position = advance(self.position);
        self.even_history[self.position] = high_rate[0];
        self.odd_history[self.position] = high_rate[1];
        even_phase(&self.even_history, self.position)
            + 0.5 * self.odd_history[delayed_index(self.position, DECIMATOR_CENTER_DELAY)]
    }

    /// Return this stage's group delay in output/base-rate samples.
    pub const fn latency() -> SampleDelay {
        SampleDelay::new(HIGH_RATE_GROUP_DELAY, 2)
    }
}

/// A fixed 2x input-process-output mono oversampler.
#[derive(Debug, Clone, Default)]
pub struct MonoOversampler2x {
    /// Input interpolation stage.
    interpolator: HalfBandInterpolator2x,
    /// Output decimation stage.
    decimator: HalfBandDecimator2x,
}

impl MonoOversampler2x {
    /// Construct a cleared 2x oversampler.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear interpolation and decimation history.
    pub fn reset(&mut self) {
        self.interpolator.reset();
        self.decimator.reset();
    }

    /// Process one input through a caller-supplied operation at 2x sample rate.
    #[inline]
    pub fn process<F>(&mut self, input: f32, mut high_rate_process: F) -> f32
    where
        F: FnMut(f32) -> f32,
    {
        let high_rate = self.interpolator.process(input);
        self.decimator.process([
            high_rate_process(high_rate[0]),
            high_rate_process(high_rate[1]),
        ])
    }

    /// Return the exact input-process-output latency.
    pub const fn latency() -> SampleDelay {
        OversamplingFactor::Two.processing_delay()
    }
}

/// A fixed 4x input-process-output mono oversampler composed from 2x stages.
#[derive(Debug, Clone, Default)]
pub struct MonoOversampler4x {
    /// Base-rate to 2x interpolation stage.
    first_interpolator: HalfBandInterpolator2x,
    /// 2x to 4x interpolation stage.
    second_interpolator: HalfBandInterpolator2x,
    /// 4x to 2x decimation stage.
    first_decimator: HalfBandDecimator2x,
    /// 2x to base-rate decimation stage.
    second_decimator: HalfBandDecimator2x,
}

impl MonoOversampler4x {
    /// Construct a cleared 4x oversampler.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear every interpolation and decimation stage.
    pub fn reset(&mut self) {
        self.first_interpolator.reset();
        self.second_interpolator.reset();
        self.first_decimator.reset();
        self.second_decimator.reset();
    }

    /// Process one input through a caller-supplied operation at 4x sample rate.
    #[inline]
    pub fn process<F>(&mut self, input: f32, mut high_rate_process: F) -> f32
    where
        F: FnMut(f32) -> f32,
    {
        let first_level = self.first_interpolator.process(input);
        let high_pair_a = self.second_interpolator.process(first_level[0]);
        let middle_a = self.first_decimator.process([
            high_rate_process(high_pair_a[0]),
            high_rate_process(high_pair_a[1]),
        ]);
        let high_pair_b = self.second_interpolator.process(first_level[1]);
        let middle_b = self.first_decimator.process([
            high_rate_process(high_pair_b[0]),
            high_rate_process(high_pair_b[1]),
        ]);
        self.second_decimator.process([middle_a, middle_b])
    }

    /// Return the exact input-process-output latency.
    pub const fn latency() -> SampleDelay {
        OversamplingFactor::Four.processing_delay()
    }
}

/// A fixed-factor mono oversampler selected once during construction.
///
/// This wrapper performs only an enum match per base-rate sample. The callback
/// remains statically dispatched and can be inlined.
#[derive(Debug, Clone)]
pub struct MonoOversampler {
    /// Inline state for exactly the selected factor.
    state: MonoOversamplerState,
}

/// Factor-specific inline storage used by [`MonoOversampler`].
#[derive(Debug, Clone)]
#[allow(
    clippy::large_enum_variant,
    reason = "fixed inline DSP state deliberately avoids heap indirection"
)]
enum MonoOversamplerState {
    /// Selected 2x state.
    Two(MonoOversampler2x),
    /// Selected 4x state.
    Four(MonoOversampler4x),
}

impl MonoOversampler {
    /// Construct a cleared oversampler for the selected fixed factor.
    pub fn new(factor: OversamplingFactor) -> Self {
        Self {
            state: match factor {
                OversamplingFactor::Two => MonoOversamplerState::Two(MonoOversampler2x::new()),
                OversamplingFactor::Four => MonoOversamplerState::Four(MonoOversampler4x::new()),
            },
        }
    }

    /// Return the selected factor.
    pub const fn factor(&self) -> OversamplingFactor {
        match &self.state {
            MonoOversamplerState::Two(_) => OversamplingFactor::Two,
            MonoOversamplerState::Four(_) => OversamplingFactor::Four,
        }
    }

    /// Return the selected input-process-output latency.
    pub const fn latency(&self) -> SampleDelay {
        self.factor().processing_delay()
    }

    /// Clear all filter history.
    pub fn reset(&mut self) {
        match &mut self.state {
            MonoOversamplerState::Two(processor) => processor.reset(),
            MonoOversamplerState::Four(processor) => processor.reset(),
        }
    }

    /// Process one sample through the callback at the selected high rate.
    #[inline]
    pub fn process<F>(&mut self, input: f32, high_rate_process: F) -> f32
    where
        F: FnMut(f32) -> f32,
    {
        match &mut self.state {
            MonoOversamplerState::Two(processor) => processor.process(input, high_rate_process),
            MonoOversamplerState::Four(processor) => processor.process(input, high_rate_process),
        }
    }
}

/// A 2x source-stage decimator for caller-generated high-rate samples.
#[derive(Debug, Clone, Default)]
pub struct SourceDecimator2x {
    /// Base-rate output stage.
    decimator: HalfBandDecimator2x,
}

impl SourceDecimator2x {
    /// Construct a cleared source decimator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear filter history.
    pub fn reset(&mut self) {
        self.decimator.reset();
    }

    /// Decimate two chronological caller-generated samples.
    #[inline]
    pub fn process(&mut self, high_rate: [f32; 2]) -> f32 {
        self.decimator.process(high_rate)
    }

    /// Return the exact source-decimation latency.
    pub const fn latency() -> SampleDelay {
        OversamplingFactor::Two.decimation_delay()
    }
}

/// A 4x source-stage decimator composed from two 2x stages.
#[derive(Debug, Clone, Default)]
pub struct SourceDecimator4x {
    /// 4x to 2x stage.
    first: HalfBandDecimator2x,
    /// 2x to base-rate stage.
    second: HalfBandDecimator2x,
}

impl SourceDecimator4x {
    /// Construct a cleared source decimator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear both decimation stages.
    pub fn reset(&mut self) {
        self.first.reset();
        self.second.reset();
    }

    /// Decimate four chronological caller-generated samples.
    #[inline]
    pub fn process(&mut self, high_rate: [f32; 4]) -> f32 {
        let first = self.first.process([high_rate[0], high_rate[1]]);
        let second = self.first.process([high_rate[2], high_rate[3]]);
        self.second.process([first, second])
    }

    /// Return the exact source-decimation latency.
    pub const fn latency() -> SampleDelay {
        OversamplingFactor::Four.decimation_delay()
    }
}

/// Inline dry-path alignment for oversampled wet/dry processing.
///
/// Fractional delays use deterministic first-order linear interpolation. This
/// has exact DC gain and exact group delay at DC; its high-frequency magnitude
/// droop is documented because it can matter for equal-power wet/dry mixes.
#[derive(Debug, Clone)]
pub struct DryPathAligner {
    /// Exact processing delay being matched.
    delay: SampleDelay,
    /// Inline dry-sample history.
    buffer: [f32; DRY_BUFFER_LEN],
    /// Ring position receiving the current sample.
    position: usize,
}

impl DryPathAligner {
    /// Construct a cleared aligner for a 2x or 4x wet processing path.
    pub fn new(factor: OversamplingFactor) -> Self {
        Self {
            delay: factor.processing_delay(),
            buffer: [0.0; DRY_BUFFER_LEN],
            position: 0,
        }
    }

    /// Return the exact delay applied to the dry path.
    pub const fn latency(&self) -> SampleDelay {
        self.delay
    }

    /// Clear all dry-path history.
    pub fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.position = 0;
    }

    /// Delay one dry sample to align it with the selected wet path.
    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        self.buffer[self.position] = input;
        let whole = self.delay.whole_samples() as usize;
        let fractional = self.delay.fractional_numerator() as f32 / self.delay.denominator() as f32;
        let newer = self.buffer[(self.position + DRY_BUFFER_LEN - whole) % DRY_BUFFER_LEN];
        let older = self.buffer[(self.position + DRY_BUFFER_LEN - whole - 1) % DRY_BUFFER_LEN];
        let output = newer + (older - newer) * fractional;
        self.position = (self.position + 1) % DRY_BUFFER_LEN;
        output
    }
}

#[cfg(test)]
mod tests;
