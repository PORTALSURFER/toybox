//! Renderer error mapping and recovery helpers.

use crate::host::GuiError;
use crate::logging::log_line_safe;

/// Map Vello initialization failures into GUI host-facing errors.
pub(crate) fn map_vello_init_error(err: vello::Error) -> GuiError {
    log_line_safe(&format!("renderer: vello init error: {err:?}"));
    match err {
        vello::Error::NoCompatibleDevice => GuiError::AdapterNotFound,
        vello::Error::UnsupportedSurfaceFormat => GuiError::SurfaceFormat,
        _ => GuiError::SurfaceFormat,
    }
}

/// Return true when the surface should be reconfigured before retrying.
pub(crate) fn should_reconfigure_surface(err: &wgpu::SurfaceError) -> bool {
    matches!(err, wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated)
}
