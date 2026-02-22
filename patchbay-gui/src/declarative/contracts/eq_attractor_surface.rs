/// One normalized attractor handle used by [`EqAttractorSurfaceModel`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EqAttractor {
    /// Stable entity id used for deterministic color assignment and actions.
    pub id: u64,
    /// Normalized x position in `[0, 1]`.
    pub x: f32,
    /// Normalized y position in `[0, 1]`.
    pub y: f32,
    /// Whether this attractor is currently selected.
    pub selected: bool,
}

impl EqAttractor {
    /// Build one attractor with normalized position clamping.
    pub fn new(id: u64, x: f32, y: f32) -> Self {
        Self {
            id,
            x: x.clamp(0.0, 1.0),
            y: y.clamp(0.0, 1.0),
            selected: false,
        }
    }

    /// Override the selected flag.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

/// Model payload consumed by [`EqAttractorSurfaceSpec`].
///
/// This model carries attractor geometry and shaping parameters only; DSP,
/// automation, and host transport policy remain plugin responsibilities.
#[derive(Clone, Debug, PartialEq)]
pub struct EqAttractorSurfaceModel {
    /// Interactive attractor handles.
    pub attractors: Vec<EqAttractor>,
    /// Global warp amount in `[0, 1]`.
    pub warp: f32,
    /// Global pull-force multiplier.
    pub pull_force: f32,
    /// Per-attractor depth values.
    pub depths: Vec<f32>,
    /// Per-attractor cycles values.
    pub cycles: Vec<f32>,
    /// Per-attractor rates in Hz.
    pub rates_hz: Vec<f32>,
    /// Global reverse toggle for curve animation direction.
    pub reverse_global: bool,
    /// Minimum displayed frequency in Hz.
    pub freq_min_hz: f32,
    /// Maximum displayed frequency in Hz.
    pub freq_max_hz: f32,
    /// Number of EQ bands used when generating the curve.
    pub eq_bands: usize,
    /// Maximum absolute gain depth in dB for generated band values.
    pub eq_depth_db: f32,
}

impl EqAttractorSurfaceModel {
    /// Build a model from attractor handles with conservative defaults.
    pub fn new(attractors: Vec<EqAttractor>) -> Self {
        Self {
            attractors,
            warp: 0.5,
            pull_force: 1.0,
            depths: Vec::new(),
            cycles: Vec::new(),
            rates_hz: Vec::new(),
            reverse_global: false,
            freq_min_hz: 20.0,
            freq_max_hz: 20_000.0,
            eq_bands: 32,
            eq_depth_db: 12.0,
        }
    }

    /// Return a normalized copy safe for deterministic rendering.
    ///
    /// This clamps coordinates/ranges, repairs invalid frequency domains, and
    /// pads per-attractor parameter vectors with defaults.
    pub(crate) fn normalized(&self) -> Self {
        let attractors = self
            .attractors
            .iter()
            .copied()
            .map(|attractor| EqAttractor {
                id: attractor.id,
                x: attractor.x.clamp(0.0, 1.0),
                y: attractor.y.clamp(0.0, 1.0),
                selected: attractor.selected,
            })
            .collect::<Vec<_>>();

        let attractor_len = attractors.len();
        let depths = sanitize_scalar_vec(&self.depths, attractor_len, 1.0, 0.0, Some(1.0));
        let cycles = sanitize_scalar_vec(&self.cycles, attractor_len, 1.0, 0.0, None);
        let rates_hz = sanitize_scalar_vec(&self.rates_hz, attractor_len, 0.0, 0.0, None);

        let mut min_hz = self.freq_min_hz;
        let mut max_hz = self.freq_max_hz;
        if !min_hz.is_finite() {
            min_hz = 20.0;
        }
        if !max_hz.is_finite() {
            max_hz = 20_000.0;
        }
        min_hz = min_hz.clamp(1.0, 48_000.0);
        max_hz = max_hz.clamp(min_hz + 1.0, 96_000.0);

        Self {
            attractors,
            warp: self.warp.clamp(0.0, 1.0),
            pull_force: if self.pull_force.is_finite() {
                self.pull_force.max(0.0)
            } else {
                0.0
            },
            depths,
            cycles,
            rates_hz,
            reverse_global: self.reverse_global,
            freq_min_hz: min_hz,
            freq_max_hz: max_hz,
            eq_bands: self.eq_bands.clamp(1, 1024),
            eq_depth_db: if self.eq_depth_db.is_finite() {
                self.eq_depth_db.abs().max(0.1)
            } else {
                12.0
            },
        }
    }
}

/// Fill and clamp one scalar vector to a target length.
fn sanitize_scalar_vec(
    values: &[f32],
    target_len: usize,
    default_value: f32,
    min_value: f32,
    max_value: Option<f32>,
) -> Vec<f32> {
    let mut normalized = Vec::with_capacity(target_len);
    for index in 0..target_len {
        let mut value = values.get(index).copied().unwrap_or(default_value);
        if !value.is_finite() {
            value = default_value;
        }
        value = value.max(min_value);
        if let Some(max_value) = max_value {
            value = value.min(max_value);
        }
        normalized.push(value);
    }
    normalized
}

/// Accent policy used while rendering attractor handles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EqAttractorColorPolicy {
    /// Resolve one stable accent per attractor id via `AccentKey::Entity(id)`.
    PerAttractorAccent,
    /// Use one explicit accent key for every attractor.
    SingleAccent(AccentKey),
}

/// Visual and smoothing options for the EQ attractor surface.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EqAttractorSurfaceStyle {
    /// Draw log-frequency guide lines.
    pub show_grid: bool,
    /// Draw attractor handle circles.
    pub show_nodes: bool,
    /// Displayed dB range used for y-axis normalization.
    pub db_range: f32,
    /// Number of x samples used for curve rendering.
    pub curve_samples: usize,
    /// Inner plot padding in pixels.
    pub padding_px: i32,
    /// Color policy for attractor handles.
    pub color_policy: EqAttractorColorPolicy,
    /// Motion smoothing time constant for warp/pull in seconds.
    pub motion_smoothing_seconds: f32,
    /// Smoothing time constant for attractor values in seconds.
    pub attractor_smoothing_seconds: f32,
    /// Smoothing time constant for band gains in seconds.
    pub band_gain_smoothing_seconds: f32,
}

impl Default for EqAttractorSurfaceStyle {
    fn default() -> Self {
        Self {
            show_grid: true,
            show_nodes: true,
            db_range: 24.0,
            curve_samples: 220,
            padding_px: 10,
            color_policy: EqAttractorColorPolicy::PerAttractorAccent,
            motion_smoothing_seconds: 0.02,
            attractor_smoothing_seconds: 0.18,
            band_gain_smoothing_seconds: 0.08,
        }
    }
}

impl EqAttractorSurfaceStyle {
    /// Build a style payload with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Return a normalized copy safe for rendering.
    pub(crate) fn normalized(self) -> Self {
        Self {
            show_grid: self.show_grid,
            show_nodes: self.show_nodes,
            db_range: if self.db_range.is_finite() {
                self.db_range.abs().max(0.1)
            } else {
                24.0
            },
            curve_samples: self.curve_samples.clamp(16, 4096),
            padding_px: self.padding_px.clamp(0, 1024),
            color_policy: self.color_policy,
            motion_smoothing_seconds: sanitize_time_seconds(self.motion_smoothing_seconds, 0.02),
            attractor_smoothing_seconds: sanitize_time_seconds(
                self.attractor_smoothing_seconds,
                0.18,
            ),
            band_gain_smoothing_seconds: sanitize_time_seconds(
                self.band_gain_smoothing_seconds,
                0.08,
            ),
        }
    }
}

/// Clamp smoothing constants to stable positive values.
fn sanitize_time_seconds(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.001, 2.0)
    } else {
        fallback
    }
}

/// Typed interaction event emitted by the EQ attractor surface.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EqAttractorSurfaceAction {
    /// Select one existing attractor.
    Select {
        /// Selected attractor id.
        id: u64,
    },
    /// Move one attractor to a new normalized position.
    Move {
        /// Moved attractor id.
        id: u64,
        /// New normalized x position in `[0, 1]`.
        x: f32,
        /// New normalized y position in `[0, 1]`.
        y: f32,
    },
    /// Request adding one new attractor at a normalized position.
    Add {
        /// Target normalized x position in `[0, 1]`.
        x: f32,
        /// Target normalized y position in `[0, 1]`.
        y: f32,
    },
    /// Request removing one attractor by id.
    Remove {
        /// Removed attractor id.
        id: u64,
    },
}
