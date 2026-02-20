impl LayoutEngineState {
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

    /// Remove stale dirty node entries that are no longer present in registry.
    fn drop_stale_dirty_nodes(&mut self) {
        self.layout_dirty_nodes
            .retain(|node_id| self.registry.contains_key(node_id));
        self.measure_dirty_nodes
            .retain(|node_id| self.registry.contains_key(node_id));
    }

    /// Record one structural gap event with bounded capacity and deduplication.
    fn record_structural_gap(&mut self, node_id: NodeId, reason: StructuralGapReason) {
        if self
            .structural_gaps
            .iter()
            .any(|entry| entry.node_id == node_id && entry.reason == reason)
        {
            return;
        }
        if self.structural_gaps.len() >= MAX_STRUCTURAL_GAP_ENTRIES {
            return;
        }
        self.structural_gaps.push(StructuralGapEntry { node_id, reason });
    }
}
