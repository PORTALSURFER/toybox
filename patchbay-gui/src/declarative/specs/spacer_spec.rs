/// Spacer specification.
#[derive(Clone, Debug)]
pub struct SpacerSpec {
    /// Spacer size.
    pub size: Size,
}

impl SpacerSpec {
    /// Create a fixed spacer.
    pub const fn new(size: Size) -> Self {
        Self { size }
    }
}
