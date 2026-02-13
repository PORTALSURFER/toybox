impl Renderer {
    /// Blit the offscreen target to the surface and present the frame.
    fn present_output(&self, surface_view: &wgpu::TextureView, output: wgpu::SurfaceTexture) {
        let mut encoder =
            self.device
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("patchbay-gui-present-blit"),
                });
        self.blitter.copy(
            &self.device.device,
            &mut encoder,
            &self.render_target_view,
            surface_view,
        );
        self.device.queue.submit(Some(encoder.finish()));
        output.present();
    }
}
