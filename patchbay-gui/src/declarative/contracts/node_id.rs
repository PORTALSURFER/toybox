/// Opaque identifier for one node in a validated declarative UI tree.
///
/// `NodeId` values are deterministic for a fixed tree structure and are used by
/// [`LayoutEngineState`] for explicit subtree invalidation. Keyed nodes are
/// key-scoped to remain stable across sibling reordering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(u64);

impl NodeId {
    /// Build a node identifier from a deterministic hash value.
    pub(crate) fn from_hash(value: u64) -> Self {
        Self(value)
    }
}
