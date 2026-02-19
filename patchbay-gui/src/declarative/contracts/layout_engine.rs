/// Cache key for measured root-frame results.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MeasureCacheKey {
    /// Stable hash of the declarative UI spec shape.
    pub spec_hash: u64,
    /// Stable hash of active theme tokens.
    pub token_hash: u64,
    /// Surface width used during planning.
    pub surface_width: u32,
    /// Surface height used during planning.
    pub surface_height: u32,
}

/// Runtime layout engine state for deterministic dirty/cached rendering.
#[derive(Clone, Debug, Default)]
pub struct LayoutEngineState {
    /// Geometry-affecting dirty flag.
    pub layout_dirty: bool,
    /// Intrinsic/content-affecting dirty flag.
    pub measure_dirty: bool,
    /// Cached root measurements by deterministic key.
    pub measure_cache: std::collections::HashMap<MeasureCacheKey, Size>,
    /// Number of cache hits observed in this engine state.
    pub measure_cache_hits: u64,
    /// Number of cache misses observed in this engine state.
    pub measure_cache_misses: u64,
}

impl LayoutEngineState {
    /// Mark layout as dirty.
    pub fn mark_layout_dirty(&mut self) {
        self.layout_dirty = true;
    }

    /// Mark measure and layout as dirty.
    pub fn mark_measure_dirty(&mut self) {
        self.measure_dirty = true;
        self.layout_dirty = true;
    }

    /// Clear all cached measurements.
    pub fn clear_measure_cache(&mut self) {
        self.measure_cache.clear();
    }

    /// Resolve a cached measurement for the given key.
    pub fn resolve_cached_measure<F>(&mut self, key: MeasureCacheKey, measure_fn: F) -> (Size, bool)
    where
        F: FnOnce() -> Size,
    {
        if let Some(size) = self.measure_cache.get(&key).copied() {
            self.measure_cache_hits = self.measure_cache_hits.saturating_add(1);
            return (size, true);
        }
        let measured = measure_fn();
        self.measure_cache.insert(key, measured);
        self.measure_cache_misses = self.measure_cache_misses.saturating_add(1);
        (measured, false)
    }
}

