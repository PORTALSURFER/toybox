/// Declarative container category associated with a layout diagnostic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutContainerKind {
    /// Root frame wrapper that hosts the full declarative content tree.
    RootFrame,
    /// Panel container.
    Panel,
    /// Flex container (`Row`/`Column`).
    Flex,
    /// Grid container.
    Grid,
    /// Absolute container.
    Absolute,
    /// Stack container.
    Stack,
    /// Scroll-view container.
    ScrollView,
    /// Wrap container.
    Wrap,
}

/// Severity level for a runtime layout diagnostic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutDiagnosticLevel {
    /// Non-fatal layout condition.
    Warning,
}

/// Runtime diagnostic emitted during declarative layout resolution.
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutDiagnostic {
    /// Severity for this diagnostic.
    pub level: LayoutDiagnosticLevel,
    /// Container category where the condition was observed.
    pub container: LayoutContainerKind,
    /// Stable diagnostic message.
    pub message: &'static str,
    /// Requested layout rectangle.
    pub requested_rect: crate::canvas::Rect,
    /// Effective container bounds.
    pub bounds: crate::canvas::Rect,
}
