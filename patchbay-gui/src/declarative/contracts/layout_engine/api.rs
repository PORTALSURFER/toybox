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

    /// Return currently retained structural gap entries.
    pub fn structural_gaps(&self) -> &[StructuralGapEntry] {
        &self.structural_gaps
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
            self.record_structural_gap(
                node_id,
                StructuralGapReason::MissingLayoutSubtreeInvalidationTarget,
            );
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
            self.record_structural_gap(
                node_id,
                StructuralGapReason::MissingMeasureSubtreeInvalidationTarget,
            );
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

    /// Drain and return structural gap events captured since the previous drain.
    pub(crate) fn take_structural_gaps(&mut self) -> Vec<StructuralGapEntry> {
        std::mem::take(&mut self.structural_gaps)
    }

    /// Resolve one cached subtree measurement by deterministic cache key.
    pub(crate) fn resolve_cached_subtree_measure<F>(&mut self, key: MeasureCacheKey, measure_fn: F) -> Size
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
}
