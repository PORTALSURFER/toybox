/// Declarative container category associated with a layout diagnostic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutContainerKind {
    /// Root frame wrapper that hosts the full declarative content tree.
    RootFrame,
    /// Panel container.
    Panel,
    /// Single-slot padding container.
    PaddingBox,
    /// Single-slot alignment container.
    AlignBox,
    /// Single-slot aspect-ratio container.
    AspectBox,
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
    /// Responsive switch-layout container.
    SwitchLayout,
}

/// Severity level for a runtime layout diagnostic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutDiagnosticLevel {
    /// Non-fatal layout condition.
    Warning,
}

/// Runtime diagnostics detail mode for declarative layout passes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LayoutDiagnosticsMode {
    /// Emit event-level diagnostics only.
    ///
    /// This preserves the existing lightweight behavior.
    #[default]
    EventsOnly,
    /// Emit per-node measured/resolved geometry diagnostics.
    PerNode,
}

/// Runtime diagnostic emitted during declarative layout resolution.
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutDiagnostic {
    /// Severity for this diagnostic.
    pub level: LayoutDiagnosticLevel,
    /// Structured code for this diagnostic event.
    pub code: LayoutDiagnosticCode,
    /// Container category where the condition was observed.
    pub container: LayoutContainerKind,
    /// Stable diagnostic message.
    pub message: &'static str,
    /// Requested layout rectangle.
    pub requested_rect: crate::canvas::Rect,
    /// Effective container bounds.
    pub bounds: crate::canvas::Rect,
}

/// Structured code for one runtime layout diagnostic event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutDiagnosticCode {
    /// Child rectangle was clipped to container bounds.
    OverflowClipped,
    /// Child rectangle did not intersect the bounds and was skipped.
    OverflowSkippedDisjoint,
    /// Child rectangle was compressed to fit bounds.
    OverflowCompressed,
    /// Child was skipped because container bounds collapsed to zero area.
    OverflowSkippedCollapsedBounds,
    /// Scroll-view content was compressed to the viewport size.
    ScrollViewContentCompressed,
    /// Layout engine observed an invalidation request that targeted no registry node.
    StructuralGapDetected,
}

/// Declarative node category associated with a per-node layout diagnostic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutNodeKind {
    /// Slot node.
    Slot,
    /// Panel container.
    Panel,
    /// Single-slot padding container.
    PaddingBox,
    /// Single-slot alignment container.
    AlignBox,
    /// Single-slot aspect-ratio container.
    AspectBox,
    /// Row container.
    Row,
    /// Column container.
    Column,
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
    /// Switch-layout container.
    SwitchLayout,
    /// Label widget.
    Label,
    /// Spacer widget.
    Spacer,
    /// Knob widget.
    Knob,
    /// Slider widget.
    Slider,
    /// Toggle widget.
    Toggle,
    /// Button widget.
    Button,
    /// Dropdown widget.
    Dropdown,
    /// Region widget.
    Region,
    /// Indicator widget.
    Indicator,
}

/// Reason flag associated with a per-node layout diagnostic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutNodeDiagnosticReason {
    /// Entry includes measured geometry.
    Measured,
    /// Entry includes resolved/final geometry.
    Resolved,
    /// Node size expanded due to minimum constraints.
    ClampedMin,
    /// Node size reduced due to max/available constraints.
    ClampedMax,
    /// Node rectangle was clipped by overflow policy.
    OverflowClipped,
    /// Node rectangle was compressed by overflow policy.
    OverflowCompressed,
    /// Node origin was offset relative to parent origin.
    Aligned,
    /// Switch-layout case branch was selected for this node.
    SwitchCaseSelected,
    /// Switch-layout fallback branch was selected for this node.
    FallbackSelected,
}

/// Per-node geometry diagnostic emitted during declarative render traversal.
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutNodeDiagnostic {
    /// Stable path of this node in render traversal order.
    pub node_path: String,
    /// Node kind for this diagnostic entry.
    pub node_kind: LayoutNodeKind,
    /// Measured geometry for this node.
    pub measured_rect: crate::canvas::Rect,
    /// Final resolved geometry for this node.
    pub resolved_rect: crate::canvas::Rect,
    /// Reason flags explaining why measured and resolved geometry differ.
    pub reasons: Vec<LayoutNodeDiagnosticReason>,
    /// Container category when this node is a container type.
    pub container: Option<LayoutContainerKind>,
}
