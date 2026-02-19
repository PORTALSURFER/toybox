/// Result of rendering a declarative UI tree.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderResult {
    /// Measured root size used for host auto-resize.
    pub measured_size: Size,
    /// Actions emitted during widget interaction handling.
    pub actions: Vec<UiAction>,
    /// Resolved uniform render scale applied for this frame.
    pub resolved_scale: f32,
    /// Resolved content rectangle used for rendering root content.
    pub content_rect: Rect,
    /// Runtime layout diagnostics produced while resolving this frame.
    pub layout_diagnostics: Vec<LayoutDiagnostic>,
    /// Structured overflow summary derived from runtime diagnostics.
    pub overflow: LayoutOverflowSummary,
    /// Per-node geometry diagnostics when enabled in root diagnostics mode.
    pub node_layout_diagnostics: Vec<LayoutNodeDiagnostic>,
}

/// Aggregate overflow counters for one render pass.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LayoutOverflowSummary {
    /// Number of clip-policy overflow events.
    pub clipped: usize,
    /// Number of compress-policy overflow events.
    pub compressed: usize,
    /// Number of skipped child placements caused by overflow handling.
    pub skipped: usize,
    /// Total overflow diagnostic events.
    pub total: usize,
}

impl LayoutOverflowSummary {
    /// Build a summary from runtime layout diagnostics.
    pub fn from_diagnostics(diagnostics: &[LayoutDiagnostic]) -> Self {
        let mut summary = Self::default();
        for diagnostic in diagnostics {
            summary.record(diagnostic.code);
        }
        summary
    }

    /// Accumulate one diagnostic code into the summary.
    fn record(&mut self, code: LayoutDiagnosticCode) {
        match code {
            LayoutDiagnosticCode::OverflowClipped => {
                self.clipped += 1;
                self.total += 1;
            }
            LayoutDiagnosticCode::OverflowSkippedDisjoint
            | LayoutDiagnosticCode::OverflowSkippedCollapsedBounds => {
                self.skipped += 1;
                self.total += 1;
            }
            LayoutDiagnosticCode::OverflowCompressed
            | LayoutDiagnosticCode::ScrollViewContentCompressed => {
                self.compressed += 1;
                self.total += 1;
            }
            LayoutDiagnosticCode::StructuralGapDetected => {}
        }
    }
}

impl Default for RenderResult {
    fn default() -> Self {
        Self {
            measured_size: Size {
                width: 0,
                height: 0,
            },
            actions: Vec::new(),
            resolved_scale: 1.0,
            content_rect: Rect {
                origin: Point { x: 0, y: 0 },
                size: Size {
                    width: 0,
                    height: 0,
                },
            },
            layout_diagnostics: Vec::new(),
            overflow: LayoutOverflowSummary::default(),
            node_layout_diagnostics: Vec::new(),
        }
    }
}
