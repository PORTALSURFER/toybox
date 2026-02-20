//! Error types for host-window creation and rendering setup.

/// Errors returned by the Patchbay GUI system.
#[derive(thiserror::Error, Debug)]
pub enum GuiError {
    /// The host did not provide a parent window.
    #[error("no parent window was provided")]
    NoParent,
    /// The raw window handle is not supported on this platform.
    #[error("unsupported window handle for this platform")]
    UnsupportedHandle,
    /// Failed to create the native window.
    #[error("failed to create Win32 window")]
    WindowCreateFailed,
    /// Failed to locate a compatible GPU adapter.
    #[error("no compatible GPU adapter found")]
    AdapterNotFound,
    /// Surface creation failed.
    #[error("failed to create wgpu surface")]
    Surface(#[source] wgpu::CreateSurfaceError),
    /// Device creation failed.
    #[error("failed to create wgpu device")]
    Device(#[source] wgpu::RequestDeviceError),
    /// Surface has no supported formats.
    #[error("wgpu surface reports no supported formats")]
    SurfaceFormat,
    /// Failed to acquire the next swapchain frame.
    #[error("failed to acquire next swapchain texture")]
    SurfaceAcquire(#[source] wgpu::SurfaceError),
    /// GUI thread failed to start.
    #[error("failed to spawn GUI thread")]
    ThreadSpawn,
    /// Device cache mutex was poisoned.
    #[error("renderer device cache mutex poisoned")]
    DeviceCachePoison,
    /// Frame-capture request could not be started.
    #[error("frame capture is unavailable because no window is open")]
    FrameCaptureUnavailable,
    /// Frame-capture state synchronization failed.
    #[error("frame capture synchronization state is poisoned")]
    FrameCaptureStatePoisoned,
    /// Frame-capture did not complete before timeout.
    #[error("frame capture timed out after {0:?}")]
    FrameCaptureTimeout(std::time::Duration),
    /// Frame readback failed.
    #[error("frame capture readback failed: {0}")]
    FrameCaptureReadback(String),
}
