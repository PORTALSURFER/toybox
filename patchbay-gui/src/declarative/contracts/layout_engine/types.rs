/// Cache key for measured subtree results.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MeasureCacheKey {
    /// Stable node identity for this cached measurement.
    pub node_id: NodeId,
    /// Stable hash of the node subtree content.
    pub node_hash: u64,
    /// Stable hash of active theme tokens.
    pub token_hash: u64,
}

/// Measure cache counters for engine diagnostics.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MeasureCacheStats {
    /// Number of cache hits observed in this engine state.
    pub hits: u64,
    /// Number of cache misses observed in this engine state.
    pub misses: u64,
    /// Number of cached entries retained in this engine state.
    pub entries: usize,
}

/// Metadata tracked for one registry node entry.
#[derive(Clone, Debug, Default)]
struct NodeRegistryEntry {
    /// Parent node id in the registry tree.
    parent: Option<NodeId>,
    /// Deterministic child list in traversal order.
    children: Vec<NodeId>,
    /// Stable debug hash of the represented subtree.
    node_hash: u64,
}

/// Bounded capacity for retained structural gap entries.
const MAX_STRUCTURAL_GAP_ENTRIES: usize = 64;

/// Structured reason for one detected strict-tree structural gap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructuralGapReason {
    /// A layout-subtree invalidation target was not present in the node registry.
    MissingLayoutSubtreeInvalidationTarget,
    /// A measure-subtree invalidation target was not present in the node registry.
    MissingMeasureSubtreeInvalidationTarget,
}

impl StructuralGapReason {
    /// Return the stable diagnostic message associated with this gap reason.
    pub fn diagnostic_message(self) -> &'static str {
        match self {
            Self::MissingLayoutSubtreeInvalidationTarget => {
                "layout subtree invalidation targeted a missing registry node"
            }
            Self::MissingMeasureSubtreeInvalidationTarget => {
                "measure subtree invalidation targeted a missing registry node"
            }
        }
    }
}

/// One structural gap event recorded by [`LayoutEngineState`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StructuralGapEntry {
    /// Missing registry node identifier.
    pub node_id: NodeId,
    /// Structured reason associated with the missing node.
    pub reason: StructuralGapReason,
}

/// Runtime layout engine state for deterministic dirty/cached rendering.
///
/// The engine owns a deterministic node registry and explicit subtree
/// invalidation APIs. Clients invalidate by [`NodeId`] rather than mutating
/// root-level dirty flags directly.
#[derive(Clone, Debug, Default)]
pub struct LayoutEngineState {
    /// Monotonic registry generation incremented on structural tree changes.
    registry_version: u64,
    /// Last observed stable hash of the full `UiSpec`.
    registry_signature: u64,
    /// Synthetic root container node id.
    root_node_id: Option<NodeId>,
    /// Lookup from declarative key to registered node id.
    key_to_node_id: std::collections::HashMap<String, NodeId>,
    /// Full registry map keyed by node id.
    registry: std::collections::HashMap<NodeId, NodeRegistryEntry>,
    /// Explicit layout-dirty subtree members for this frame.
    layout_dirty_nodes: std::collections::HashSet<NodeId>,
    /// Explicit measure-dirty subtree members for this frame.
    measure_dirty_nodes: std::collections::HashSet<NodeId>,
    /// Full-tree layout invalidation marker.
    invalidate_all_layout_flag: bool,
    /// Full-tree measure invalidation marker.
    invalidate_all_measure_flag: bool,
    /// Cached measured subtree sizes.
    measure_cache: std::collections::HashMap<MeasureCacheKey, Size>,
    /// Cumulative measure-cache hit counter.
    measure_cache_hits: u64,
    /// Cumulative measure-cache miss counter.
    measure_cache_misses: u64,
    /// Bounded structural gap events recorded between render passes.
    structural_gaps: Vec<StructuralGapEntry>,
}
