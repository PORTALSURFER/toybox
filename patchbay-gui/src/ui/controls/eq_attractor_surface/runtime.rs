/// Smoothed per-attractor runtime fields used by the EQ attractor surface.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct EqAttractorSurfaceSmoothedAttractorState {
    /// Smoothed normalized x coordinate.
    pub(crate) x: f32,
    /// Smoothed normalized y coordinate.
    pub(crate) y: f32,
    /// Smoothed depth value.
    pub(crate) depth: f32,
    /// Smoothed cycles value.
    pub(crate) cycles: f32,
    /// Smoothed rate in Hz.
    pub(crate) rate_hz: f32,
}

/// Per-widget runtime cache for EQ attractor surface rendering.
#[derive(Clone, Debug, Default)]
pub(crate) struct EqAttractorSurfaceRuntimeState {
    /// Active attractor id while a pointer drag gesture is in progress.
    pub(crate) active_drag_id: Option<u64>,
    /// Smoothed global warp value.
    pub(crate) smoothed_warp: f32,
    /// Smoothed global pull-force value.
    pub(crate) smoothed_pull_force: f32,
    /// Smoothed per-band gain values.
    pub(crate) smoothed_band_gains: Vec<f32>,
    /// Smoothed per-attractor runtime values keyed by attractor id.
    pub(crate) smoothed_attractors: HashMap<u64, EqAttractorSurfaceSmoothedAttractorState>,
    /// True once runtime values have been initialized from model targets.
    pub(crate) initialized: bool,
}

impl<'a> Ui<'a> {
    /// Load runtime state for one EQ attractor surface widget.
    pub(crate) fn begin_eq_attractor_surface_runtime(
        &mut self,
        id: WidgetId,
    ) -> EqAttractorSurfaceRuntimeState {
        self.state
            .eq_attractor_surface_runtime
            .get(&id)
            .cloned()
            .unwrap_or_default()
    }

    /// Persist runtime state for one EQ attractor surface widget.
    pub(crate) fn set_eq_attractor_surface_runtime(
        &mut self,
        id: WidgetId,
        runtime: EqAttractorSurfaceRuntimeState,
    ) {
        self.state.eq_attractor_surface_runtime.insert(id, runtime);
    }
}
