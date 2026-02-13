impl Node {
    /// Set cross-axis alignment for row/column nodes.
    ///
    /// Non-flex node kinds are returned unchanged.
    pub fn align(mut self, align: Align) -> Self {
        if let Self::Row(flex) | Self::Column(flex) = &mut self {
            flex.align = align;
        }
        self
    }

    /// Align row/column children to cross-axis start.
    pub fn align_start(self) -> Self {
        self.align(Align::Start)
    }

    /// Center row/column children on the cross-axis.
    pub fn align_center(self) -> Self {
        self.align(Align::Center)
    }

    /// Align row/column children to cross-axis end.
    pub fn align_end(self) -> Self {
        self.align(Align::End)
    }

    /// Stretch row/column children across the cross-axis.
    pub fn align_stretch(self) -> Self {
        self.align(Align::Stretch)
    }

    /// Set main-axis distribution for row/column nodes.
    ///
    /// Non-flex node kinds are returned unchanged.
    pub fn justify(mut self, justify: Justify) -> Self {
        if let Self::Row(flex) | Self::Column(flex) = &mut self {
            flex.justify = justify;
        }
        self
    }

    /// Pack row/column children at main-axis start.
    pub fn justify_start(self) -> Self {
        self.justify(Justify::Start)
    }

    /// Center row/column children on the main axis.
    pub fn justify_center(self) -> Self {
        self.justify(Justify::Center)
    }

    /// Pack row/column children at main-axis end.
    pub fn justify_end(self) -> Self {
        self.justify(Justify::End)
    }

    /// Distribute row/column spacing between children.
    pub fn justify_space_between(self) -> Self {
        self.justify(Justify::SpaceBetween)
    }

    /// Distribute row/column spacing around children.
    pub fn justify_space_around(self) -> Self {
        self.justify(Justify::SpaceAround)
    }

    /// Distribute row/column spacing evenly including edges.
    pub fn justify_space_evenly(self) -> Self {
        self.justify(Justify::SpaceEvenly)
    }
}
