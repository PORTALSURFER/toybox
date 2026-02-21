
/// UI frame context used to draw widgets and handle input.
pub struct Ui<'a> {
    /// Destination canvas for all widget drawing.
    canvas: &'a mut Canvas,
    /// Immutable input snapshot for the current frame.
    input: &'a InputState,
    /// Mutable per-window UI interaction state.
    state: &'a mut UiState,
    /// Sequential layout cursor and sizing configuration.
    layout: &'a mut Layout,
    /// Theme colors and typography values.
    theme: Theme,
    /// Saved layout scopes used by `with_layout`.
    layout_stack: Vec<Layout>,
    /// Active clip rectangles inherited from declarative container bounds.
    clip_stack: Vec<Rect>,
    /// Nested bounds tracking stack for auto-size containers.
    bounds_stack: Vec<Option<Rect>>,
    /// Vector draw commands collected for the renderer overlay pass.
    vector_commands: Vec<VectorCommand>,
    /// Use vector text instead of CPU bitmap glyphs when available.
    vector_text_enabled: bool,
    /// Use vector shape commands instead of CPU raster primitives when available.
    vector_shapes_enabled: bool,
}
