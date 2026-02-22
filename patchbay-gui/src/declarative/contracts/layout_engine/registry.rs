impl LayoutEngineState {
    /// Walk one node subtree and register deterministic parent/child topology.
    fn walk_node(&mut self, node: &Node, parent_id: NodeId, path: String) -> NodeId {
        let node_id = node_id_for_node(&path, node);
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
            Node::AspectBox(aspect_box) => {
                self.walk_node(
                    aspect_box.content(),
                    node_id,
                    format!("{path}/aspect-content[0]"),
                );
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
            Node::TextBox(_)
            | Node::Spacer(_)
            | Node::Knob(_)
            | Node::Slider(_)
            | Node::Toggle(_)
            | Node::Button(_)
            | Node::Dropdown(_)
            | Node::TabBar(_)
            | Node::CurveEditor(_)
            | Node::EqAttractorSurface(_)
            | Node::Region(_)
            | Node::Indicator(_) => {}
        }
        node_id
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
        Node::TabBar(tab_bar) => Some(&tab_bar.key),
        Node::CurveEditor(curve_editor) => Some(&curve_editor.key),
        Node::EqAttractorSurface(surface) => Some(&surface.key),
        Node::Region(region) => Some(&region.key),
        _ => None,
    }
}

/// Build a deterministic node id from a structural path string.
fn node_id_from_path(path: &str) -> NodeId {
    NodeId::from_hash(stable_debug_hash(&path))
}

/// Build a deterministic node id from a stable declarative key string.
fn node_id_from_key(key: &str) -> NodeId {
    NodeId::from_hash(stable_debug_hash(&("node-key", key)))
}

/// Build a deterministic node id for one declarative node.
///
/// Keyed nodes use key-scoped identifiers to avoid sibling-order aliasing
/// across structural edits. Anonymous nodes fall back to path-scoped ids.
fn node_id_for_node(path: &str, node: &Node) -> NodeId {
    if let Some(key) = node_key(node) {
        return node_id_from_key(key);
    }
    node_id_from_path(path)
}
