/// Spacer specification.
#[derive(Clone, Debug)]
pub struct SpacerSpec {
    /// Layout constraints.
    pub layout: LayoutBox,
}

impl SpacerSpec {
    /// Create a spacer.
    pub const fn new() -> Self {
        Self {
            layout: LayoutBox::auto(),
        }
    }

    /// Override layout constraints.
    pub const fn layout(mut self, layout: LayoutBox) -> Self {
        self.layout = layout;
        self
    }
}
