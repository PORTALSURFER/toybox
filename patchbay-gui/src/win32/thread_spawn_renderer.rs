/// Acquire or create a shared renderer device and construct a renderer.
fn create_renderer(
    device_cache: &Arc<Mutex<Option<Arc<RendererDevice>>>>,
    window: SurfaceWindow,
    size: Size,
) -> Result<Renderer, GuiError> {
    log_line_safe("win32: creating renderer");
    let renderer_device = load_renderer_device(device_cache)?;
    let renderer = Renderer::new_with_device(renderer_device, window, size)?;
    log_line_safe("win32: renderer created");
    Ok(renderer)
}

/// Load the renderer device from cache or create it when missing.
fn load_renderer_device(
    device_cache: &Arc<Mutex<Option<Arc<RendererDevice>>>>,
) -> Result<Arc<RendererDevice>, GuiError> {
    let mut cache = device_cache.lock().map_err(|_| GuiError::DeviceCachePoison)?;
    if let Some(device) = cache.as_ref() {
        return Ok(Arc::clone(device));
    }
    let device = Arc::new(RendererDevice::new()?);
    *cache = Some(Arc::clone(&device));
    Ok(device)
}
