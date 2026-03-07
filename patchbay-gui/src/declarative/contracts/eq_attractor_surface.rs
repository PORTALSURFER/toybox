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
/// This model carries preview-only wave and attractor-well inputs. The host
/// plugin remains responsible for the real DSP, automation, and transport
/// policy; the surface only mirrors that behavior deterministically.
#[derive(Clone, Debug, PartialEq)]
pub struct EqAttractorSurfaceModel {
    /// Interactive attractor handles.
    pub attractors: Vec<EqAttractor>,
    /// Per-attractor pull strengths used by the static wells.
    pub attractor_pulls: Vec<f32>,
    /// Per-attractor influence radii in normalized x space.
    pub attractor_radii: Vec<f32>,
    /// Global moving-wave depth in `[0, 1]`.
    pub wave_depth: f32,
    /// Global moving-wave cycle multiplier.
    pub wave_cycles: f32,
    /// Global moving-wave phase token in radians.
    ///
    /// This lets the preview animate one shared wave without inventing
    /// independent per-attractor motion.
    pub wave_phase_token: f32,
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
            attractor_pulls: Vec::new(),
            attractor_radii: Vec::new(),
            wave_depth: 1.0,
            wave_cycles: 1.0,
            wave_phase_token: 0.0,
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
    /// pads attractor control vectors with conservative defaults.
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
        let attractor_pulls =
            sanitize_scalar_vec(&self.attractor_pulls, attractor_len, 1.0, 0.0, Some(4.0));
        let attractor_radii =
            sanitize_scalar_vec(&self.attractor_radii, attractor_len, 0.12, 0.01, Some(1.0));

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
            attractor_pulls,
            attractor_radii,
            wave_depth: finite_scalar(self.wave_depth, 1.0).clamp(0.0, 1.0),
            wave_cycles: finite_scalar(self.wave_cycles, 1.0).max(0.0),
            wave_phase_token: finite_scalar(self.wave_phase_token, 0.0),
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

/// Return a finite scalar value or the provided fallback.
fn finite_scalar(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        fallback
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
    /// Motion smoothing time constant for shared wave inputs in seconds.
    pub motion_smoothing_seconds: f32,
    /// Smoothing time constant for attractor-well values in seconds.
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

#[cfg(test)]
mod eq_attractor_surface_contract_tests {
    use super::*;

    #[test]
    fn model_normalized_clamps_and_pads_vectors() {
        let model = EqAttractorSurfaceModel {
            attractors: vec![EqAttractor::new(1, -1.0, 2.0), EqAttractor::new(2, 0.2, 0.8)],
            attractor_pulls: vec![-3.0],
            attractor_radii: vec![f32::INFINITY],
            wave_depth: 2.0,
            wave_cycles: f32::NAN,
            wave_phase_token: f32::INFINITY,
            reverse_global: true,
            freq_min_hz: f32::INFINITY,
            freq_max_hz: f32::NEG_INFINITY,
            eq_bands: 0,
            eq_depth_db: -2.0,
        };

        let normalized = model.normalized();
        assert_eq!(normalized.attractors.len(), 2);
        assert_eq!(normalized.attractors[0].x, 0.0);
        assert_eq!(normalized.attractors[0].y, 1.0);
        assert_eq!(normalized.attractor_pulls, vec![0.0, 1.0]);
        assert_eq!(normalized.attractor_radii, vec![0.12, 0.12]);
        assert_eq!(normalized.wave_depth, 1.0);
        assert_eq!(normalized.wave_cycles, 1.0);
        assert_eq!(normalized.wave_phase_token, 0.0);
        assert_eq!(normalized.eq_bands, 1);
        assert_eq!(normalized.eq_depth_db, 2.0);
        assert!(normalized.freq_max_hz > normalized.freq_min_hz);
    }

    #[test]
    fn style_normalized_clamps_invalid_ranges() {
        let style = EqAttractorSurfaceStyle {
            show_grid: true,
            show_nodes: false,
            db_range: -0.001,
            curve_samples: 1,
            padding_px: -50,
            color_policy: EqAttractorColorPolicy::PerAttractorAccent,
            motion_smoothing_seconds: f32::INFINITY,
            attractor_smoothing_seconds: 0.0,
            band_gain_smoothing_seconds: 99.0,
        };

        let normalized = style.normalized();
        assert_eq!(normalized.db_range, 0.1);
        assert_eq!(normalized.curve_samples, 16);
        assert_eq!(normalized.padding_px, 0);
        assert_eq!(normalized.motion_smoothing_seconds, 0.02);
        assert_eq!(normalized.attractor_smoothing_seconds, 0.001);
        assert_eq!(normalized.band_gain_smoothing_seconds, 2.0);
    }
}
