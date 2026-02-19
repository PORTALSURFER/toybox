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
        }
    }
}
