//! Renderer error mapping and recovery helpers.

use crate::host::GuiError;
use crate::logging::log_line_safe;

/// WGPU surface-acquisition states used by the recovery policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SurfaceAcquireStatus {
    /// A surface texture was acquired without a recoverable status.
    Success,
    /// A surface texture was acquired, but the surface should be reconfigured.
    Suboptimal,
    /// Acquisition timed out and should be retried later.
    Timeout,
    /// The surface is currently occluded and should be retried later.
    Occluded,
    /// The surface configuration is outdated.
    Outdated,
    /// The surface was lost and needs recovery.
    Lost,
    /// Acquisition raised a validation error.
    Validation,
}

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
pub(crate) fn should_reconfigure_surface(err: &wgpu::CurrentSurfaceTexture) -> bool {
    should_reconfigure_surface_status(classify_surface_status(err))
}

/// Classify a WGPU surface acquisition result without consuming a texture.
fn classify_surface_status(status: &wgpu::CurrentSurfaceTexture) -> SurfaceAcquireStatus {
    match status {
        wgpu::CurrentSurfaceTexture::Success(_) => SurfaceAcquireStatus::Success,
        wgpu::CurrentSurfaceTexture::Suboptimal(_) => SurfaceAcquireStatus::Suboptimal,
        wgpu::CurrentSurfaceTexture::Timeout => SurfaceAcquireStatus::Timeout,
        wgpu::CurrentSurfaceTexture::Occluded => SurfaceAcquireStatus::Occluded,
        wgpu::CurrentSurfaceTexture::Outdated => SurfaceAcquireStatus::Outdated,
        wgpu::CurrentSurfaceTexture::Lost => SurfaceAcquireStatus::Lost,
        wgpu::CurrentSurfaceTexture::Validation => SurfaceAcquireStatus::Validation,
    }
}

/// Return true when a classified surface status requires one reconfiguration.
fn should_reconfigure_surface_status(status: SurfaceAcquireStatus) -> bool {
    matches!(
        status,
        SurfaceAcquireStatus::Suboptimal
            | SurfaceAcquireStatus::Lost
            | SurfaceAcquireStatus::Outdated
    )
}

#[cfg(test)]
mod tests {
    use super::{SurfaceAcquireStatus, should_reconfigure_surface_status};

    #[test]
    fn recoverable_surface_statuses_reconfigure_once() {
        for status in [
            SurfaceAcquireStatus::Suboptimal,
            SurfaceAcquireStatus::Lost,
            SurfaceAcquireStatus::Outdated,
        ] {
            assert!(should_reconfigure_surface_status(status));
        }
    }

    #[test]
    fn non_recoverable_surface_statuses_do_not_reconfigure() {
        for status in [
            SurfaceAcquireStatus::Success,
            SurfaceAcquireStatus::Timeout,
            SurfaceAcquireStatus::Occluded,
            SurfaceAcquireStatus::Validation,
        ] {
            assert!(!should_reconfigure_surface_status(status));
        }
    }
}
