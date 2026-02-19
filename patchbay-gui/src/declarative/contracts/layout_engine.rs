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
}

impl LayoutEngineState {
    /// Return the last registry generation observed by this engine.
    pub fn last_registry_version(&self) -> u64 {
        self.registry_version
    }

    /// Return current measure-cache counters.
    pub fn measure_cache_stats(&self) -> MeasureCacheStats {
        MeasureCacheStats {
            hits: self.measure_cache_hits,
            misses: self.measure_cache_misses,
            entries: self.measure_cache.len(),
        }
    }

    /// Resolve a `NodeId` for a unique declarative node key.
    pub fn node_id_for_key(&self, key: &str) -> Option<NodeId> {
        self.key_to_node_id.get(key).copied()
    }

    /// Return true when this engine registry contains `node_id`.
    pub fn contains_node(&self, node_id: NodeId) -> bool {
        self.registry.contains_key(&node_id)
    }

    /// Mark a node subtree as layout-dirty.
    pub fn invalidate_layout_subtree(&mut self, node_id: NodeId) {
        if !self.contains_node(node_id) {
            return;
        }
        self.mark_subtree_layout_dirty(node_id);
        self.mark_ancestor_layout_dirty(node_id);
    }

    /// Mark a node subtree as measure-dirty.
    ///
    /// Measure dirtiness implies layout dirtiness for the same subtree and
    /// ancestors.
    pub fn invalidate_measure_subtree(&mut self, node_id: NodeId) {
        if !self.contains_node(node_id) {
            return;
        }
        self.mark_subtree_measure_dirty(node_id);
        self.mark_ancestor_measure_dirty(node_id);
        self.mark_subtree_layout_dirty(node_id);
        self.mark_ancestor_layout_dirty(node_id);
    }

    /// Mark the full tree as layout-dirty.
    pub fn invalidate_all_layout(&mut self) {
        self.invalidate_all_layout_flag = true;
    }

    /// Mark the full tree as measure-dirty.
    ///
    /// This implies full-tree layout invalidation.
    pub fn invalidate_all_measure(&mut self) {
        self.invalidate_all_measure_flag = true;
        self.invalidate_all_layout_flag = true;
    }

    /// Rebuild the deterministic node registry when the spec changes.
    pub(crate) fn sync_registry(&mut self, spec: &UiSpec) {
        let signature = stable_debug_hash(spec);
        if self.registry_signature == signature && self.root_node_id.is_some() {
            return;
        }
        self.registry_signature = signature;
        self.registry_version = self.registry_version.saturating_add(1);
        self.registry.clear();
        self.key_to_node_id.clear();
        self.layout_dirty_nodes.clear();
        self.measure_dirty_nodes.clear();
        self.invalidate_all_measure_flag = true;
        self.invalidate_all_layout_flag = true;

        let root_path = format!("root-frame:{}", spec.root.key);
        let root_id = node_id_from_path(&root_path);
        self.root_node_id = Some(root_id);
        self.registry.insert(
            root_id,
            NodeRegistryEntry {
                parent: None,
                children: Vec::new(),
                node_hash: signature,
            },
        );
        self.key_to_node_id.insert(spec.root.key.clone(), root_id);
        self.walk_node(&spec.root.content, root_id, format!("{root_path}/content"));
    }

    /// Apply pending measure invalidations to cached subtree entries.
    pub(crate) fn apply_measure_invalidations(&mut self) {
        self.drop_stale_dirty_nodes();
        if self.invalidate_all_measure_flag {
            self.measure_cache.clear();
            self.measure_dirty_nodes.clear();
            self.invalidate_all_measure_flag = false;
            return;
        }
        if self.measure_dirty_nodes.is_empty() {
            return;
        }
        self.measure_cache
            .retain(|key, _| !self.measure_dirty_nodes.contains(&key.node_id));
        self.measure_dirty_nodes.clear();
    }

    /// Consume pending layout-dirty state for this frame.
    pub(crate) fn consume_layout_dirty(&mut self) {
        self.drop_stale_dirty_nodes();
        self.layout_dirty_nodes.clear();
        self.invalidate_all_layout_flag = false;
    }

    /// Resolve one cached subtree measurement by deterministic cache key.
    pub(crate) fn resolve_cached_subtree_measure<F>(
        &mut self,
        key: MeasureCacheKey,
        measure_fn: F,
    ) -> Size
    where
        F: FnOnce(&mut LayoutEngineState) -> Size,
    {
        if let Some(size) = self.measure_cache.get(&key).copied() {
            self.measure_cache_hits = self.measure_cache_hits.saturating_add(1);
            return size;
        }
        let measured = measure_fn(self);
        self.measure_cache.insert(key, measured);
        self.measure_cache_misses = self.measure_cache_misses.saturating_add(1);
        measured
    }

    /// Return the cached root node id if available.
    pub(crate) fn root_node_id(&self) -> Option<NodeId> {
        self.root_node_id
    }

    /// Return one child node id for a deterministic child index.
    pub(crate) fn child_node_id(&self, node_id: NodeId, child_index: usize) -> Option<NodeId> {
        self.registry
            .get(&node_id)
            .and_then(|entry| entry.children.get(child_index).copied())
    }

    /// Return cached subtree hash for `node_id` when available.
    pub(crate) fn node_hash(&self, node_id: NodeId) -> Option<u64> {
        self.registry.get(&node_id).map(|entry| entry.node_hash)
    }

    /// Return deterministic token hash helper.
    pub(crate) fn token_hash(tokens: &ThemeTokens) -> u64 {
        stable_debug_hash(tokens)
    }

    /// Mark all descendants of `root` as layout-dirty.
    fn mark_subtree_layout_dirty(&mut self, root: NodeId) {
        let mut stack = vec![root];
        while let Some(node_id) = stack.pop() {
            if !self.layout_dirty_nodes.insert(node_id) {
                continue;
            }
            if let Some(entry) = self.registry.get(&node_id) {
                stack.extend(entry.children.iter().copied());
            }
        }
    }

    /// Mark all descendants of `root` as measure-dirty.
    fn mark_subtree_measure_dirty(&mut self, root: NodeId) {
        let mut stack = vec![root];
        while let Some(node_id) = stack.pop() {
            if !self.measure_dirty_nodes.insert(node_id) {
                continue;
            }
            if let Some(entry) = self.registry.get(&node_id) {
                stack.extend(entry.children.iter().copied());
            }
        }
    }

    /// Mark all ancestors of `node_id` as layout-dirty.
    fn mark_ancestor_layout_dirty(&mut self, node_id: NodeId) {
        let mut current = self.registry.get(&node_id).and_then(|entry| entry.parent);
        while let Some(parent) = current {
            self.layout_dirty_nodes.insert(parent);
            current = self.registry.get(&parent).and_then(|entry| entry.parent);
        }
    }

    /// Mark all ancestors of `node_id` as measure-dirty.
    fn mark_ancestor_measure_dirty(&mut self, node_id: NodeId) {
        let mut current = self.registry.get(&node_id).and_then(|entry| entry.parent);
        while let Some(parent) = current {
            self.measure_dirty_nodes.insert(parent);
            current = self.registry.get(&parent).and_then(|entry| entry.parent);
        }
    }

    /// Walk one node subtree and register deterministic parent/child topology.
    fn walk_node(&mut self, node: &Node, parent_id: NodeId, path: String) -> NodeId {
        let node_id = node_id_from_path(&path);
        self.registry.insert(
            node_id,
            NodeRegistryEntry {
                parent: Some(parent_id),
                children: Vec::new(),
                node_hash: stable_debug_hash(node),
            },
        );
        self.registry
            .entry(parent_id)
            .or_default()
            .children
            .push(node_id);
        if let Some(key) = node_key(node) {
            self.key_to_node_id.insert(key.to_owned(), node_id);
        }
        match node {
            Node::Slot(slot) => {
                self.walk_node(slot.child(), node_id, format!("{path}/slot-child[0]"));
            }
            Node::Panel(panel) => {
                self.walk_node(&panel.content, node_id, format!("{path}/panel-content[0]"));
            }
            Node::PaddingBox(padding_box) => {
                self.walk_node(
                    padding_box.content(),
                    node_id,
                    format!("{path}/padding-content[0]"),
                );
            }
            Node::AlignBox(align_box) => {
                self.walk_node(align_box.content(), node_id, format!("{path}/align-content[0]"));
            }
            Node::Row(flex) | Node::Column(flex) => {
                for (index, child) in flex.children.iter().enumerate() {
                    self.walk_node(child, node_id, format!("{path}/flex-child[{index}]"));
                }
            }
            Node::Grid(grid) => {
                for (index, child) in grid.children.iter().enumerate() {
                    self.walk_node(child, node_id, format!("{path}/grid-child[{index}]"));
                }
            }
            Node::Absolute(absolute) => {
                for (index, child) in absolute.children.iter().enumerate() {
                    self.walk_node(&child.node, node_id, format!("{path}/absolute-child[{index}]"));
                }
            }
            Node::Stack(stack) => {
                for (index, child) in stack.children.iter().enumerate() {
                    self.walk_node(child, node_id, format!("{path}/stack-child[{index}]"));
                }
            }
            Node::ScrollView(scroll_view) => {
                self.walk_node(
                    scroll_view.content(),
                    node_id,
                    format!("{path}/scroll-content[0]"),
                );
            }
            Node::Wrap(wrap) => {
                for (index, child) in wrap.children.iter().enumerate() {
                    self.walk_node(child, node_id, format!("{path}/wrap-child[{index}]"));
                }
            }
            Node::SwitchLayout(switch_layout) => {
                for (index, case_entry) in switch_layout.cases().iter().enumerate() {
                    self.walk_node(
                        case_entry.child(),
                        node_id,
                        format!("{path}/switch-case[{index}]"),
                    );
                }
                self.walk_node(
                    switch_layout.fallback(),
                    node_id,
                    format!("{path}/switch-fallback[0]"),
                );
            }
            Node::Label(_)
            | Node::Spacer(_)
            | Node::Knob(_)
            | Node::Slider(_)
            | Node::Toggle(_)
            | Node::Button(_)
            | Node::Dropdown(_)
            | Node::Region(_)
            | Node::Indicator(_) => {}
        }
        node_id
    }

    /// Remove stale dirty node entries that are no longer present in registry.
    fn drop_stale_dirty_nodes(&mut self) {
        self.layout_dirty_nodes
            .retain(|node_id| self.registry.contains_key(node_id));
        self.measure_dirty_nodes
            .retain(|node_id| self.registry.contains_key(node_id));
    }
}

/// Return the optional declarative key for one node variant.
fn node_key(node: &Node) -> Option<&str> {
    match node {
        Node::Panel(panel) => Some(&panel.key),
        Node::Knob(knob) => Some(&knob.key),
        Node::Slider(slider) => Some(&slider.key),
        Node::Toggle(toggle) => Some(&toggle.key),
        Node::Button(button) => Some(&button.key),
        Node::Dropdown(dropdown) => Some(&dropdown.key),
        Node::Region(region) => Some(&region.key),
        _ => None,
    }
}

/// Build a deterministic node id from a structural path string.
fn node_id_from_path(path: &str) -> NodeId {
    NodeId::from_hash(stable_debug_hash(&path))
}

/// Compute a stable hash from the debug representation of a value.
pub(crate) fn stable_debug_hash(value: &impl std::fmt::Debug) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    use std::hash::Hash as _;
    use std::hash::Hasher as _;
    format!("{value:?}").hash(&mut hasher);
    hasher.finish()
}
