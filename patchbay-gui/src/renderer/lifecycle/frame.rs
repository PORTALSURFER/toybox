impl Renderer {
    /// Return true if vector text rendering is available.
    pub fn vector_text_available(&self) -> bool {
        self.vector_painter.has_text_font()
    }

    /// Replace the queued vector commands for the next render pass.
    pub fn set_vector_commands(&mut self, commands: Vec<VectorCommand>) {
        self.vector_commands = commands;
    }

    /// Set the surface transform used for the next render pass.
    pub fn set_presentation_transform(&mut self, transform: PresentationTransform) {
        self.presentation_transform = Some(transform);
    }
}
