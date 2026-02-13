/// Fully initialized rendering resources used to construct a renderer instance.
struct RendererInitResources {
    /// GPU renderer used to rasterize the Vello scene.
    vello_renderer: vello::Renderer,
    /// Offscreen texture used as the Vello render target.
    render_target_texture: wgpu::Texture,
    /// View into the offscreen render target texture.
    render_target_view: wgpu::TextureView,
    /// Utility blitter used to copy to the swapchain texture.
    blitter: wgpu::util::TextureBlitter,
    /// Texture backing the uploaded CPU canvas.
    canvas_texture: wgpu::Texture,
    /// Vello image handle for the canvas texture.
    canvas_image: vello::peniko::ImageData,
    /// Initial logical canvas size.
    initial_canvas_size: Size,
}
