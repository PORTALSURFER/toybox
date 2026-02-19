impl Node {
    /// Create a row container.
    pub fn row(children: Vec<Node>) -> Self {
        Self::Row(FlexSpec::row(children))
    }

    /// Create a column container.
    pub fn column(children: Vec<Node>) -> Self {
        Self::Column(FlexSpec::column(children))
    }

    /// Create a stack container.
    pub fn stack(children: Vec<Node>) -> Self {
        Self::Stack(StackSpec::new(children))
    }

    /// Create a scroll-view container.
    pub fn scroll_view(content: Node) -> Self {
        Self::ScrollView(ScrollViewSpec::new(content))
    }

    /// Create a wrap container.
    pub fn wrap(children: Vec<Node>) -> Self {
        Self::Wrap(WrapSpec::new(children))
    }
}

/// Create a row container node.
pub fn row(children: Vec<Node>) -> Node {
    Node::row(children)
}

/// Create a column container node.
pub fn column(children: Vec<Node>) -> Node {
    Node::column(children)
}

/// Create a stack container node.
pub fn stack(children: Vec<Node>) -> Node {
    Node::stack(children)
}

/// Create a scroll-view container node.
pub fn scroll_view(content: Node) -> Node {
    Node::scroll_view(content)
}

/// Create a wrap container node.
pub fn wrap(children: Vec<Node>) -> Node {
    Node::wrap(children)
}

/// Create a slot container node.
pub fn slot(child: Node) -> Node {
    Node::slot(child)
}
